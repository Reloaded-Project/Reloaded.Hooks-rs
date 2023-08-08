extern crate alloc;

use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{
    buffers::buffer_abstractions::Buffer,
    function_attribute::FunctionAttribute,
    function_info::FunctionInfo,
    integration::{
        architecture_details::ArchitectureDetails, platform_functions::PlatformFunctions,
    },
    jit::{compiler::Jit, operation::Operation},
    settings::proximity_target::ProximityTarget,
};

/// Options and additional context necessary for the wrapper generator.
#[derive(Clone, Copy)]
pub struct WrapperGenerationOptions<'a, T, TRegister, TJit>
where
    T: FunctionInfo,
    TJit: Jit<TRegister>,
{
    /// Address of the function to be called.
    pub target_address: usize,

    /// Target within which in memory the wrapper should be allocated.
    pub proximity_target: ProximityTarget,

    /// Information about the function for which the wrapper needs to be generated.
    pub function_info: &'a T,

    /// Dynamically compiles the specified sequence of instructions
    pub jit: TJit,

    /// The details of the architecture in question
    pub architecture_details: &'a ArchitectureDetails,

    /// The platform_functions.
    pub platform_functions: &'a PlatformFunctions,

    /// Marker to assure Rust that TRegister is logically part of the struct.
    _marker: PhantomData<TRegister>,
}

impl<'a, TFunctionInfo, TRegister, TJit>
    WrapperGenerationOptions<'a, TFunctionInfo, TRegister, TJit>
where
    TFunctionInfo: FunctionInfo,
    TJit: Jit<TRegister>,
{
    fn get_buffer_from_factory(&self) -> (bool, Box<dyn Buffer>) {
        let platform_functions = self.platform_functions;
        let mut platform_lock = platform_functions.buffer_factory.write();

        let buffer_factory = platform_lock.as_mut();
        let buf_opt = buffer_factory.get_buffer(
            self.proximity_target.item_size,
            self.proximity_target.target_address,
            self.proximity_target.requested_proximity,
            self.architecture_details.code_alignment,
        );

        let has_buf_in_range = buf_opt.is_some();
        let buf_boxed: Box<dyn Buffer> = match buf_opt {
            Some(buffer) => buffer,
            None => buffer_factory
                .get_any_buffer(
                    self.proximity_target.item_size,
                    self.architecture_details.code_alignment,
                )
                .unwrap(),
        };

        (has_buf_in_range, buf_boxed)
    }
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
    TJit: Jit<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TFunctionAttribute,
    to_convention: TFunctionAttribute,
    options: WrapperGenerationOptions<TFunctionInfo, TRegister, TJit>,
) -> *const u8 {
    let (has_buf_in_range, buf_boxed) = options.get_buffer_from_factory();

    // Start assembling some code.
    let assembly = Vec::<Operation<TRegister>>::new();
    0 as *const u8
}