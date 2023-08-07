extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::structs::operation::Operation;

use super::{
    buffers::buffer_abstractions::Buffer,
    function_attribute::FunctionAttribute,
    function_info::FunctionInfo,
    init::{get_architecture_details, get_platform_mut},
    settings::proximity_target::ProximityTarget,
};

/// Options and additional context necessary for the wrapper generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapperGenerationOptions<'a, T: FunctionInfo> {
    /// Address of the function to be called.
    pub target_address: usize,

    /// Target within which in memory the wrapper should be allocated.
    pub proximity_target: ProximityTarget,

    /// Information about the function for which the wrapper needs to be generated.
    pub function_info: &'a T,

    /// Dynamically compiles the specified sequence of instructions
    pub jit_code: fn(&[Operation<T>]) -> &[u8],
}

/// Creates a wrapper function which allows you to call methods of `fromConvention` using
/// `toConvention`.
///
/// # Parameters
///
/// - `fromConvention` - The calling convention to convert to `toConvention`. This is the convention of the function (`options.target_address`) called.
/// - `toConvention` - The target convention to which convert to `fromConvention`. This is the convention of the function returned.
/// - `options` - The parameters for this wrapper generation task.
#[allow(warnings)]
pub fn generate_wrapper<
    TRegister,
    TFunctionAttribute: FunctionAttribute<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TFunctionAttribute,
    to_convention: TFunctionAttribute,
    options: WrapperGenerationOptions<TFunctionInfo>,
) -> *const u8 {
    // Get the memory for our wrapper.
    let mut platform_lock = get_platform_mut();
    let platform = platform_lock.as_mut().unwrap();

    let mut architecture = get_architecture_details();

    let buffer_factory = &mut *platform.buffer_factory;
    let target = &options.proximity_target;
    let buf_opt = buffer_factory.get_buffer(
        target.item_size,
        target.target_address,
        target.requested_proximity,
        architecture.code_alignment,
    );

    let has_buf_in_range = buf_opt.is_some();
    let buf_boxed: Box<dyn Buffer> = match buf_opt {
        Some(buffer) => buffer,
        None => buffer_factory
            .get_any_buffer(target.item_size, architecture.code_alignment)
            .unwrap(),
    };

    // Start assembling some code.
    let assembly = Vec::<Operation<TRegister>>::new();
    0 as *const u8
}
