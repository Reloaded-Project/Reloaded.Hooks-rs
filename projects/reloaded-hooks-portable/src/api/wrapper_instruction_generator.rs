extern crate alloc;
use core::hash::Hash;

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
    optimize::eliminate_common_callee_saved_registers::eliminate_common_callee_saved_registers,
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
#[allow(warnings)]
pub fn generate_wrapper_instructions<
    TRegister: RegisterInfo + Clone + Hash + Eq,
    TFunctionAttribute: FunctionAttribute<TRegister>,
    TJit: Jit<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TFunctionAttribute,
    to_convention: TFunctionAttribute,
    options: WrapperInstructionGeneratorOptions<TFunctionInfo>,
) -> Vec<Operation<TRegister>> {
    let mut ops = Vec::<Operation<TRegister>>::new();
    let mut stack_pointer = options.stack_entry_alignment;

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

    // Reserve required space for function called
    ops.push(StackAlloc::new(from_convention.reserved_stack_space() as i32).into());
    stack_pointer += from_convention.reserved_stack_space() as usize;

    // Re-push stack parameters of function returned (right to left)
    let params = options.function_info.get_parameters(&to_convention);
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

    for param in params.0.iter().rev() {
        ops.push(PushStack::new(stack_pointer as isize, param.size_in_bytes()).into());
        base_pointer -= (param.size_in_bytes() * 2); // since we are relative to SP
    }

    // Push register parameters of function returned (right to left)

    // Inject parameter (if applicable)
    if let Some(injected_value) = options.injected_paramter {
        ops.push(PushConst::new(injected_value).into());
    }

    // Return Result
    todo!("rest of code");
    ops
}
