extern crate alloc;
use core::{hash::Hash, mem::size_of};

use alloc::vec::Vec;

use super::{
    function_attribute::FunctionAttribute,
    function_info::FunctionInfo,
    jit::{
        compiler::{Jit, JitCapabilities},
        operation::Operation,
    },
    traits::register_info::RegisterInfo,
};
use crate::{
    api::jit::operation_aliases::*,
    optimize::{
        combine_push_operations::{merge_pop_operations, merge_push_operations},
        eliminate_common_callee_saved_registers::eliminate_common_callee_saved_registers,
        optimize_reg_parameters::optimize_push_pop_parameters,
        optimize_stack_parameters::optimize_stack_parameters,
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
    pub injected_paramter: Option<usize>,
}

/// Creates the instructions responsible for wrapping one object kind to another.
///
/// # Parameters
///
/// - `fromConvention` - The calling convention to convert to `toConvention`. This is the convention of the function (`options.target_address`) called.
/// - `toConvention` - The target convention to which convert to `fromConvention`. This is the convention of the function returned.
/// - `options` - The parameters for this wrapper generation task.
///
/// # Remarks
///
/// This process is documented in the Wiki under `Design Docs -> Wrapper Generation`.
#[allow(warnings)]
pub fn generate_wrapper_instructions<
    TRegister: RegisterInfo + Clone + Hash + Eq + Copy + Default,
    TFunctionAttribute: FunctionAttribute<TRegister>,
    TJit: Jit<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TFunctionAttribute,
    to_convention: TFunctionAttribute,
    options: WrapperInstructionGeneratorOptions<TFunctionInfo>,
) -> Vec<Operation<TRegister>> {
    let mut ops = Vec::<Operation<TRegister>>::new();
    let mut stack_pointer =
        options.stack_entry_alignment + from_convention.reserved_stack_space() as usize;

    // Backup Always Saved Registers (LR)
    for register in to_convention.always_saved_registers() {
        ops.push(Push::new(register.clone()).into());
        stack_pointer += register.size_in_bytes();
    }

    // Backup callee saved registers
    let callee_saved_regs = eliminate_common_callee_saved_registers(
        from_convention.callee_saved_registers(),
        to_convention.callee_saved_registers(),
    );

    for register in callee_saved_regs {
        ops.push(Push::new(register.clone()).into());
        stack_pointer += register.size_in_bytes();
    }

    // Insert Dummy for Stack Alignment
    let align_stack_idx = ops.len();
    ops.push(StackAlloc::new(0).into()); // insert a dummy for now.

    // Re-push stack parameters of function returned (right to left)
    let mut setup_params_ops = Vec::<Operation<TRegister>>::new();
    let fn_returned_params = options.function_info.get_parameters(&to_convention);
    let mut base_pointer = stack_pointer as usize;

    /*
        Context [x64 as example].

        At the current moment in time, the variable before the return address is at -stack_pointer.

        On platforms like ARM that don't do stack returns, this is natural, but on platforms like
        x64 where return is done via address on stack, `options.stack_entry_alignment` offsets this
        such that -stack_pointer is guaranteed to points to the base of the last stack parameter.

        From there, we can re push registers, just have to be careful to keep track of SP, which is
        raising as we push more.
    */

    for param in fn_returned_params.0.iter().rev() {
        setup_params_ops.push(PushStack::new(stack_pointer as isize, param.size_in_bytes()).into());
        base_pointer -= (param.size_in_bytes() * 2); // since we are relative to SP
        stack_pointer += param.size_in_bytes();
    }

    // Push register parameters of function returned (right to left)
    for param in fn_returned_params.1.iter().rev() {
        setup_params_ops.push(Push::new(param.1.clone()).into());
        stack_pointer += param.0.size_in_bytes();
    }

    // Inject parameter (if applicable)
    if let Some(injected_value) = options.injected_paramter {
        setup_params_ops.push(PushConst::new(injected_value).into());
        stack_pointer += size_of::<usize>();
    }

    // Pop register parameters of the function being called (left to right)
    let fn_called_params = options.function_info.get_parameters(&from_convention);
    for param in fn_called_params.1.iter() {
        setup_params_ops.push(Pop::new(param.1.clone()).into());
        stack_pointer -= param.0.size_in_bytes();
    }

    // Optimize the re-pushing process.
    let mut optimized = optimize_stack_parameters(setup_params_ops.as_mut_slice());
    optimized = optimize_push_pop_parameters(optimized);
    // optimized = reorder_mov_sequence(optimized);
    if options
        .jit_capabilities
        .contains(&JitCapabilities::CanMultiPush)
    {
        optimized = merge_push_operations(optimized);
        optimized = merge_pop_operations(optimized);
    }

    // Push optimised call setup to stack
    ops.extend_from_slice(optimized);

    // Now write the correct stack alignment value
    let stack_misalignment = stack_pointer % from_convention.required_stack_alignment();
    ops[align_stack_idx] = StackAlloc::new(stack_misalignment as i32).into();

    // Reserve required space for function called
    ops.push(StackAlloc::new(from_convention.reserved_stack_space() as i32).into());
    stack_pointer += from_convention.reserved_stack_space() as usize;

    // Call the Method
    if options.can_generate_relative_jumps {
        ops.push(CallRel::new(options.target_address).into());
    } else {
        ops.push(
            CallAbs {
                scratch_register: TRegister::default(),
                target_address: options.target_address,
            }
            .into(),
        );
    }

    // Return Result
    todo!("rest of code");
    ops
}
