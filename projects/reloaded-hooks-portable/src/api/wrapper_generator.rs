extern crate alloc;

use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::api::jit::operation::Operation;

use super::{
    buffers::buffer_abstractions::Buffer, function_attribute::FunctionAttribute,
    function_info::FunctionInfo, jit::compiler::Jit, platforms::platform_functions::BUFFER_FACTORY,
    settings::proximity_target::ProximityTarget, traits::register_info::RegisterInfo,
};

/// Options and additional context necessary for the wrapper generator.
#[derive(Clone, Copy)]
pub struct WrapperGenerationOptions<'a, T, TRegister, TJit>
where
    TRegister: RegisterInfo,
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

    /// Marker to assure Rust that TRegister is logically part of the struct.
    _marker: PhantomData<TRegister>,
}

impl<'a, TFunctionInfo, TRegister, TJit>
    WrapperGenerationOptions<'a, TFunctionInfo, TRegister, TJit>
where
    TRegister: RegisterInfo,
    TFunctionInfo: FunctionInfo,
    TJit: Jit<TRegister>,
{
    fn get_buffer_from_factory(&self) -> (bool, Box<dyn Buffer>) {
        let mut buffer_factory_lock = BUFFER_FACTORY.lock();

        let buf_opt = buffer_factory_lock.get_buffer(
            self.proximity_target.item_size,
            self.proximity_target.target_address,
            self.proximity_target.requested_proximity,
            <TJit as Jit<TRegister>>::code_alignment(),
        );

        let has_buf_in_range = buf_opt.is_ok();
        let buf_boxed: Box<dyn Buffer> = match buf_opt {
            Ok(buffer) => buffer,
            Err(_) => buffer_factory_lock
                .get_any_buffer(
                    self.proximity_target.item_size,
                    <TJit as Jit<TRegister>>::code_alignment(),
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
    TRegister: RegisterInfo + Copy,
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
    0 as *const u8;
    todo!();
}
