use core::cmp::max;

use super::assembly_hook::AssemblyHook;
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
) -> Result<AssemblyHook<'a>, AssemblyHookError>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    // Documented in docs/dev/design/assembly-hooks/overview.md

    // Assumption: The memory at hook point is already readable, i.e. r-x or rwx
    // Lock native function memory, to ensure we get accurate info.
    let _guard = MUTUAL_EXCLUSOR.lock();

    let orig_code_length = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.hook_address,
        settings.max_permitted_bytes,
    );

    // Max possible lengths of custom (hook) code and original code when emplaced onto 'Hook Function' address.
    let hook_code_length = get_hookfunction_hook_length::<TDisassembler, TRewriter, TRegister, TJit>(
        settings,
        orig_code_length,
    );
    let hook_orig_length = orig_code_length + TJit::max_branch_bytes() as usize;

    // The requires length of buffer for our custom code.
    let required_buf_length = max(hook_code_length, hook_orig_length);

    // Get Hook Length
    let hook_length =
        TDisassembler::disassemble_length(settings.hook_address, settings.max_permitted_bytes);

    /*

       if hook_length > settings.max_permitted_bytes {
           return Err(AssemblyHookError::TooManyBytes((), settings.max_permitted_bytes);
       }
    */

    todo!();
}

/// Retrieves the max possible ASM length for the hook code (i.e. 'hook enabled')
/// once emplaced in the 'Hook Function'.
///
/// 'Hook Function': See diagram in docs/dev/design/assembly-hooks/overview.md
///
/// # Parameters
/// - `settings`: The settings for the assembly hook.
/// - `max_orig_code_length`: The maximum possible length of the original code.
fn get_hookfunction_hook_length<TDisassembler, TRewriter, TRegister, TJit>(
    settings: &AssemblyHookSettings<'_>,
    max_orig_code_length: usize,
) -> usize
where
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TJit: Jit<TRegister>,
{
    // New code length + extra max possible length + jmp back to original code
    let hook_code_length = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.asm_code.as_ptr() as usize,
        settings.asm_code.len(),
    );

    let result = hook_code_length + TJit::max_branch_bytes() as usize;
    if settings.behaviour == AsmHookBehaviour::DoNotExecuteOriginal {
        result
    } else {
        result + max_orig_code_length
    }
}

/// Retrieves the max possible ASM length for the given code once relocated to the 'Hook Function'.
fn get_relocated_code_length<TDisassembler, TRewriter, TRegister>(
    code_address: usize,
    min_length: usize,
) -> usize
where
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    let len = TDisassembler::disassemble_length(code_address, min_length);
    let extra_length = len.1 * TRewriter::max_ins_size_increase();
    len.0 + extra_length
}
