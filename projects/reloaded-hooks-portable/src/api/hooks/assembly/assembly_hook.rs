extern crate alloc;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::assembly_hook_error::{ArrayTooShortKind, AssemblyHookError},
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        rewriter::code_rewriter::CodeRewriter,
        settings::assembly_hook_settings::AssemblyHookSettings,
        traits::register_info::RegisterInfo,
    },
    helpers::{
        atomic_write_masked::atomic_write_masked,
        make_inline_rel_branch::{make_inline_branch, INLINE_BRANCH_LEN},
    },
    internal::assembly_hook::create_assembly_hook,
};
use alloc::boxed::Box;
use bitfield::bitfield;
use core::marker::PhantomData;

bitfield! {
    /// `AddImmediate` represents the bitfields of the ADD (immediate) instruction
    /// in AArch64 architecture.
    pub struct AssemblyHookPackedProps(u8);
    impl Debug;

    /// Length of the 'branch to orig' inline array.
    branch_to_orig_len, set_branch_to_orig_len: 6, 4;

    /// Length of the 'branch to hook' inline array.
    branch_to_hook_len, set_branch_to_hook_len: 3, 1;

    /// True if the hook is enabled, else false.
    is_enabled, set_is_enabled: 0;
}

impl AssemblyHookPackedProps {
    /// Creates a new `AssemblyHookPackedProps` with specified properties.
    pub fn new(is_enabled: bool, branch_to_orig_len: u8, branch_to_hook_len: u8) -> Self {
        let mut props = AssemblyHookPackedProps(0);
        props.set_is_enabled(is_enabled);
        props.set_branch_to_orig_len(branch_to_orig_len);
        props.set_branch_to_hook_len(branch_to_hook_len);
        props
    }
}

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
    /// The code placed at the hook function when the hook is enabled.
    enabled_code: Box<[u8]>, // 0

    /// The code placed at the hook function when the hook is disabled.
    disabled_code: Box<[u8]>, // 4/8

    /// The address of the stub containing custom code.
    stub_address: usize, // 8/16

    /// Stores sizes of 'branch_to_orig_opcode' and 'branch_to_hook_opcode'.
    props: AssemblyHookPackedProps, // 12/24

    /// The code to branch to 'orig' segment in the buffer, when disabling the hook.
    branch_to_orig_opcode: [u8; INLINE_BRANCH_LEN], // 13/25

    /// The code to branch to 'hook' segment in the buffer, when enabling the hook.
    branch_to_hook_opcode: [u8; INLINE_BRANCH_LEN], // 18/30 (-1 on AArch64)

    // End: 40 (AArch64) [no pad: 33]
    // End: 24/40 (x86)  [no pad: 23/35]

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
        is_enabled: bool,
        branch_to_orig: Box<[u8]>,
        branch_to_hook: Box<[u8]>,
        enabled_code: Box<[u8]>,
        disabled_code: Box<[u8]>,
        stub_address: usize,
    ) -> Result<Self, AssemblyHookError<TRegister>> {
        let branch_to_orig_opcode =
            Self::inline_branch(&branch_to_orig, ArrayTooShortKind::ToOrig)?;
        let branch_to_hook_opcode =
            Self::inline_branch(&branch_to_hook, ArrayTooShortKind::ToHook)?;
        let props = AssemblyHookPackedProps::new(
            is_enabled,
            branch_to_orig_opcode.len() as u8,
            branch_to_hook_opcode.len() as u8,
        );

        Ok(Self {
            props,
            branch_to_orig_opcode,
            branch_to_hook_opcode,
            enabled_code,
            disabled_code,
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
    fn write_hook(&self, branch_opcode: &[u8], code: &[u8], num_bytes: usize) {
        // Write the branch first, as per docs
        TBuffer::overwrite(self.stub_address, branch_opcode);

        // Now write the remaining code
        TBuffer::overwrite(self.stub_address + num_bytes, &code[num_bytes..]);

        // Now write the non-branch code
        atomic_write_masked::<TBuffer>(self.stub_address, &code[..num_bytes], num_bytes);
    }

    /// Enables the hook.
    /// This will cause the hook to be written to memory.
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will write the hook to memory.
    pub fn enable(&self) {
        let num_bytes = self.props.branch_to_hook_len() as usize;
        self.write_hook(&self.branch_to_hook_opcode, &self.enabled_code, num_bytes);
    }

    /// Disables the hook.
    /// This will cause the hook to be no-opped.
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will no-op the hook.
    pub fn disable(&self) {
        let num_bytes = self.props.branch_to_orig_len() as usize;
        self.write_hook(&self.branch_to_orig_opcode, &self.disabled_code, num_bytes);
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        self.props.is_enabled()
    }

    fn inline_branch(
        rc: &[u8],
        kind: ArrayTooShortKind,
    ) -> Result<[u8; INLINE_BRANCH_LEN], AssemblyHookError<TRegister>> {
        make_inline_branch(rc).map_err(|e| AssemblyHookError::InlineBranchError(e, kind))
    }
}
