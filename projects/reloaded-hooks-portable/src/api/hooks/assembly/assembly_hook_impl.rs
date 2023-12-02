use super::assembly_hook::AssemblyHook;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::assembly_hook_error::{
            AssemblyHookError, RewriteErrorDetails,
            RewriteErrorSource::{self, *},
        },
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        platforms::platform_functions::MUTUAL_EXCLUSOR,
        rewriter::code_rewriter::{CodeRewriter, CodeRewriterError},
        settings::assembly_hook_settings::{AsmHookBehaviour, AssemblyHookSettings},
        traits::register_info::RegisterInfo,
    },
    helpers::allocate_with_proximity::allocate_with_proximity,
};
use core::{
    cmp::max,
    slice::{self},
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
pub fn create_assembly_hook<
    'a,
    TJit,
    TRegister: Clone,
    TDisassembler,
    TRewriter,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &AssemblyHookSettings<TRegister>,
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
    // This should make hooking operation thread safe provided no presence of 3rd party
    // library instances, which is a-ok for Reloaded3.

    let _guard = MUTUAL_EXCLUSOR.lock();

    // Length of the originanl code at the insertion address
    let orig_code_length = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.hook_address,
        settings.max_permitted_bytes,
    );

    if orig_code_length > settings.max_permitted_bytes {
        return Err(AssemblyHookError::TooManyBytes(
            orig_code_length,
            settings.max_permitted_bytes,
        ));
    }

    // When emplaced onto 'Hook Function' address.
    // Max possible lengths of custom (hook) code and original code
    let hook_code_length = get_hookfunction_hook_length::<TDisassembler, TRewriter, TRegister, TJit>(
        settings,
        orig_code_length,
    );
    let hook_orig_length = orig_code_length + TJit::max_branch_bytes() as usize;

    // The requires length of buffer for our custom code.
    let max_possible_buf_length = max(hook_code_length, hook_orig_length);

    // Allocate that buffer, and write our custom code to it.
    let mut buf = allocate_with_proximity::<TJit, TRegister, TBufferFactory, TBuffer>(
        settings.hook_address,
        max_possible_buf_length as u32,
    )
    .1;

    // Rewrite that code to new buffer address.
    let buf_addr = buf.get_address() as usize;
    let new_orig_code = TRewriter::rewrite_code(
        settings.hook_address as *const u8,
        orig_code_length,
        settings.hook_address,
        buf_addr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(OriginalCode, settings.hook_address, buf_addr, e))?;

    let new_hook_code = TRewriter::rewrite_code(
        settings.asm_code.as_ptr(),
        settings.asm_code.len(),
        settings.asm_code_address,
        buf_addr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(CustomCode, settings.asm_code_address, buf_addr, e))?;

    // Reserve the code needed
    let max_buf_length = max(new_hook_code.len(), new_orig_code.len());
    buf.advance(max_buf_length);

    // Write the default code.
    let code_to_write = unsafe {
        if settings.auto_activate {
            slice::from_raw_parts(new_hook_code.as_ptr(), new_hook_code.len())
        } else {
            slice::from_raw_parts(new_orig_code.as_ptr(), new_orig_code.len())
        }
    };

    TBuffer::overwrite(buf_addr, code_to_write);

    todo!();
}

fn new_rewrite_error(
    source: RewriteErrorSource,
    old_location: usize,
    new_location: usize,
    e: CodeRewriterError,
) -> AssemblyHookError {
    AssemblyHookError::RewriteError(
        RewriteErrorDetails::new(source, old_location, new_location),
        e,
    )
}

/// Retrieves the max possible ASM length for the hook code (i.e. 'hook enabled')
/// once emplaced in the 'Hook Function'.
///
/// 'Hook Function': See diagram in docs/dev/design/assembly-hooks/overview.md
///
/// # Parameters
/// - `settings`: The settings for the assembly hook.
/// - `max_orig_code_length`: The maximum possible length of the original code.
fn get_hookfunction_hook_length<TDisassembler, TRewriter, TRegister: Clone, TJit>(
    settings: &AssemblyHookSettings<TRegister>,
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
