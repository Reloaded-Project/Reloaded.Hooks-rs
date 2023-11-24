use super::{assembly_hook::AssemblyHook, assembly_hook_dependencies::AssemblyHookDependencies};
use crate::api::{
    errors::assembly_hook_error::AssemblyHookError, jit::compiler::Jit,
    length_disassembler::LengthDisassembler, platforms::platform_functions::MUTUAL_EXCLUSOR,
    settings::assembly_hook_settings::AssemblyHookSettings, traits::register_info::RegisterInfo,
};

/// Creates an assembly hook at a specified location in memory.
///
/// # Overview
///
/// This function injects a `jmp` instruction into an arbitrary sequence of assembly instructions
/// to redirect execution to custom code.
///
/// # Usage Notes
/// - The hook supports both Position Independent Code (PIC) and Position Relative Code.
/// - For optimal performance and compatibility, relative branching is preferred where possible.
/// - Programmers should specify the maximum permissible hook length. If this constraint is not met,
///   an error is thrown.
///
/// # Error Handling
///
/// Errors are propagated via `Result`.
/// If the hook cannot be created within the constraints specified in `settings`, an error is thrown.
///
/// # Hook Lengths
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
pub fn create_assembly_hook<'a, TJit, TRegister, TDisassembler>(
    settings: &AssemblyHookSettings,
    deps: &AssemblyHookDependencies<'a, TJit, TRegister, TDisassembler>,
) -> Result<AssemblyHook<'a>, AssemblyHookError>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
{
    // Documented in docs/dev/design/assembly-hooks/overview.md

    // Lock native memory
    let _guard = MUTUAL_EXCLUSOR.lock();

    // Assumption: The memory is already readable, i.e. r-x or rwx
    // Get Hook Length
    let hook_length =
        TDisassembler::disassemble_length(settings.hook_address, settings.max_permitted_bytes);

    let a = 5;

    /*

       if hook_length > settings.max_permitted_bytes {
           return Err(AssemblyHookError::TooManyBytes((), settings.max_permitted_bytes);
       }
    */

    todo!();
}
