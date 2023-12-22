extern crate alloc;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::assembly_hook_error::AssemblyHookError,
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        rewriter::code_rewriter::CodeRewriter,
        settings::assembly_hook_settings::AssemblyHookSettings,
        traits::register_info::RegisterInfo,
    },
    helpers::{
        atomic_write_masked::atomic_write_masked, jit_jump_operation::create_jump_operation,
    },
    internal::assembly_hook::create_assembly_hook,
};
use alloc::vec::Vec;
use core::ptr::NonNull;
use core::{marker::PhantomData, slice::from_raw_parts_mut};

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
))]
use super::assembly_hook_props_4byteins::*;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
use super::assembly_hook_props_other::*;

/// Represents an assembly hook.
#[repr(C)] // Not 'packed' because this is not in array and malloc in practice will align this.
pub struct AssemblyHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    // docs/dev/design/assembly-hooks/overview.md
    /// The address of the stub containing custom code.
    stub_address: usize, // 0

    /// Address of 'props' structure
    props: NonNull<AssemblyHookPackedProps>, // 4/8

    // Struct size: 8/16 bytes.

    // Dummy type parameters for Rust compiler to comply.
    _unused_buf: PhantomData<TBuffer>,
    _unused_tj: PhantomData<TJit>,
    _unused_tr: PhantomData<TRegister>,
    _unused_td: PhantomData<TDisassembler>,
    _unused_rwr: PhantomData<TRewriter>,
    _unused_fac: PhantomData<TBufferFactory>,
}

impl<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
    AssemblyHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    pub fn new(
        props: NonNull<AssemblyHookPackedProps>,
        stub_address: usize,
    ) -> Result<Self, AssemblyHookError<TRegister>> {
        Ok(Self {
            props,
            stub_address,
            _unused_buf: PhantomData,
            _unused_tj: PhantomData,
            _unused_tr: PhantomData,
            _unused_td: PhantomData,
            _unused_rwr: PhantomData,
            _unused_fac: PhantomData,
        })
    }

    /// Creates an assembly hook at a specified location in memory.
    ///
    /// # Overview
    ///
    /// This function injects a `jmp` instruction into an arbitrary sequence of assembly instructions
    /// to redirect execution to custom code.
    ///
    /// # Arguments
    /// - `settings`: A reference to `AssemblyHookSettings` containing configuration for the hook, including
    ///   the hook address, the assembly code to be executed, and other parameters.
    ///
    /// # Error Handling
    ///
    /// Errors are propagated via `Result`.
    /// If the hook cannot be created within the constraints specified in `settings`, an error is thrown.
    ///
    /// # Examples
    /// Basic usage involves creating an `AssemblyHookSettings` instance and passing it to this function.
    /// ```compile_fail
    /// use reloaded_hooks_portable::api::hooks::assembly::assembly_hook::AssemblyHook;
    /// use reloaded_hooks_portable::api::settings::assembly_hook_settings::AssemblyHookSettings;
    ///
    /// let settings = AssemblyHookSettings::new_minimal(0x12345678, &[0x90, 0x90], 128);
    /// AssemblyHook::new(&settings, /* AssemblyHookDependencies */);
    /// ```
    ///
    /// # Hook Lengths
    ///
    /// Standard hook lengths for each platform.
    ///
    /// TMA == Targeted Memory Allocation
    ///
    /// | Architecture   | Relative            | TMA          | Worst Case      |
    /// |----------------|---------------------|--------------|-----------------|
    /// | x86            | 5 bytes (+- 2GiB)   | 5 bytes      | 5 bytes         |
    /// | x86_64         | 5 bytes (+- 2GiB)   | 6 bytes      | 13 bytes        |
    /// | x86_64 (macOS) | 5 bytes (+- 2GiB)   | 13 bytes     | 13 bytes        |
    /// | ARM64          | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
    /// | ARM64 (macOS)  | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
    ///
    /// Note: 12/13 bytes worst case on x86 depending on register number used.
    ///
    /// If you are on Windows/Linux/macOS, expect the relative length to be used basically every time
    /// in practice. However, do feel free to use the worst case length inside settings if you are unsure.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it reads from raw memory. Make sure the passed pointers and
    /// lengths are correct.
    #[allow(clippy::type_complexity)]
    pub unsafe fn create(
        settings: &AssemblyHookSettings<TRegister>,
    ) -> Result<
        AssemblyHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>,
        AssemblyHookError<TRegister>,
    > {
        create_assembly_hook::<TJit, TRegister, TDisassembler, TRewriter, TBuffer, TBufferFactory>(
            settings,
        )
    }

    /// Writes the hook to memory, either enabling or disabling it based on the provided parameters.
    unsafe fn swap_hook(&self, temp_branch_offset: usize) {
        let props = self.props.as_ref();

        // Backup current code from swap buffer.
        let swap_buffer_real = props.get_swap_buffer();
        let swap_buffer_copy = swap_buffer_real.to_vec();

        // Copy current code into swap buffer
        let buf_buffer_real =
            from_raw_parts_mut(self.stub_address as *mut u8, props.get_swap_size());
        swap_buffer_real.copy_from_slice(buf_buffer_real);

        // JIT temp branch to hook/orig code.
        let mut vec = Vec::<u8>::with_capacity(8);
        _ = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
            self.stub_address,
            true,
            self.stub_address + temp_branch_offset,
            None,
            &mut vec,
        );
        let branch_opcode = &vec;
        let branch_bytes = branch_opcode.len();

        // Write the temp branch first, as per docs
        // This also overwrites some extra code afterwards, but that's a-ok for now.
        unsafe {
            atomic_write_masked::<TBuffer>(self.stub_address, branch_opcode, branch_bytes);
        }

        // Now write the remaining code
        TBuffer::overwrite(
            self.stub_address + branch_bytes,
            &swap_buffer_copy[branch_bytes..],
        );

        // And now re-insert the code we temp overwrote with the branch
        unsafe {
            atomic_write_masked::<TBuffer>(self.stub_address, &swap_buffer_copy, branch_bytes);
        }
    }

    /// Enables the hook.
    /// This will cause the hook to be written to memory.
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will write the hook to memory.
    pub fn enable(&self) {
        unsafe {
            let props = &mut (*self.props.as_ptr());
            if props.is_enabled() {
                return;
            };

            self.swap_hook(props.get_swap_size());
            props.set_is_enabled(true);
        }
    }

    /// Disables the hook.
    /// This will cause the hook to be no-opped.
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will no-op the hook.
    pub fn disable(&self) {
        unsafe {
            let props = &mut (*self.props.as_ptr());
            if !props.is_enabled() {
                return;
            };

            self.swap_hook(props.get_swap_size() + props.get_hook_fn_size());
            props.set_is_enabled(false);
        }
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        unsafe { self.props.as_ref().is_enabled() }
    }
}

impl<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory> Drop
    for AssemblyHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    fn drop(&mut self) {
        unsafe {
            self.props.as_mut().free();
        }
    }
}
