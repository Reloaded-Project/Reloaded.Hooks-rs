use super::{assembly_hook::AssemblyHook, assembly_hook_dependencies::AssemblyHookDependencies};
use crate::api::{
    errors::assembly_hook_error::AssemblyHookError,
    jit::compiler::Jit,
    length_disassembler::LengthDisassembler,
    platforms::platform_functions::MUTUAL_EXCLUSOR,
    rewriter::code_rewriter::CodeRewriter,
    settings::assembly_hook_settings::{AsmHookBehaviour, AssemblyHookSettings},
    traits::register_info::RegisterInfo,
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
pub fn create_assembly_hook<'a, TJit, TRegister, TDisassembler, TRewriter>(
    settings: &AssemblyHookSettings,
    deps: &AssemblyHookDependencies<'a, TJit, TRegister, TDisassembler, TRewriter>,
) -> Result<AssemblyHook<'a>, AssemblyHookError>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    // Documented in docs/dev/design/assembly-hooks/overview.md

    // Assumption: The memory at hook point is already readable, i.e. r-x or rwx

    // Length of original code.
    let orig_code_length = if settings.behaviour == AsmHookBehaviour::DoNotExecuteOriginal {
        0
    } else {
        settings.max_permitted_bytes
    };

    let stub_hook_length = settings.asm_code.len() + TJit::max_branch_bytes() as usize;

    // Get Hook Length

    // Lock native function memory, to ensure we get accurate info.
    let _guard = MUTUAL_EXCLUSOR.lock();

    let hook_length =
        TDisassembler::disassemble_length(settings.hook_address, settings.max_permitted_bytes);

    /*

       if hook_length > settings.max_permitted_bytes {
           return Err(AssemblyHookError::TooManyBytes((), settings.max_permitted_bytes);
       }
    */

    todo!();
}
