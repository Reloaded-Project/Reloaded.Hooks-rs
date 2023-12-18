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
use alloc::vec::Vec;
use alloca::with_alloca;
use core::{
    cmp::max,
    mem::{transmute, MaybeUninit},
    ops::{Add, Sub},
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
    TJit,
    TRegister: Clone + Default,
    TDisassembler,
    TRewriter,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &AssemblyHookSettings<TRegister>,
) -> Result<
    AssemblyHook<TBuffer, TJit, TRegister, TDisassembler, TRewriter, TBufferFactory>,
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

    let jump_back_address = settings.hook_address + orig_code_length;

    let hook_params = HookFunctionCommonParams {
        behaviour: settings.behaviour,
        asm_code: settings.asm_code,
        asm_code_length: settings.asm_code.len(),
        jump_back_address,
        settings,
        can_relative_jump: alloc_result.0,
        vector_capacity: max_possible_buf_length as u32,
        orig_code_length,
    };

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
    let new_hook_code =
        construct_hook_function::<TJit, TRegister, TRewriter, TBuffer, TBufferFactory>(
            &hook_params,
            buf_addr,
        )?;

    // Write the default code.
    let enabled_code = new_hook_code.into_boxed_slice();
    let disabled_code = new_orig_code.into_boxed_slice();

    TBuffer::overwrite(
        buf_addr,
        if settings.auto_activate {
            &enabled_code
        } else {
            &disabled_code
        },
    );

    // Write the other 2 stubs.
    let entry_end_ptr = buf_addr + max(enabled_code.len(), disabled_code.len());

    // 'Hook Function' @ hook
    let hook_at_hook =
        construct_hook_function::<TJit, TRegister, TRewriter, TBuffer, TBufferFactory>(
            &hook_params,
            entry_end_ptr,
        )?;

    TBuffer::overwrite(entry_end_ptr, &hook_at_hook);

    let hook_at_hook_end = entry_end_ptr + hook_at_hook.len();

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
    TBuffer::overwrite(hook_at_hook_end, &orig_at_orig);

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

    // Now JIT a jump to the original code.
    overwrite_code(settings.hook_address, &code);

    // Now be a good citizen and add nops to the end of our jump.
    // This will ensure we don't leave invalid instructions.
    let num_nops = orig_code_length - code.len();
    if num_nops > 0 {
        with_alloca(
            orig_code_length - code.len(),
            |nops: &mut [MaybeUninit<u8>]| {
                let slice = unsafe { transmute::<&mut [MaybeUninit<u8>], &mut [u8]>(nops) };
                TJit::fill_nops(slice);
                overwrite_code(settings.hook_address + code.len(), slice);
            },
        );
    }

    // Advance the buffer to account for code written.
    buf.advance(hook_at_hook_end.add(orig_at_orig.len()).sub(buf_addr));

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

fn construct_hook_function<TJit, TRegister, TRewriter, TBuffer, TBufferFactory>(
    params: &HookFunctionCommonParams<TRegister>,
    buf_addr: usize, // This parameter changes between calls
) -> Result<Vec<u8>, AssemblyHookError<TRegister>>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
{
    unsafe {
        let mut code = Vec::with_capacity(params.vector_capacity as usize);

        // Depending on the behaviour, include the original code before or after
        // hook is 'second'
        if params.behaviour == AsmHookBehaviour::ExecuteAfter {
            // Include original code first
            let original_code = TRewriter::rewrite_code(
                params.settings.hook_address as *const u8,
                params.orig_code_length,
                params.settings.hook_address,
                buf_addr,
                params.settings.scratch_register.clone(),
            )
            .map_err(|e| {
                new_rewrite_error(OriginalCode, params.settings.hook_address, buf_addr, e)
            })?;

            code.extend_from_slice(&original_code);
        }

        // Include hook code
        let hook_code = TRewriter::rewrite_code(
            params.asm_code.as_ptr(),
            params.asm_code.len(),
            params.asm_code_length,
            buf_addr + code.len(),
            params.settings.scratch_register.clone(),
        )
        .map_err(|e| new_rewrite_error(CustomCode, params.asm_code_length, buf_addr, e))?;
        code.extend_from_slice(&hook_code);

        // Include original code after if required
        // hook is 'first'
        if params.behaviour == AsmHookBehaviour::ExecuteFirst {
            let original_code = TRewriter::rewrite_code(
                params.settings.hook_address as *const u8,
                params.orig_code_length,
                params.settings.hook_address,
                buf_addr + code.len(),
                params.settings.scratch_register.clone(),
            )
            .map_err(|e| {
                new_rewrite_error(OriginalCode, params.settings.hook_address, buf_addr, e)
            })?;

            code.extend_from_slice(&original_code);
        }

        // Add jump back to the end of the sequence
        let jmp = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
            buf_addr.wrapping_add(code.len()),
            params.can_relative_jump,
            params.jump_back_address,
            params.settings.scratch_register.clone(),
        )
        .map_err(|e| AssemblyHookError::JitError(e))?;
        code.extend_from_slice(&jmp);

        Ok(code)
    }
}

struct HookFunctionCommonParams<'a, TRegister: Clone> {
    behaviour: AsmHookBehaviour,
    asm_code: &'a [u8],
    asm_code_length: usize,
    jump_back_address: usize,
    settings: &'a AssemblyHookSettings<'a, TRegister>,
    can_relative_jump: bool,
    vector_capacity: u32,
    orig_code_length: usize,
}
