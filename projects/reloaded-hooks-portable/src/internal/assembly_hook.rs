extern crate alloc;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::assembly_hook_error::{
            AssemblyHookError, RewriteErrorDetails,
            RewriteErrorSource::{self, *},
        },
        hooks::assembly::assembly_hook::AssemblyHook,
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        platforms::platform_functions::MUTUAL_EXCLUSOR,
        rewriter::code_rewriter::{CodeRewriter, CodeRewriterError},
        settings::assembly_hook_settings::{AsmHookBehaviour, AssemblyHookSettings},
        traits::register_info::RegisterInfo,
    },
    helpers::{
        allocate_with_proximity::allocate_with_proximity,
        jit_jump_operation::create_jump_operation, overwrite_code::overwrite_code,
    },
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
/// | ARM64          | 4 bytes (+- 128MiB) | 12 bytes     | 20 bytes        |
/// | ARM64 (macOS)  | 4 bytes (+- 128MiB) | 8 bytes      | 20 bytes        |
#[allow(clippy::type_complexity)]
pub unsafe fn create_assembly_hook<
    'a,
    TJit,
    TRegister: Clone + Default,
    TDisassembler,
    TRewriter,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &AssemblyHookSettings<TRegister>,
) -> Result<
    AssemblyHook<'a, TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>,
    AssemblyHookError<TRegister>,
>
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

    // Length of the original code at the insertion address
    let orig_code_lengths = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.hook_address,
        settings.max_permitted_bytes,
    );

    let orig_code_length = orig_code_lengths.1;
    let max_orig_code_length = orig_code_lengths.0;

    // Max possible lengths of custom (hook) code and original code
    // When emplaced onto 'Hook Function' address.
    let hook_code_max_length =
        get_max_hookfunction_hook_length::<TDisassembler, TRewriter, TRegister, TJit>(
            settings,
            max_orig_code_length,
        );
    let hook_orig_max_length = max_orig_code_length + TJit::max_branch_bytes() as usize;

    // The required length of buffer for custom/stub code.
    // See: Layout in documentation page.
    let max_possible_buf_length = max(hook_code_max_length, hook_orig_max_length)
        + hook_code_max_length
        + hook_orig_max_length;

    // Allocate that buffer, and write our custom code to it.
    let alloc_result = allocate_with_proximity::<TJit, TRegister, TBufferFactory, TBuffer>(
        settings.hook_address,
        max_possible_buf_length as u32,
    );
    let mut buf = alloc_result.1;
    let buf_addr = buf.get_address() as usize;

    // Make jump to new buffer
    let code = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        settings.hook_address,
        alloc_result.0,
        buf_addr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;

    // Bail out if the jump to buffer is greater than expected.
    if code.len() > settings.max_permitted_bytes {
        return Err(AssemblyHookError::TooManyBytes(
            code.len(),
            settings.max_permitted_bytes,
        ));
    }

    let jump_back_address = settings.hook_address + code.len();

    // Stub Mem Layout (See Docs)
    // - entry: [Hook Function / Original Code]
    // - hook: Hook Function
    // - orig: Original Code

    // 'Original Code' @ entry
    let mut new_orig_code = TRewriter::rewrite_code(
        settings.hook_address as *const u8,
        orig_code_length,
        settings.hook_address,
        buf_addr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(OriginalCode, settings.hook_address, buf_addr, e))?;
    let jmp = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        buf_addr.wrapping_add(new_orig_code.len()),
        alloc_result.0,
        jump_back_address,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;
    new_orig_code.extend_from_slice(&jmp);

    // 'Hook Function' @ entry
    let mut new_hook_code = TRewriter::rewrite_code(
        settings.asm_code.as_ptr(),
        settings.asm_code.len(),
        settings.asm_code_address,
        buf_addr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(CustomCode, settings.asm_code_address, buf_addr, e))?;
    let jmp = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        buf_addr.wrapping_add(new_hook_code.len()),
        alloc_result.0,
        jump_back_address,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;
    new_hook_code.extend_from_slice(&jmp);

    let entry_end_ptr = buf_addr + max(new_orig_code.len(), new_hook_code.len());

    // 'Hook Function' @ hook
    let mut hook_at_hook = TRewriter::rewrite_code(
        settings.asm_code.as_ptr(),
        settings.asm_code.len(),
        settings.asm_code_address,
        entry_end_ptr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(HookCodeAtHook, settings.asm_code_address, entry_end_ptr, e))?;
    let jmp = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        entry_end_ptr.wrapping_add(hook_at_hook.len()),
        alloc_result.0,
        jump_back_address,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;
    hook_at_hook.extend_from_slice(&jmp);

    let hook_at_hook_end = buf_addr + max(new_orig_code.len(), new_hook_code.len());

    // 'Original Code' @ orig
    let mut orig_at_orig = TRewriter::rewrite_code(
        settings.hook_address as *const u8,
        orig_code_length,
        settings.hook_address,
        hook_at_hook_end,
        settings.scratch_register.clone(),
    )
    .map_err(|e| new_rewrite_error(OrigCodeAtOrig, settings.hook_address, hook_at_hook_end, e))?;
    let jmp = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        hook_at_hook_end.wrapping_add(orig_at_orig.len()),
        alloc_result.0,
        jump_back_address,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;
    orig_at_orig.extend_from_slice(&jmp);

    // Branch `entry -> orig`
    // Branch `entry -> hook`
    let branch_to_hook = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        buf_addr,
        true,
        entry_end_ptr,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;

    let branch_to_orig = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        buf_addr,
        true,
        hook_at_hook_end,
        settings.scratch_register.clone(),
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;

    let hook_end = hook_at_hook_end + orig_at_orig.len();
    buf.advance(hook_end);

    // Write the code for
    // 'Hook Function' @ hook
    // 'Original Code' @ orig
    TBuffer::overwrite(entry_end_ptr, &hook_at_hook);
    TBuffer::overwrite(hook_at_hook_end, &orig_at_orig);

    // Write the default code.
    let enabled_code =
        unsafe { slice::from_raw_parts(new_hook_code.as_ptr(), new_hook_code.len()) };
    let disabled_code =
        unsafe { slice::from_raw_parts(new_orig_code.as_ptr(), new_orig_code.len()) };
    let code_to_write = if settings.auto_activate {
        enabled_code
    } else {
        disabled_code
    };

    TBuffer::overwrite(buf_addr, code_to_write);

    // Now JIT a jump to the original code.
    overwrite_code(settings.hook_address, &code);

    // Populate remaining fields
    AssemblyHook::new(
        settings.auto_activate,
        branch_to_orig,
        branch_to_hook,
        enabled_code,
        disabled_code,
        buf_addr,
    )
}

fn new_rewrite_error<TRegister>(
    source: RewriteErrorSource,
    old_location: usize,
    new_location: usize,
    e: CodeRewriterError,
) -> AssemblyHookError<TRegister> {
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
fn get_max_hookfunction_hook_length<TDisassembler, TRewriter, TRegister: Clone, TJit>(
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
    let hook_code_max_length = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.asm_code.as_ptr() as usize,
        settings.asm_code.len(),
    )
    .0;

    let result = hook_code_max_length + TJit::max_branch_bytes() as usize;
    if settings.behaviour == AsmHookBehaviour::DoNotExecuteOriginal {
        result
    } else {
        result + max_orig_code_length
    }
}

/// Retrieves the max possible ASM length for the given code once relocated to the 'Hook Function'.
///
/// Returns
///
/// - `(max_length, length)`: The max possible length of code, and original length of code.
fn get_relocated_code_length<TDisassembler, TRewriter, TRegister>(
    code_address: usize,
    min_length: usize,
) -> (usize, usize)
where
    TRegister: RegisterInfo,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
{
    let len = TDisassembler::disassemble_length(code_address, min_length);
    let extra_length = len.1 * TRewriter::max_ins_size_increase();
    (len.0 + extra_length, len.0)
}
