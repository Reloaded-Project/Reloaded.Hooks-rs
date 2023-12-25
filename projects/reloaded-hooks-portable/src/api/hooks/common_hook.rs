extern crate alloc;
use crate::api::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    jit::compiler::Jit,
    traits::register_info::RegisterInfo,
};
use core::marker::PhantomData;
use core::ptr::NonNull;

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
))]
use super::stub::stub_props_4byteins::*;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
use super::stub::stub_props_other::*;

/// Represents an assembly hook.
#[repr(C)] // Not 'packed' because this is not in array and malloc in practice will align this.
pub struct CommonHook<TBuffer, TJit, TRegister, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy,
    TBufferFactory: BufferFactory<TBuffer>,
{
    // docs/dev/design/assembly-hooks/overview.md
    /// The address of the stub containing custom code.
    stub_address: usize, // 0

    /// Address of 'props' structure
    props: NonNull<StubPackedProps>, // 4/8

    // Struct size: 8/16 bytes.

    // Dummy type parameters for Rust compiler to comply.
    _unused_buf: PhantomData<TBuffer>,
    _unused_tj: PhantomData<TJit>,
    _unused_tr: PhantomData<TRegister>,
    _unused_fac: PhantomData<TBufferFactory>,
}

impl<TBuffer, TJit, TRegister, TBufferFactory> CommonHook<TBuffer, TJit, TRegister, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy,
    TBufferFactory: BufferFactory<TBuffer>,
{
    pub fn new(props: NonNull<StubPackedProps>, stub_address: usize) -> Self {
        Self {
            props,
            stub_address,
            _unused_buf: PhantomData,
            _unused_tj: PhantomData,
            _unused_tr: PhantomData,
            _unused_fac: PhantomData,
        }
    }

    /// Enables the hook at `stub_address`.
    ///
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will perform a thread safe enabling of the hook.
    pub fn enable(&self) {
        unsafe {
            (*self.props.as_ptr())
                .enable::<TRegister, TJit, TBufferFactory, TBuffer>(self.stub_address);
        }
    }

    /// Disables the hook at `stub_address`.
    ///
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will perform a thread safe disabling of the hook.
    pub fn disable(&self) {
        unsafe {
            (*self.props.as_ptr())
                .disable::<TRegister, TJit, TBufferFactory, TBuffer>(self.stub_address);
        }
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        unsafe { self.props.as_ref().is_enabled() }
    }
}

impl<TBuffer, TJit, TRegister, TBufferFactory> Drop
    for CommonHook<TBuffer, TJit, TRegister, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy,
    TBufferFactory: BufferFactory<TBuffer>,
{
    fn drop(&mut self) {
        unsafe {
            self.props.as_mut().free();
        }
    }
}
