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
    helpers::make_inline_rel_branch::{make_inline_branch, INLINE_BRANCH_LEN},
    internal::assembly_hook::create_assembly_hook,
};
use alloc::rc::Rc;
use bitfield::bitfield;
use core::marker::PhantomData;

bitfield! {
    /// `AddImmediate` represents the bitfields of the ADD (immediate) instruction
    /// in AArch64 architecture.
    pub struct AssemblyHookLengths(u8);
    impl Debug;

    // Note: Can shorten this to 3 bits if needed.

    /// Length of the 'branch to orig' inline array.
    branch_to_orig_len, set_branch_to_orig_len: 8, 4;

    /// Length of the 'branch to hook' inline array.
    branch_to_hook_len, set_branch_to_hook_len: 4, 0;
}

impl AssemblyHookLengths {
    /// Creates a new `AssemblyHookLengths` with specified lengths.
    pub fn new(branch_to_orig_len: u8, branch_to_hook_len: u8) -> Self {
        let mut lengths = AssemblyHookLengths(0);
        lengths.set_branch_to_orig_len(branch_to_orig_len);
        lengths.set_branch_to_hook_len(branch_to_hook_len);
        lengths
    }
}

/// Represents an assembly hook.
#[repr(C, packed)] // Human packed, to sacrifice some size,
pub struct AssemblyHook<'a, TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
where
    TBuffer: Buffer,
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
{
    // docs/dev/design/assembly-hooks/overview.md

    /*
        The first 4 fields are set up to account for struct packing.
        With the assumption that 5 bytes max is required for any current architecture for a short relative jump.
        Jump is 2/5 bytes for x86 (depending on payload size) and 4 bytes for ARM64.

        Right now 'is_enabled' is left unpadded for performance reasons, as there's free space. Once more
        metadata is needed, I expect all fields to be left unpadded.
    */
    /// True if this hook is currently disabled, else false.
    is_enabled: bool, // 0

    /// Stores sizes of 'branch_to_orig_opcode' and 'branch_to_hook_opcode'.
    lengths: AssemblyHookLengths, // 1

    // 1 (is_enabled) + 1 (lengths) + 5 + 5 == 12 bytes
    /// The code to branch to 'orig' segment in the buffer, when disabling the hook.
    branch_to_orig_opcode: [u8; INLINE_BRANCH_LEN], // 2

    /// The code to branch to 'hook' segment in the buffer, when enabling the hook.#
    branch_to_hook_opcode: [u8; INLINE_BRANCH_LEN], // 7

    /// The code placed at the hook function when the hook is enabled.
    enabled_code: &'a [u8], // 12/16

    /// The code placed at the hook function when the hook is disabled.
    disabled_code: &'a [u8], // 16/24

    /// The address of the hook.
    hook_address: usize, // 20/32

    // Dummy type parameters for Rust compiler to comply.
    _unused_buf: PhantomData<TBuffer>,
    _unused_tj: PhantomData<TJit>,
    _unused_tr: PhantomData<TRegister>,
    _unused_td: PhantomData<TDisassembler>,
    _unused_rwr: PhantomData<TRewriter>,
    _unused_fac: PhantomData<TBufferFactory>,
}

impl<'a, TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
    AssemblyHook<'a, TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>
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
        branch_to_orig: Rc<[u8]>,
        branch_to_hook: Rc<[u8]>,
        enabled_code: &'a [u8],
        disabled_code: &'a [u8],
        hook_address: usize,
    ) -> Result<Self, AssemblyHookError<TRegister>> {
        let branch_to_orig_opcode =
            Self::inline_branch(&branch_to_orig, ArrayTooShortKind::ToOrig)?;
        let branch_to_hook_opcode =
            Self::inline_branch(&branch_to_hook, ArrayTooShortKind::ToHook)?;
        let lengths = AssemblyHookLengths::new(
            branch_to_orig_opcode.len() as u8,
            branch_to_hook_opcode.len() as u8,
        );

        Ok(Self {
            is_enabled,
            lengths,
            branch_to_orig_opcode,
            branch_to_hook_opcode,
            enabled_code,
            disabled_code,
            hook_address,
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
    /// | x86_64         | 5 bytes (+- 2GiB)   | 6 bytes      | 12 bytes        |
    /// | x86_64 (macOS) | 5 bytes (+- 2GiB)   | 12 bytes     | 12 bytes        |
    /// | ARM64          | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
    /// | ARM64 (macOS)  | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
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
        AssemblyHook<'a, TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>,
        AssemblyHookError<TRegister>,
    > {
        return create_assembly_hook::<
            TJit,
            TRegister,
            TDisassembler,
            TRewriter,
            TBuffer,
            TBufferFactory,
        >(settings);
    }

    /// Enables the hook.
    /// This will cause the hook to be written to memory.
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will write the hook to memory.
    pub fn enable(&self) {
        // TODO: Strategy from document.
        TBuffer::overwrite(self.hook_address, self.enabled_code);
    }

    /// Disables the hook.
    /// This will cause the hook to be no-opped.
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will no-op the hook.
    pub fn disable(&self) {
        // TODO: Strategy from document.
        TBuffer::overwrite(self.hook_address, self.disabled_code);
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        self.is_enabled
    }

    fn inline_branch(
        rc: &[u8],
        kind: ArrayTooShortKind,
    ) -> Result<[u8; INLINE_BRANCH_LEN], AssemblyHookError<TRegister>> {
        make_inline_branch(rc).map_err(|e| AssemblyHookError::InlineBranchError(e, kind))
    }
}
