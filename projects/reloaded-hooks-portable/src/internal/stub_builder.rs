extern crate alloc;
use super::stub_builder_settings::{HookBuilderSettings, HookBuilderSettingsMixin};
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::hook_builder_error::{HookBuilderError, RewriteErrorDetails, RewriteErrorSource},
        hooks::stub::stub_props_common::*,
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        rewriter::code_rewriter::{CodeRewriter, CodeRewriterError},
        traits::register_info::RegisterInfo,
    },
    helpers::{
        allocate_with_proximity::allocate_with_proximity,
        atomic_write_masked::MAX_ATOMIC_WRITE_BYTES,
    },
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{
    cmp::max,
    mem::size_of,
    ops::{Add, Sub},
    ptr::NonNull,
    slice::from_raw_parts,
};
use derive_new::new;

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
))]
use crate::api::hooks::stub::stub_props_4byteins::*;
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
use crate::api::hooks::stub::stub_props_other::*;

/// Creates the buffer required to call the [`create_hook_stub`] function.
/// This is necessary in order to allow for various hooks to do a 'fast exit' in
/// the event that it iis not possible to install the hook.
#[allow(clippy::type_complexity)]
pub unsafe fn create_hook_stub_buffer<
    TJit,
    TRegister: Clone + Default,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    source_address: usize,
    max_buf_length: usize,
) -> HookBuilderStubAllocation<TBuffer>
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
{
    let alloc_result = allocate_with_proximity::<TJit, TRegister, TBufferFactory, TBuffer>(
        source_address,
        max_buf_length as u32,
    );

    HookBuilderStubAllocation::<TBuffer>::new(alloc_result.0, alloc_result.1)
}

/// Shared code used to create both the 'stub' and the 'heap' used throughout the various hooks in
/// `reloaded-hooks`.
///
/// # Overview
///
/// This function performs the following:
///
/// - Creates the `Stub`.  
/// - Creates the 'Heap' data.  
///
/// For more details see the documentation page:
/// docs/dev/design/common.md#hook-memory-layouts-thread-safety
///
/// This function only creates the stub and heap.
/// It does not apply the hook itself to the original code, as that is implementation specific.
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
#[allow(clippy::type_complexity)]
pub unsafe fn create_stub<TRegister: Clone + Default, TBuffer: Buffer>(
    settings: &mut HookBuilderSettings,
    alloc: &mut HookBuilderStubAllocation<TBuffer>,
    mixin: &mut dyn HookBuilderSettingsMixin<TRegister>,
) -> Result<HookBuilderResult, HookBuilderError<TRegister>>
where
    TRegister: RegisterInfo,
{
    // Assumption: The memory at hook point is already readable, i.e. r-x or rwx

    // Lock native function memory, to ensure we get accurate info.
    // This should make hooking operation thread safe provided no presence of 3rd party
    // library instances, which is a-ok for Reloaded3.

    // The required length of buffer for custom/stub code.
    // See: Layout in documentation page.

    // Allocate that buffer, and write our custom code to it.
    let buf = &mut alloc.buf;
    let buf_addr = buf.get_address() as usize;

    // Preallocate buffers for props, and code
    let swap_length = settings.max_swap_length;
    let mut props_buf = Vec::<u8>::with_capacity(swap_length + size_of::<StubPackedProps>());

    // Reserve space for StubPackedProps, and get a pointer to it.
    #[allow(clippy::uninit_vec)]
    props_buf.set_len(size_of::<StubPackedProps>());
    let props: &mut StubPackedProps =
        unsafe { &mut *(props_buf.as_mut_ptr() as *mut StubPackedProps) };

    let mut code_buf_1 = Vec::<u8>::with_capacity(swap_length);
    let mut code_buf_2 = Vec::<u8>::with_capacity(swap_length);

    // Stub Mem Layout (See Docs)
    // - entry: [Hook Function / Original Code]
    // - hook: Hook Function
    // - orig: Original Code

    // 'Original Code' @ entry
    mixin.get_orig_function(buf_addr, &mut code_buf_1)?;

    // 'Hook Function' @ entry
    mixin.get_hook_function(buf_addr, &mut code_buf_2)?;

    // Write the default code.
    let enabled_len = code_buf_2.len();
    let disabled_len = code_buf_1.len();
    let swap_space_len = max(enabled_len, disabled_len);

    let enabled_code = from_raw_parts(code_buf_2.as_ptr(), enabled_len);
    let disabled_code = from_raw_parts(code_buf_1.as_ptr(), disabled_len);

    let old_len = props_buf.len();
    if settings.auto_activate {
        TBuffer::overwrite(buf_addr, enabled_code);
        props_buf.extend_from_slice(disabled_code);
    } else {
        TBuffer::overwrite(buf_addr, disabled_code);
        props_buf.extend_from_slice(enabled_code);
    }

    props.set_is_enabled(settings.auto_activate);

    // Small payload! We can atomic write over the whole thing.
    // BUT it MUST be aligned, because some architectures require that
    // writes be aligned to ensure atomicity. For example MOVDQU on x86 is not
    // atomic unless aligned.
    let padded_len = (swap_space_len).next_power_of_two();
    if swap_space_len <= MAX_ATOMIC_WRITE_BYTES as usize && (buf_addr % padded_len) == 0 {
        // We could technically re-encode here, and handle misaligned code, but it's expensive for x86
        // to re-encode. So we only allow aligned for now.
        props.set_is_swap_only(true);
        props_buf.set_len(old_len + padded_len as usize);
        props.set_swap_size(padded_len as usize);

        // Advance the buffer to account for code written.
        buf.advance(swap_space_len);
    } else {
        props.set_is_swap_only(false);
        props_buf.set_len(old_len + swap_space_len);
        props.set_swap_size(swap_space_len);
        code_buf_1.clear();
        code_buf_2.clear();

        // Write the other 2 stubs.
        let entry_end_ptr = buf_addr + swap_space_len;

        // 'Hook Function' @ hook
        mixin.get_hook_function(entry_end_ptr, &mut code_buf_1)?;

        TBuffer::overwrite(entry_end_ptr, &code_buf_1);
        props.set_hook_fn_size(code_buf_1.len());
        let hook_at_hook_end = entry_end_ptr + code_buf_1.len();
        code_buf_1.clear();

        // 'Original Code' @ orig
        mixin.get_orig_function(hook_at_hook_end, &mut code_buf_1)?;
        TBuffer::overwrite(hook_at_hook_end, &code_buf_1);

        // Advance the buffer to account for code written.
        buf.advance(hook_at_hook_end.add(code_buf_1.len()).sub(buf_addr));
    }

    // Populate remaining fields
    let props = alloc_and_copy_packed_props(&props_buf);
    Ok(HookBuilderResult::new(props, buf_addr))
}

#[derive(Clone, Copy, new)]
pub struct HookBuilderResult {
    pub props: NonNull<StubPackedProps>,
    pub stub: usize,
}

#[derive(Clone, new)]
pub struct HookBuilderStubAllocation<TBuffer: Buffer> {
    pub can_relative_jump: bool,
    pub buf: Box<TBuffer>,
}

/// Retrieves the max possible ASM length for the given code once relocated to the 'Hook Function'.
///
/// Returns
///
/// - `(max_length, length)`: The max possible length of code, and original length of code.
pub fn get_relocated_code_length<TDisassembler, TRewriter, TRegister>(
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

pub fn new_rewrite_error<TRegister>(
    source: RewriteErrorSource,
    old_location: usize,
    new_location: usize,
    e: CodeRewriterError,
) -> HookBuilderError<TRegister> {
    HookBuilderError::RewriteError(
        RewriteErrorDetails::new(source, old_location, new_location),
        e,
    )
}
