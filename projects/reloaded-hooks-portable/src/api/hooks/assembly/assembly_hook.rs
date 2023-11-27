use super::assembly_hook_impl::create_assembly_hook;
use crate::api::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    errors::assembly_hook_error::AssemblyHookError,
    jit::compiler::Jit,
    length_disassembler::LengthDisassembler,
    rewriter::code_rewriter::CodeRewriter,
    settings::assembly_hook_settings::AssemblyHookSettings,
    traits::register_info::RegisterInfo,
};

/// Represents an assembly hook.
pub struct AssemblyHook<'a> {
    /// True if this hook is currently disabled, else false.
    is_enabled: bool,

    /// The code placed at the hook function when the hook is enabled.
    enabled_code: &'a [u8],

    /// The code placed at the hook function when the hook is disabled.
    disabled_code: &'a [u8],

    /// The address of the hook.
    hook_address: usize,
}

impl<'a> AssemblyHook<'a> {
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
    /// | ARM64 (macOS)  | 4 bytes (+- 128MiB) | 8 bytes      | 24 bytes        |
    ///
    /// If you are on Windows/Linux/macOS, expect the relative length to be used basically every time
    /// in practice. However, do feel free to use the worst case length inside settings if you are unsure.
    pub fn new<TJit, TRegister: Clone, TDisassembler, TRewriter, TBufferFactory, TBuffer>(
        settings: &AssemblyHookSettings<TRegister>,
    ) -> Result<AssemblyHook<'a>, AssemblyHookError>
    where
        TJit: Jit<TRegister>,
        TRegister: RegisterInfo,
        TDisassembler: LengthDisassembler,
        TRewriter: CodeRewriter<TRegister>,
        TBufferFactory: BufferFactory<TBuffer>,
        TBuffer: Buffer,
    {
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
    pub fn enable(&self) -> Result<(), AssemblyHookError> {
        todo!();
    }

    /// Disables the hook.
    /// This will cause the hook to be no-opped.
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will no-op the hook.
    pub fn disable(&self) -> Result<(), AssemblyHookError> {
        todo!();
    }

    /// Returns true if the hook is enabled, else false.
    pub fn get_is_enabled(&self) -> bool {
        self.is_enabled
    }
}
