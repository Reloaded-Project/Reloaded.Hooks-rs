extern crate alloc;

use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::api::jit::operation::Operation;

use super::{
    buffers::buffer_abstractions::Buffer, calling_convention_info::CallingConventionInfo,
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

        // Try known relative jump ranges.
        for &requested_proximity in TJit::max_relative_jump_distances() {
            let proximity_target = ProximityTarget::with_address_and_requested_proximity(
                self.target_address,
                requested_proximity,
            );

            let buf_opt = buffer_factory_lock.get_buffer(
                proximity_target.item_size,
                proximity_target.target_address,
                proximity_target.requested_proximity,
                <TJit as Jit<TRegister>>::code_alignment(),
            );

            if let Ok(buffer) = buf_opt {
                return (true, buffer);
            }
        }

        let buf_boxed = buffer_factory_lock
            .get_any_buffer(256, <TJit as Jit<TRegister>>::code_alignment())
            .unwrap();
        (false, buf_boxed)
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
    TRegister: RegisterInfo + Copy + PartialEq + 'static,
    TConventionInfo: CallingConventionInfo<TRegister>,
    TJit: Jit<TRegister>,
    TFunctionInfo: FunctionInfo,
>(
    from_convention: TConventionInfo,
    to_convention: TConventionInfo,
    options: WrapperGenerationOptions<TFunctionInfo, TRegister, TJit>,
) -> *const u8 {
    let (has_buf_in_range, buf_boxed) = options.get_buffer_from_factory();

    // Start assembling some code.
    let assembly = Vec::<Operation<TRegister>>::new();
    0 as *const u8;
    todo!();
}
