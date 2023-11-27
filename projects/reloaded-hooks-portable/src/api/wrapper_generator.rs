extern crate alloc;

use super::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    calling_convention_info::CallingConventionInfo,
    function_info::FunctionInfo,
    jit::compiler::Jit,
    traits::register_info::RegisterInfo,
};
use crate::{api::jit::operation::Operation, helpers::allocate_with_proximity};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Options and additional context necessary for the wrapper generator.
#[derive(Clone, Copy)]
pub struct WrapperGenerationOptions<'a, T, TRegister, TJit, TBufferFactory, TBuffer>
where
    TRegister: RegisterInfo,
    T: FunctionInfo,
    TJit: Jit<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
    TBuffer: Buffer,
{
    /// Address of the function to be called.
    pub target_address: usize,

    /// Information about the function for which the wrapper needs to be generated.
    pub function_info: &'a T,

    // Markers to prevent Rust from coimplaining
    _marker_tj: TJit,
    _marker_tr: PhantomData<TRegister>,
    _marker_tbf: PhantomData<TBufferFactory>,
    _marker_v: PhantomData<TBuffer>,
}

impl<'a, TFunctionInfo, TRegister, TJit, TBufferFactory, TBuffer>
    WrapperGenerationOptions<'a, TFunctionInfo, TRegister, TJit, TBufferFactory, TBuffer>
where
    TRegister: RegisterInfo,
    TFunctionInfo: FunctionInfo,
    TJit: Jit<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
    TBuffer: Buffer,
{
    fn get_buffer_from_factory(&self) -> (bool, Box<TBuffer>) {
        allocate_with_proximity::allocate_with_proximity::<TJit, TRegister, TBufferFactory, TBuffer>(
            self.target_address,
            128,
        )
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
    TBufferFactory: BufferFactory<TBuffer>,
    TBuffer: Buffer,
>(
    from_convention: TConventionInfo,
    to_convention: TConventionInfo,
    options: WrapperGenerationOptions<TFunctionInfo, TRegister, TJit, TBufferFactory, TBuffer>,
) -> *const u8 {
    let (has_buf_in_range, buf_boxed) = options.get_buffer_from_factory();

    // Start assembling some code.
    let assembly = Vec::<Operation<TRegister>>::new();
    0 as *const u8;
    todo!();
}
