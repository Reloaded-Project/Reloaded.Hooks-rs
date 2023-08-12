extern crate alloc;
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

/// Options and additional context necessary for the wrapper generator.
#[derive(Clone, Copy)]
pub struct WrapperInstructionGeneratorOptions<'a, TFunctionInfo>
where
    TFunctionInfo: FunctionInfo,
{
    /// True if the code is within relative jump distance; and JIT
    /// can emit relative jump to target function.
    pub can_generate_relative_jumps: bool,

    /// Returns the functionalities supported by this JIT.
    /// These functionalities affect code generation performed by this library.
    pub jit_capabilities: &'a [JitCapabilities],

    /// Address of the function to be called.
    pub target_address: usize,

    /// Information about the function for which the wrapper needs to be generated.
    pub function_info: &'a TFunctionInfo,
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
    TRegister: RegisterInfo,
    TFunctionAttribute: FunctionAttribute<TRegister>,
    TJit: Jit<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TFunctionAttribute,
    to_convention: TFunctionAttribute,
    options: WrapperInstructionGeneratorOptions<TFunctionInfo>,
) -> *const u8 {
    let assembly = Vec::<Operation<TRegister>>::new();
    0 as *const u8
}
