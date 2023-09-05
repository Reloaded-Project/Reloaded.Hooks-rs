extern crate alloc;
use core::{hash::Hash, mem::size_of, slice};

use alloc::string::ToString;
use alloc::vec::Vec;
use smallvec::SmallVec;

use super::{
    function_attribute::FunctionAttribute,
    function_info::{FunctionInfo, ParameterType},
    jit::{compiler::JitCapabilities, operation::Operation, return_operation::ReturnOperation},
    traits::register_info::RegisterInfo,
};
use crate::{
    api::{
        errors::wrapper_generation_error::WrapperGenerationError, function_attribute::StackCleanup,
        jit::operation_aliases::*,
    },
    optimize::{
        combine_push_operations::{merge_pop_operations, merge_push_operations},
        eliminate_common_callee_saved_registers::eliminate_common_callee_saved_registers,
        merge_stackalloc_and_return::merge_stackalloc_and_return,
        optimize_reg_parameters::optimize_push_pop_parameters,
        optimize_stack_parameters::{optimize_stack_parameters, update_stack_push_offsets},
        reorder_mov_sequence::reorder_mov_sequence,
    },
};

/// Options and additional context necessary for the wrapper generator.
#[derive(Clone, Copy)]
pub struct WrapperInstructionGeneratorOptions<'a, TFunctionInfo>
where
    TFunctionInfo: FunctionInfo,
{
    /// True if the code is within relative jump distance; and JIT
    /// can emit relative jump to target function.
    pub can_generate_relative_jumps: bool,

    /// Stack offset upon entry into the wrapper generator.
    /// This is 0 for architectures with a link register, or [usize] for architectures which have
    /// return addresses on stack.
    pub stack_entry_alignment: usize,

    /// Returns the functionalities supported by this JIT.
    /// These functionalities affect code generation performed by this library.
    pub jit_capabilities: &'a [JitCapabilities],

    /// Address of the function to be called.
    pub target_address: usize,

    /// Information about the function for which the wrapper needs to be generated.
    pub function_info: &'a TFunctionInfo,

    /// If this parameter is specified, the wrapper will inject an additional parameter
    /// with the specified value into the target (called) function.
    ///
    /// # Remarks
    ///
    /// This is useful for example when the target function is your own method when hooking
    /// and you want to inject a 'this' pointer.
    pub injected_parameter: Option<usize>,

    /// Enables optimization of wrappers.
    /// This should only ever be disabled for debugging purposes.
    pub enable_optimizations: bool,
}

/// Creates the instructions responsible for wrapping one object kind to another.
///
/// # Parameters
///
/// - `from_convention` - The calling convention to convert to `to_convention`. This is the convention of the function (`options.target_address`) called.
/// - `to_convention` - The target convention to which convert to `from_convention`. This is the convention of the function returned.
/// - `options` - The parameters for this wrapper generation task.
///
/// # Remarks
///
/// This process is documented in the Wiki under `Design Docs -> Wrapper Generation`.
#[allow(warnings)]
pub fn generate_wrapper_instructions<
    TRegister: RegisterInfo + Hash + Eq + Copy + Default,
    TFunctionAttribute: FunctionAttribute<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: &TFunctionAttribute,
    to_convention: &TFunctionAttribute,
    options: WrapperInstructionGeneratorOptions<TFunctionInfo>,
) -> Result<Vec<Operation<TRegister>>, WrapperGenerationError> {
    let mut ops = Vec::<Operation<TRegister>>::with_capacity(32);
    let mut stack_pointer =
        options.stack_entry_alignment + from_convention.reserved_stack_space() as usize;

    // Backup Always Saved Registers (LR)
    for register in to_convention.always_saved_registers() {
        ops.push(Push::new(*register).into());
        stack_pointer += register.size_in_bytes();
    }

    // Backup callee saved registers
    let callee_saved_regs = eliminate_common_callee_saved_registers(
        from_convention.callee_saved_registers(),
        to_convention.callee_saved_registers(),
    );

    for register in &callee_saved_regs {
        ops.push(Push::new(*register).into());
        stack_pointer += register.size_in_bytes();
    }

    let after_backup_sp = stack_pointer as usize;

    // Insert Dummy for Stack Alignment
    let align_stack_idx = ops.len();
    ops.push(StackAlloc::new(0).into()); // insert a dummy for now.

    // Re-push stack parameters of function returned (right to left)
    let num_params = options.function_info.parameters().len();
    let returned_stack_params_size = (size_of::<ParameterType>() * num_params);
    let returned_reg_params_size = (size_of::<(ParameterType, TRegister)>() * num_params);
    let mut setup_params_ops = SmallVec::<[Operation<TRegister>; 32]>::new_const();
    let mut callee_cleanup_return_size = stack_pointer - options.stack_entry_alignment;

    // Note: Allocating on stack to avoid heap allocations.
    alloca::with_alloca(returned_stack_params_size + returned_reg_params_size, |f| {
        // Find out which parameters are stack spilled and which are in registers
        // Note: We use our stack allocated memory as buffer for returned stack + reg parameters
        // Ugly workaround for lack of native alloca in Rust.
        let mut returned_stack_params_buf = unsafe {
            slice::from_raw_parts_mut::<ParameterType>(f.as_ptr() as *mut ParameterType, num_params)
        };
        let mut returned_reg_params_buf = unsafe {
            slice::from_raw_parts_mut::<(ParameterType, TRegister)>(
                (f.as_ptr() as usize + returned_stack_params_size)
                    as *mut (ParameterType, TRegister),
                num_params,
            )
        };

        let fn_returned_params = options.function_info.get_parameters_as_slice(
            to_convention,
            &mut returned_stack_params_buf,
            &mut returned_reg_params_buf,
        );

        /*
            Context [x64 as example].

            At the current moment in time, the variable before the return address is at -stack_pointer.

            On platforms like ARM that don't do stack returns, this is natural, but on platforms like
            x64 where return is done via address on stack, `options.stack_entry_alignment` offsets this
            such that -stack_pointer is guaranteed to points to the base of the last stack parameter.

            From there, we can re push registers, just have to be careful to keep track of SP, which is
            raising as we push more.
        */

        let mut current_offset = stack_pointer as isize;
        for param in fn_returned_params.0.iter().rev() {
            let param_size_bytes = param.size_in_bytes();
            setup_params_ops.push(PushStack::new(current_offset, param_size_bytes).into());
            current_offset += (param_size_bytes * 2) as isize;
            stack_pointer += param_size_bytes;
            callee_cleanup_return_size += param_size_bytes;
        }

        // Push register parameters of function returned (right to left)
        for param in fn_returned_params.1.iter().rev() {
            setup_params_ops.push(Push::new(param.1).into());
            stack_pointer += param.0.size_in_bytes();
        }
    });

    // Inject parameter (if applicable)
    if let Some(injected_value) = options.injected_parameter {
        setup_params_ops.push(PushConst::new(injected_value).into());
        stack_pointer += size_of::<usize>();
    }

    // Pop register parameters of the function being called (left to right)
    let fn_called_params = options.function_info.get_parameters_as_vec(from_convention);
    for param in fn_called_params.1.iter() {
        setup_params_ops.push(Pop::new(param.1).into());
        stack_pointer -= param.0.size_in_bytes();
    }

    // Optimize the parameter pushing process
    let scratch_register = from_convention.scratch_register();
    let mut optimized = setup_params_ops.as_mut_slice();
    let mut new_optimized: Vec<Operation<TRegister>> = Vec::new();

    if options.enable_optimizations {
        optimized = optimize_stack_parameters(optimized);
        optimized = optimize_push_pop_parameters(optimized);

        let reordered = reorder_mov_sequence(optimized, &scratch_register); // perf hit
        if reordered.is_some() {
            new_optimized = unsafe { reordered.unwrap_unchecked() };
            optimized = &mut new_optimized[..];
        }

        if options
            .jit_capabilities
            .contains(&JitCapabilities::CanMultiPush)
        {
            optimized = merge_push_operations(optimized); // perf hit
            optimized = merge_pop_operations(optimized);
        }
    }

    // Now write the correct stack alignment value, and correct offsets
    // We wrote the code earlier, ignoring stack alignment because we didn't know it yet, but now
    // we know, so items might need adjusting here.
    let stack_misalignment = stack_pointer as u32 % from_convention.required_stack_alignment();
    if stack_misalignment != 0 {
        ops[align_stack_idx] = StackAlloc::new(stack_misalignment as i32).into();
        stack_pointer += stack_misalignment as usize;
        update_stack_push_offsets(optimized, stack_misalignment as i32);
    } else {
        ops.remove(align_stack_idx);
    }

    ops.extend_from_slice(optimized);

    // Reserve required space for function called
    let reserved_space = from_convention.reserved_stack_space() as i32;
    if reserved_space != 0 {
        ops.push(StackAlloc::new(reserved_space).into());
        stack_pointer += from_convention.reserved_stack_space() as usize;
    }

    // Call the Method
    if options.can_generate_relative_jumps {
        ops.push(CallRel::new(options.target_address).into());
    } else {
        if scratch_register.is_none() {
            return Err(WrapperGenerationError::NoScratchRegister(
                "Needed for Absolute Call.".to_string(),
            ));
        }

        ops.push(
            CallAbs {
                scratch_register: scratch_register.unwrap(),
                target_address: options.target_address,
            }
            .into(),
        );
    }

    // Move return value to proper register
    let fn_called_return_reg = from_convention.return_register();
    let fn_returned_return_reg = to_convention.return_register();
    if fn_called_return_reg != fn_returned_return_reg {
        ops.push(Mov::new(fn_called_return_reg, fn_returned_return_reg).into());
    }

    // Fix the stack
    let stack_ofs = if from_convention.stack_cleanup_behaviour() == StackCleanup::Callee {
        stack_misalignment as isize
    } else {
        after_backup_sp as isize - stack_pointer as isize
    };

    if stack_ofs != 0 {
        ops.push(StackAlloc::new(stack_ofs as i32).into());
    }

    // Pop Callee Saved Registers
    for register in callee_saved_regs.iter().rev() {
        ops.push(Pop::new(*register).into());
    }

    // Pop Always Saved Registers (like LR)
    for register in to_convention.always_saved_registers().iter().rev() {
        ops.push(Pop::new(*register).into());
    }

    if to_convention.stack_cleanup_behaviour() == StackCleanup::Callee {
        ops.push(ReturnOperation::new(callee_cleanup_return_size).into());
    } else {
        ops.push(ReturnOperation::new(0).into());
    }

    if options.enable_optimizations {
        merge_stackalloc_and_return(&mut ops);
    }

    Ok(ops)
}

#[cfg(test)]
pub mod tests {
    use crate::api::jit::operation::Operation::MultiPush;
    use crate::{
        api::function_info::ParameterType, helpers::test_helpers::MockRegister::*,
        helpers::test_helpers::*,
    };

    use super::*;
    use smallvec::smallvec;

    fn get_x86_jit_capabilities() -> Vec<JitCapabilities> {
        vec![
            JitCapabilities::CanEncodeIPRelativeCall,
            JitCapabilities::CanEncodeIPRelativeJump,
            JitCapabilities::CanMultiPush,
        ]
    }

    // X86-LIKE TESTS //

    #[test]
    fn ms_thiscall_to_cdecl_unoptimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // re-push right param
        assert_eq!(vec[1], PushStack::new(nint * 3, nint as usize).into()); // re-push left param
        assert_eq!(vec[2], Pop::new(R1).into()); // pop left param into reg
        assert_eq!(vec[3], CallRel::new(4096).into());
        assert_eq!(vec[4], Return::new(0).into()); // caller cleanup, so no offset here
    }

    #[test]
    fn ms_thiscall_to_cdecl_optimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // re-push right param
        assert_eq!(vec[1], MovFromStack::new((nint * 3) as i32, R1).into()); // mov left param to register
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new(0).into()); // caller cleanup, so no offset here
    }

    #[test]
    fn ms_cdecl_to_thiscall_unoptimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // push right param
        assert_eq!(vec[1], Push::new(R1).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new(-(nint * 2) as i32).into()); // caller stack cleanup
        assert_eq!(vec[4], Return::new(nint as usize).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_cdecl_to_thiscall_optimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // push right param
        assert_eq!(vec[1], Push::new(R1).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(
            vec[3],
            Return::new((nint * 2) as usize + nint as usize).into()
        ); // cleanup 2*nint (cdecl) + nint (thiscall)
    }

    #[test]
    fn ms_cdecl_to_fastcall_unoptimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            &FASTCALL_LIKE_FUNCTION_ATTRIBUTE,
            false,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_eq!(vec[0], Push::new(R2).into()); // push right param
        assert_eq!(vec[1], Push::new(R1).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new(-(nint * 2) as i32).into()); // caller stack cleanup
        assert_eq!(vec[4], Return::new(0).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_cdecl_to_fastcall_optimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            &FASTCALL_LIKE_FUNCTION_ATTRIBUTE,
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], MultiPush(smallvec![Push::new(R2), Push::new(R1)])); // push right param
        assert_eq!(vec[1], CallRel::new(4096).into());
        assert_eq!(vec[2], Return::new((nint * 2) as usize).into()); // caller stack cleanup (2 cdecl parameters)
    }

    // EXTRA X86-LIKE TESTS //

    #[test]
    fn ms_stdcall_to_thiscall_optimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &STDCALL_LIKE_FUNCTION_ATTRIBUTE,
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // push right param
        assert_eq!(vec[1], Push::new(R1).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new(nint as usize).into()); // callee stack cleanup (only non-register parameter)
    }

    #[test]
    fn ms_thiscall_to_stdcall_optimized() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters(
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            &STDCALL_LIKE_FUNCTION_ATTRIBUTE,
            true,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], PushStack::new(nint, nint as usize).into()); // re-push right param
        assert_eq!(vec[1], MovFromStack::new((nint * 3) as i32, R1).into()); // mov left param to register
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new((nint * 2) as usize).into()); // caller cleanup, so no offset here
    }

    /// Creates the instructions responsible for wrapping one object kind to another.
    ///
    /// # Parameters
    ///
    /// - `from_convention` - The calling convention to convert to `to_convention`. This is the convention of the function (`options.target_address`) called.
    /// - `to_convention` - The target convention to which convert to `from_convention`. This is the convention of the function returned.
    /// - `optimized` - Whether to generate optimized code
    fn two_parameters(
        from_convention: &MockFunctionAttribute,
        to_convention: &MockFunctionAttribute,
        optimized: bool,
    ) -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError> {
        // Two parameters
        let mock_function = MockFunction {
            parameters: vec![ParameterType::nint, ParameterType::nint],
        };

        let capabiltiies = get_x86_jit_capabilities();
        let options = get_common_options(optimized, &mock_function, &capabiltiies);
        generate_wrapper_instructions(from_convention, to_convention, options)
    }

    fn get_common_options<'a>(
        optimized: bool,
        mock_function: &'a MockFunction,
        capabilties: &'a [JitCapabilities],
    ) -> WrapperInstructionGeneratorOptions<'a, MockFunction> {
        WrapperInstructionGeneratorOptions {
            stack_entry_alignment: size_of::<isize>(), // no_alignment
            target_address: 0x1000,                    // some arbitrary address
            function_info: mock_function,
            injected_parameter: None, // some arbitrary value
            jit_capabilities: capabilties,
            can_generate_relative_jumps: true,
            enable_optimizations: optimized,
        }
    }
}
