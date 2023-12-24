extern crate alloc;

use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::fast_hook_error::FastHookError,
        jit::{
            compiler::Jit,
            operation_aliases::{CallRel, JumpAbs, JumpRel},
        },
        length_disassembler::LengthDisassembler,
        platforms::platform_functions::MUTUAL_EXCLUSOR,
        rewriter::code_rewriter::CodeRewriter,
        settings::basic_hook_settings::BasicHookSettings,
        traits::register_info::RegisterInfo,
    },
    helpers::{overwrite_code::overwrite_code, relative_branch_range_check::can_direct_branch},
    internal::hook_builder::create_hook_stub_buffer,
};
use alloc::vec::Vec;
use core::fmt::Debug;

/// Creates a 'fast branch hook'
///
/// # Overview
///
/// Creates a variant of the 'branch hook' which cannot be disabled and cannot support calling
/// convention conversion.
///
/// Use this hook variant if you have no intent to disable the hook and use the same calling convention.
///
/// # Safety
///
/// Wrong hook can of course crash the process :)
///
/// # Returns
///
/// Either address of the old method via `Ok` or an error via `Err`.
#[allow(clippy::type_complexity)]
pub unsafe fn create_branch_hook_with_pointer<
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy + Debug,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &BasicHookSettings<TRegister>,
    original_fn_address: *mut usize,
) -> Result<(), FastHookError<TRegister>> {
    create_branch_hook_with_callback::<
        TJit,
        TRegister,
        TDisassembler,
        TRewriter,
        TBuffer,
        TBufferFactory,
    >(settings, &|val| {
        *original_fn_address = val;
    })
}

/// Creates a 'fast branch hook'
///
/// # Overview
///
/// Creates a variant of the 'branch hook' which cannot be disabled and cannot support calling
/// convention conversion.
///
/// Use this hook variant if you have no intent to disable the hook and use the same calling convention.
///
/// # Safety
///
/// Wrong hook can of course crash the process :)
///
/// # Returns
///
/// Either `Ok` or an error via `Err`.
#[allow(clippy::type_complexity)]
pub unsafe fn create_branch_hook_with_callback<
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy + Debug,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &BasicHookSettings<TRegister>,
    original_val_receiver: impl FnOnce(usize),
) -> Result<(), FastHookError<TRegister>> {
    // Documented in docs/dev/design/assembly-hooks/overview.md
    const MAX_BRANCH_LENGTH: usize = 24; // sufficient for relative/absolute jmp/call in any architecture

    // Lock native function memory, to ensure we get accurate info at hook address.
    // This should make hooking operation thread safe provided no presence of 3rd party
    // library instances, which is a-ok for Reloaded3.
    let _guard = MUTUAL_EXCLUSOR.lock();

    // Decode the existing branch to be modified.
    let target =
        TJit::decode_call_target(settings.hook_address, TJit::standard_relative_call_bytes())?;
    original_val_receiver(target);

    // Determine if we are in range for a direct branch to target.
    // If not, we will need to use a stub.
    // We subtract branch length for those architectures that jump relative to the start of the next instruction.
    let is_direct_branch = can_direct_branch(
        settings.hook_address,
        settings.new_target,
        TJit::max_standard_relative_call_distance(),
        TJit::standard_relative_call_bytes(),
    );

    let mut code = Vec::<u8>::with_capacity(24); // sufficient for relative/absolute jmp in any architecture
    if is_direct_branch {
        // We can branch directly to the target.
        // This is the most optimal solution.
        // No stub needed, and best performance.
        let mut pc = settings.hook_address;
        TJit::encode_call(&CallRel::new(settings.new_target), &mut pc, &mut code).unwrap();

        overwrite_code(settings.hook_address, &code);
        return Ok(());
    }

    // We cannot branch directly to the target. We need to use a stub.

    // Get intermediary buffer we will be using
    let mut alloc = create_hook_stub_buffer::<TJit, TRegister, TBuffer, TBufferFactory>(
        settings.hook_address,
        MAX_BRANCH_LENGTH,
    );

    let buf_ptr = alloc.buf.get_address() as usize;
    let is_direct_branch = can_direct_branch(
        buf_ptr,
        settings.new_target,
        TJit::max_standard_relative_call_distance(),
        TJit::standard_relative_call_bytes(),
    );

    let mut pc = buf_ptr;
    if is_direct_branch {
        TJit::encode_jump(&JumpRel::new(settings.new_target), &mut pc, &mut code)?;

        TBuffer::overwrite(buf_ptr, &code);
        alloc.buf.advance(code.len());
        overwrite_code(settings.hook_address, &code);
        return Ok(());
    }

    let reg = settings.scratch_register.ok_or(FastHookError::StringError(
        "Scratch register is required for create_branch_hook_with_callback",
    ))?;

    TJit::encode_abs_jump(
        &JumpAbs::new_with_reg(settings.new_target, reg),
        &mut pc,
        &mut code,
    )?;

    debug_assert!(code.len() <= MAX_BRANCH_LENGTH);
    TBuffer::overwrite(buf_ptr, &code);
    alloc.buf.advance(code.len());
    overwrite_code(settings.hook_address, &code);

    Ok(())
}
