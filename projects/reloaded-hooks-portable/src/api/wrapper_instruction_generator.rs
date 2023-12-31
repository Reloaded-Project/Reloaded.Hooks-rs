extern crate alloc;
use super::jit::compiler::Jit;
use super::{
    calling_convention_info::CallingConventionInfo,
    function_info::{FunctionInfo, ParameterType},
    jit::{compiler::JitCapabilities, operation::Operation, return_operation::ReturnOperation},
    traits::register_info::{find_register_with_category, RegisterCategory, RegisterInfo},
};
use crate::optimize::decompose_push_pop_operations::{
    decompose_pop_operations_ex, decompose_push_operations,
};
use crate::optimize::merge_stackalloc_operations::combine_stack_alloc_operations;
use crate::{
    api::{
        calling_convention_info::StackCleanup,
        errors::wrapper_generation_error::WrapperGenerationError, jit::operation_aliases::*,
    },
    optimize::{
        combine_push_operations::{merge_pop_operations, merge_push_operations},
        eliminate_common_callee_saved_registers::eliminate_common_callee_saved_registers,
        optimize_push_pop_parameters::{optimize_push_pop_parameters, update_stack_push_offsets},
        reorder_mov_sequence::reorder_mov_sequence,
    },
};
use alloc::vec::Vec;
use alloc::{rc::Rc, string::ToString};
use core::cell::RefCell;
use core::{hash::Hash, mem::size_of, slice};
use smallvec::SmallVec;

/// Overkill in practice, but just in case, any leftover memory at end of buffers will
/// be swept up by other hooks which can better estimate a byte count than we can for wrapper generation.
///
/// # Rationale
///
/// Most space in generated code comes from re-pushing parameters if needed, or shifting them between
/// registers. All the other instructions amount to roughly 12 bytes (x86) or 16 bytes (ARM) for most generated code.
///
/// The average length of instruction used to 'push' a parameter, is 4 bytes. (Note: x86 can be shorter
/// when moving between registers).
///
/// Assuming we have left over space of a generous 160 bytes (for brevity), the maximum number of
/// parameters we can push is 160 / 4 = 40 parameters.
///
/// This is very unlikely to be the case in practice, as functions with that many parameters would
/// not require the generation of a wrapper, they would be inlined or use the default call convention.
///
/// Who would pass 40 parameters to a function anyway !?
pub const MAX_WRAPPER_LENGTH: usize = 192;

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
    pub jit_capabilities: JitCapabilities,

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

    /// Size of the standard register. This is used to determine the size of the padding
    /// required for the stack in case the last pushed item during callee save is larger
    /// than the standard register size.
    pub standard_register_size: usize,
}

/// Creates a new instance of WrapperInstructionGeneratorOptions with the given parameters.
///
/// # Type Parameters
///
/// * `TFunctionInfo` - Information about the function for which the wrapper needs to be generated.
/// * `TRegister` - Register information type.
/// * `TJit` - Jit implementation type.
///
/// # Parameters
///
/// * `can_generate_relative_jumps` - Whether the code is within relative jump distance.
/// * `target_address` - Address of the function to be called.
/// * `function_info` - Reference to information about the function.
/// * `injected_parameter` - Optional injected parameter value.
///
/// # Returns
///
/// A new instance of WrapperInstructionGeneratorOptions.
pub fn new_wrapper_instruction_generator_options<TFunctionInfo, TRegister, TJit>(
    can_generate_relative_jumps: bool,
    target_address: usize,
    function_info: &TFunctionInfo,
    injected_parameter: Option<usize>,
) -> WrapperInstructionGeneratorOptions<'_, TFunctionInfo>
where
    TFunctionInfo: FunctionInfo,
    TRegister: RegisterInfo + Copy + Clone,
    TJit: Jit<TRegister>,
{
    WrapperInstructionGeneratorOptions {
        stack_entry_alignment: TJit::stack_entry_misalignment() as usize,
        standard_register_size: TJit::standard_register_size(),
        jit_capabilities: TJit::get_jit_capabilities(),
        enable_optimizations: true,
        can_generate_relative_jumps,
        target_address,
        function_info,
        injected_parameter,
    }
}

/// Creates the instructions responsible for wrapping one object kind to another.
///
/// # Parameters
///
/// - `conv_called` - The calling convention to convert to `conv_current`. This is the convention of the function (`options.target_address`) called.
/// - `conv_current` - The target convention to which convert to `conv_called`. This is the convention of the function returned.
/// - `options` - The parameters for this wrapper generation task.
///
/// # Remarks
///
/// This process is documented in the Wiki under `Design Docs -> Wrapper Generation`.
#[allow(warnings)]
pub fn generate_wrapper_instructions<
    TRegister: RegisterInfo + Hash + Eq + Copy + Default + 'static,
    TFunctionAttribute: CallingConventionInfo<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    conv_called: &TFunctionAttribute,
    conv_current: &TFunctionAttribute,
    options: &WrapperInstructionGeneratorOptions<TFunctionInfo>,
) -> Result<Vec<Operation<TRegister>>, WrapperGenerationError> {
    let mut ops = Vec::<Operation<TRegister>>::with_capacity(32);
    let mut stack_pointer =
        options.stack_entry_alignment + conv_current.reserved_stack_space() as usize;
    let standard_reg_size = options.standard_register_size;
    let called_reserved_space = conv_called.reserved_stack_space();
    let mut profitable_push_decompose = false; // Transform Push -> MovToStack + StackAlloc
    let mut profitable_pop_decompose = false; // Transform Pop -> MovToStack + StackAlloc

    /*
        Rough Summary of this function.
        To whoever reads this, god bless you.

        1. **Initialize Operations Vector**
        - Initializes a vector `ops` to hold the generated operations.
        - Sets the initial `stack_pointer` position based on `options.stack_entry_alignment` and
          `conv_called.reserved_stack_space()`.

        2. **Backup Registers**
        - Backs up the "always saved" registers and the callee-saved registers of `conv_current`.
        - The callee-saved registers common to both conventions are not backed up.

        3. **Insert Dummy Stack Allocation**
        - Inserts a dummy `StackAlloc` operation for stack alignment, which will be updated later.
        - This is used to align stack to the required alignment of `conv_called`.

        4. **Re-push Stack and Register Parameters**
        - Re-pushes the stack and register parameters of `conv_current` to stack.
        - Adjusts the `stack_pointer` accordingly.

        5. **Inject Parameter (If Applicable)**
        - If there is an injected parameter specified in `options`, it pushes it onto the stack and
          adjusts the `stack_pointer`.

        6. **Pop Register Parameters of the Called Function**
        - Pops the register parameters of the function being called (`conv_called`).
        - Adjusts the `stack_pointer` accordingly.

        7. **Optimize Parameter Pushing and Popping**
        - If optimizations are enabled, it optimizes the push/pop operations generated in steps 4-6.
        - This includes reordering, merging, and optimizing the move sequences.

        8. **Update Stack Alignment**
        - Calculates the stack misalignment and updates the previously inserted dummy `StackAlloc`
          operation with the correct value.
        - Updates the offsets of stack push operations accordingly.

        9. **Reserve Stack Space for Called Function**
        - Reserves the required stack space for the function being called as specified by `conv_called`.

        10. **Call the Target Method**
        - Depending on the availability of relative jumps and scratch registers, it generates either
            a relative or an absolute call to the target method specified in `options`.

        11. **Move Return Value to Proper Register**
        - If the return registers of the called and returned functions differ, it generates a move
            operation to place the return value in the correct register.

        12. **Fix the Stack**
        - Adjusts the stack pointer based on the stack cleanup behaviour of the `conv_called`.

        13. **Restore Callee Saved Registers**
        - Pops the callee-saved registers and always saved registers (in reverse order they were pushed).

        14. **Return Operation**
        - Generates a return operation with appropriate stack cleanup size, based on the `conv_current`
          stack cleanup behaviour.

        15. **Run Final Optimization Passes**
        - Merge StackAlloc and Return Operations (If Supported)
        - Merge Push & Pop Sequences into MultiPop and MultiPush.

        16. **Return Generated Operations**
        - Returns the generated list of operations as `Ok(ops)`, or an error if any issues are
          encountered during the generation.
    */

    // Note: Scratch registers are sourced from method returned (wrapper), not method called (wrapped),
    //       based on caller saved registers.
    //
    // In case of a hook of a custom method
    //    - conv_called (wrapped): cdecl
    //    - conv_current (wrapper): 'usercall'
    //
    // In this case, we are calling `conv_called` from the wrapper we create which is still
    // `conv_current`. Therefore, we need to use the scratch registers of `conv_current`.
    let mut scratch_registers = Rc::new(RefCell::new(conv_current.caller_saved_registers()));
    // Note: We still need to eliminate caller saved regs used as function parameters. This will be done later.

    // Backup Always Saved Registers (LR, etc.)
    for register in conv_current.always_saved_registers() {
        ops.push(Push::new(*register).into());
        stack_pointer += register.size_in_bytes();
    }

    // Backup callee saved registers
    let mut callee_saved_regs = eliminate_common_callee_saved_registers(
        conv_called.callee_saved_registers(),
        conv_current.callee_saved_registers(),
    );

    // Sort registers in ascending order of size
    callee_saved_regs.sort_by(|a, b| a.size_in_bytes().cmp(&b.size_in_bytes()));

    for register in &callee_saved_regs {
        ops.push(Push::new(*register).into());
        stack_pointer += register.size_in_bytes();
    }

    // Add extra padding space if last pushed item is greater than standard reg size
    // this is required so any further pushes (e.g. re-pushed parameters) don't overwrite
    // the upper bits of the last pushed larger than regular register.
    let last = callee_saved_regs.last();
    let mut callee_saved_reg_padding = 0;
    if last.is_some() {
        callee_saved_reg_padding = last.unwrap().size_in_bytes() - standard_reg_size;
        if callee_saved_reg_padding > 0 {
            ops.push(StackAlloc::new(callee_saved_reg_padding as i32).into());
            profitable_push_decompose = true;
        }

        stack_pointer += callee_saved_reg_padding;
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
    let mut callee_cleanup_return_size = 0;

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
            conv_current,
            &mut returned_stack_params_buf,
            &mut returned_reg_params_buf,
        );

        // Eliminate caller saved regs from scratch which are used as function parameters
        // To get our 'true' scratch registers.
        (*scratch_registers)
            .borrow_mut()
            .retain(|&f| !fn_returned_params.1.iter().any(|reg| f == reg.1));

        /*
            Context [x64 as example].

            At the current moment in time, the variable before the return address is at -stack_pointer.

            On platforms like ARM that don't do stack returns, this is natural, but on platforms like
            x64 where return is done via address on stack, `options.stack_entry_alignment` offsets this
            such that -stack_pointer is guaranteed to points to the base of the last stack parameter.

            From there, we can re push registers, just have to be careful to keep track of SP, which is
            raising as we push more.
        */

        // Re-push stack parameters of function returned (right to left)
        let mut current_offset = stack_pointer as isize;
        for param in fn_returned_params.0.iter().rev() {
            let param_size_bytes = param.size_in_bytes();
            setup_params_ops.push(
                PushStack::new(
                    current_offset as i32,
                    param_size_bytes as u32,
                    scratch_registers.clone(),
                )
                .into(),
            );
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
        let reg = find_register_with_category(
            RegisterCategory::GeneralPurpose,
            &scratch_registers.borrow(),
        );
        setup_params_ops.push(PushConst::new(injected_value, reg).into());
        stack_pointer += size_of::<usize>();
    }

    // Pop register parameters of the function being called (left to right)
    let fn_called_params = options.function_info.get_parameters_as_vec(conv_called);
    for param in fn_called_params.1.iter() {
        setup_params_ops.push(Pop::new(param.1).into());
        stack_pointer -= param.0.size_in_bytes();
    }

    // Optimize the parameter pushing process
    let mut optimized = setup_params_ops.as_mut_slice();
    let mut new_optimized: Vec<Operation<TRegister>> = Vec::new();

    if options.enable_optimizations {
        optimized = optimize_push_pop_parameters(optimized);

        let reordered = reorder_mov_sequence(optimized, &scratch_registers.borrow()); // perf hit
        if reordered.is_some() {
            new_optimized = unsafe { reordered.unwrap_unchecked() };
            optimized = &mut new_optimized[..];
        }
    }

    // Now write the correct stack alignment value, and correct offsets
    // We wrote the code earlier, ignoring stack alignment because we didn't know it yet, but now
    // we know, so items might need adjusting here.
    let stack_misalignment = stack_pointer as u32 % conv_called.required_stack_alignment();
    if stack_misalignment != 0 {
        ops[align_stack_idx] = StackAlloc::new(stack_misalignment as i32).into();
        stack_pointer += stack_misalignment as usize;
        profitable_push_decompose = true;
        update_stack_push_offsets(optimized, stack_misalignment as i32);
    } else {
        ops.remove(align_stack_idx);
    }

    ops.extend_from_slice(optimized);

    // Reserve required space for function called
    if called_reserved_space != 0 {
        ops.push(StackAlloc::new(called_reserved_space as i32).into());
        profitable_push_decompose = true;
    }

    // Call the Method
    if options
        .jit_capabilities
        .contains(JitCapabilities::CAN_RELATIVE_JUMP_TO_ANY_ADDRESS)
        || options.can_generate_relative_jumps
    {
        ops.push(CallRel::new(options.target_address).into());
    } else {
        let abs_call_register = find_register_with_category(
            RegisterCategory::GeneralPurpose,
            &scratch_registers.borrow(),
        );

        if abs_call_register.is_none() {
            return Err(WrapperGenerationError::NoScratchRegister(
                "No General Purpose Scratch Register Found. Needed for Absolute Call.".to_string(),
            ));
        }

        ops.push(
            CallAbs {
                scratch_register: unsafe { abs_call_register.unwrap_unchecked().extend() },
                target_address: options.target_address,
            }
            .into(),
        );
    }

    // Move return value to proper register
    let fn_called_return_reg = conv_called.return_register();
    let fn_returned_return_reg = conv_current.return_register();
    if fn_called_return_reg != fn_returned_return_reg {
        ops.push(Mov::new(fn_called_return_reg, fn_returned_return_reg).into());
    }

    // Fix the stack
    let stack_ofs = if conv_called.stack_cleanup_behaviour() == StackCleanup::Callee {
        stack_misalignment as isize
            - called_reserved_space as isize
            - callee_saved_reg_padding as isize
    } else {
        after_backup_sp as isize
            - stack_pointer as isize
            - called_reserved_space as isize
            - callee_saved_reg_padding as isize
    };

    if stack_ofs != 0 {
        ops.push(StackAlloc::new(stack_ofs as i32).into());
        profitable_pop_decompose = true;
    }

    // Pop Callee Saved Registers
    for register in callee_saved_regs.iter().rev() {
        ops.push(Pop::new(*register).into());
    }

    // Pop Always Saved Registers (like LR)
    for register in conv_current.always_saved_registers().iter().rev() {
        ops.push(Pop::new(*register).into());
    }

    if conv_current.stack_cleanup_behaviour() == StackCleanup::Callee {
        ops.push(ReturnOperation::new(callee_cleanup_return_size).into());
    } else {
        ops.push(ReturnOperation::new(0).into());
    }

    // Final optimizations which pass over all instructions.
    if options.enable_optimizations {
        // TODO: Our push-ing could be optimized for architectures that are neither ARM
        // (can push multiple regs at once) or X86 (explicit 'push' instruction).

        // Right now our code assumes that the architecture has either of those things,
        // but if neither is true, the code generation may not be the most efficient possible.

        if options
            .jit_capabilities
            .contains(JitCapabilities::CAN_MOV_TO_STACK)
        {
            if profitable_push_decompose {
                // Transform Push -> MovToStack + StackAlloc
                decompose_push_operations(&mut ops, options.standard_register_size);
            }

            if profitable_pop_decompose {
                // Transform Pop -> MovToStack + StackAlloc
                decompose_pop_operations_ex(&mut ops, options.standard_register_size);
            }

            // Merge Multiple in-a-row StackAlloc Into one.
            // This merges stackalloc as result of padding
            // end of callee saved registers with, stackalloc.
            combine_stack_alloc_operations(&mut ops);
        }

        if options
            .jit_capabilities
            .contains(JitCapabilities::CAN_MULTI_PUSH)
        {
            merge_push_operations(&mut ops);
            merge_pop_operations(&mut ops);
        }
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

    fn get_x86_jit_capabilities() -> JitCapabilities {
        JitCapabilities::CAN_ENCODE_IP_RELATIVE_CALL
            | JitCapabilities::CAN_ENCODE_IP_RELATIVE_JUMP
            | JitCapabilities::CAN_MULTI_PUSH
    }

    // EXTRA TESTS //

    #[test]
    fn ms_thiscall_to_cdecl_unoptimized_with_call_absolute() {
        let nint = size_of::<isize>() as isize;
        let result = two_parameters_with_address(
            &THISCALL_LIKE_FUNCTION_ATTRIBUTE,
            &CDECL_LIKE_FUNCTION_ATTRIBUTE,
            false,
            0xFFFFFFFF,
        );

        assert!(result.is_ok());
        let vec: Vec<Operation<MockRegister>> = result.unwrap();
        assert_eq!(vec.len(), 5);
        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_push_stack(&vec[1], nint * 3, nint); // re-push left param
        assert_eq!(vec[2], Pop::new(R1).into()); // pop left param into reg
        assert_eq!(vec[3], CallAbs::new(0xFFFFFFFF).into());
        assert_eq!(vec[4], Return::new(0).into()); // caller cleanup, so no offset here
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

        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_push_stack(&vec[1], nint * 3, nint); // re-push left param
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
        assert_push_stack(&vec[0], nint, nint); // re-push right param
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
        assert_push_stack(&vec[0], nint, nint); // push right param
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
        assert_eq!(vec.len(), 5);
        assert_push_stack(&vec[0], nint, nint); // push right param
        assert_eq!(vec[1], Push::new(R1).into()); // push left param
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], StackAlloc::new((-nint * 2) as i32).into()); // caller cleanup 2*nint (cdecl)
        assert_eq!(vec[4], Return::new(nint as usize).into()); // return, popping nint from stack (1 thiscall stack parameter)
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
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], MultiPush(smallvec![Push::new(R2), Push::new(R1)])); // push right param
        assert_eq!(vec[1], CallRel::new(4096).into());
        assert_eq!(vec[2], StackAlloc::new((-nint * 2) as i32).into()); // caller stack cleanup (2 cdecl parameters)
        assert_eq!(vec[3], Return::new(0).into());
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
        assert_push_stack(&vec[0], nint, nint); // push right param
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
        assert_push_stack(&vec[0], nint, nint); // re-push right param
        assert_eq!(vec[1], MovFromStack::new((nint * 3) as i32, R1).into()); // mov left param to register
        assert_eq!(vec[2], CallRel::new(4096).into());
        assert_eq!(vec[3], Return::new((nint * 2) as usize).into()); // caller cleanup, so no offset here
    }

    /// Creates the instructions responsible for wrapping one object kind to another.
    ///
    /// # Parameters
    ///
    /// - `conv_called` - The calling convention to convert to `conv_current`. This is the convention of the function (`options.target_address`) called.
    /// - `conv_current` - The target convention to which convert to `conv_called`. This is the convention of the function returned.
    /// - `optimized` - Whether to generate optimized code
    fn two_parameters(
        conv_called: &MockFunctionAttribute,
        conv_current: &MockFunctionAttribute,
        optimized: bool,
    ) -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError> {
        two_parameters_with_address(conv_called, conv_current, optimized, 4096)
    }

    /// Creates the instructions responsible for wrapping one object kind to another.
    ///
    /// # Parameters
    ///
    /// - `conv_called` - The calling convention to convert to `conv_current`. This is the convention of the function (`options.target_address`) called.
    /// - `conv_current` - The target convention to which convert to `conv_called`. This is the convention of the function returned.
    /// - `optimized` - Whether to generate optimized code
    /// - `target_address` - Address to jump to.
    fn two_parameters_with_address(
        conv_called: &MockFunctionAttribute,
        conv_current: &MockFunctionAttribute,
        optimized: bool,
        target_address: usize,
    ) -> Result<Vec<Operation<MockRegister>>, WrapperGenerationError> {
        // Two parameters
        let mock_function = MockFunction {
            parameters: vec![ParameterType::nint, ParameterType::nint],
        };

        let capabiltiies = get_x86_jit_capabilities();
        let options = get_common_options(
            optimized,
            target_address,
            target_address < 0x7FFFFFFF,
            &mock_function,
            capabiltiies,
        );
        generate_wrapper_instructions(conv_called, conv_current, &options)
    }

    fn get_common_options(
        optimized: bool,
        target_address: usize,
        can_generate_relative: bool,
        mock_function: &MockFunction,
        capabilties: JitCapabilities,
    ) -> WrapperInstructionGeneratorOptions<MockFunction> {
        WrapperInstructionGeneratorOptions {
            stack_entry_alignment: size_of::<isize>(), // no_alignment
            target_address,                            // some arbitrary address
            standard_register_size: size_of::<isize>(),
            function_info: mock_function,
            injected_parameter: None,
            jit_capabilities: capabilties,
            can_generate_relative_jumps: can_generate_relative,
            enable_optimizations: optimized,
        }
    }

    fn assert_push_stack(op: &Operation<MockRegister>, offset: isize, item_size: isize) {
        if let Operation::PushStack(x) = op {
            assert!(x.has_offset_and_size(offset as i32, item_size as u32));
        }
    }
}
