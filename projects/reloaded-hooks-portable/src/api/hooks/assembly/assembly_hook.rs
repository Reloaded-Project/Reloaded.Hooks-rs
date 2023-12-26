extern crate alloc;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        errors::{
            assembly_hook_error::AssemblyHookError,
            hook_builder_error::{
                HookBuilderError,
                RewriteErrorSource::{CustomCode, OriginalCode},
            },
        },
        hooks::common_hook::CommonHook,
        jit::compiler::Jit,
        length_disassembler::LengthDisassembler,
        platforms::platform_functions::MUTUAL_EXCLUSOR,
        rewriter::code_rewriter::CodeRewriter,
        settings::assembly_hook_settings::{AsmHookBehaviour, AssemblyHookSettings},
        traits::register_info::RegisterInfo,
    },
    helpers::{jit_jump_operation::create_jump_operation, overwrite_code::overwrite_code},
    internal::{
        stub_builder::{
            create_hook_stub_buffer, create_stub, get_relocated_code_length, new_rewrite_error,
        },
        stub_builder_settings::{HookBuilderSettings, HookBuilderSettingsMixin},
    },
};
use alloc::vec::Vec;
use alloca::with_alloca;
use core::{
    cmp::max,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
};
use derive_new::new;

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
/// let code = &[0x90, 0x90];
/// let settings = AssemblyHookSettings::new_minimal(0x12345678, code.as_ptr() as usize, code.len(), 128);
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
/// | x86_64         | 5 bytes (+- 2GiB)   | 6 bytes      | 13 bytes        |
/// | x86_64 (macOS) | 5 bytes (+- 2GiB)   | 13 bytes     | 13 bytes        |
/// | ARM64          | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
/// | ARM64 (macOS)  | 4 bytes (+- 128MiB) | 12 bytes     | 24 bytes        |
///
/// Note: 12/13 bytes worst case on x86 depending on register number used.
///
/// If you are on Windows/Linux/macOS, expect the relative length to be used basically every time
/// in practice. However, do feel free to use the worst case length inside settings if you are unsure.
///
/// # Safety
///
/// This function is unsafe because it reads from raw memory. Make sure the passed pointers and
/// lengths are correct.
#[allow(clippy::type_complexity)]
pub unsafe fn create_assembly_hook<
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo + Clone + Default + Copy,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
>(
    settings: &AssemblyHookSettings<TRegister>,
) -> Result<CommonHook<TBuffer, TJit, TRegister, TBufferFactory>, AssemblyHookError<TRegister>> {
    // Documented in docs/dev/design/assembly-hooks/overview.md

    // Lock native function memory, to ensure we get accurate info at hook address.
    // This should make hooking operation thread safe provided no presence of 3rd party
    // library instances, which is a-ok for Reloaded3.
    let _guard = MUTUAL_EXCLUSOR.lock();

    // Length of the original code to be hooked.
    let orig_code_lengths = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.hook_address,
        settings.max_permitted_bytes,
    );

    let orig_code_length = orig_code_lengths.1;
    let max_orig_code_length = orig_code_lengths.0;

    // Max possible lengths of custom (hook) code and original code
    // When placed inside the stub.
    let stub_orig_max_len = max_orig_code_length + TJit::max_branch_bytes() as usize;
    let stub_hook_max_len = hookfunction_max_len::<TDisassembler, TRewriter, TRegister, TJit>(
        settings,
        max_orig_code_length,
    );

    // Setup the stub builder.
    let max_swap_length = max(stub_hook_max_len, stub_orig_max_len);
    let max_buf_length = max_swap_length + stub_hook_max_len + stub_orig_max_len;

    // Get stub buffer we will be using.
    let mut alloc = create_hook_stub_buffer::<TJit, TRegister, TBuffer, TBufferFactory>(
        settings.hook_address,
        max_buf_length,
    );

    let buf_addr = alloc.buf.get_address() as usize;

    // Make jump to new buffer
    let mut code = Vec::<u8>::with_capacity(orig_code_length);
    create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
        settings.hook_address,
        alloc.can_relative_jump,
        buf_addr,
        settings.scratch_register,
        &mut code,
    )
    .map_err(|e| AssemblyHookError::JitError(e))?;

    // Bail out if the jump to buffer is greater than expected.
    if orig_code_length > settings.max_permitted_bytes {
        return Err(AssemblyHookError::TooManyBytes(
            orig_code_length,
            settings.max_permitted_bytes,
        ));
    }

    let mixin: &mut dyn HookBuilderSettingsMixin<TRegister> =
        &mut AssemblyHookMixin::<TRegister, TJit, TBuffer, TRewriter, TBufferFactory>::new(
            orig_code_length,
            settings.hook_address + orig_code_length,
            alloc.can_relative_jump,
            settings,
        );

    let mut builder_settings = HookBuilderSettings::new(
        settings.hook_address,
        max_swap_length,
        settings.auto_activate,
    );

    let stub = create_stub::<TRegister, TBuffer>(&mut builder_settings, &mut alloc, mixin)?;

    // Write jump to custom code.
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

    Ok(CommonHook::new(stub.props, buf_addr))
}

/// Mixin that provides the 'Assembly Hook' specific functionality for [`HookBuilderSettings`].
#[derive(new)]
pub struct AssemblyHookMixin<
    'a,
    TRegister: Clone + RegisterInfo + Default + Copy,
    TJit: Jit<TRegister>,
    TBuffer: Buffer,
    TRewriter: CodeRewriter<TRegister>,
    TBufferFactory: BufferFactory<TBuffer>,
> {
    /// Size of the original code.
    orig_code_length: usize,

    /// Address of where the generated code should 'jump back' to.
    jump_back_address: usize,

    /// True if the buffer should be able to use relative jumps to 'jump back'.
    can_relative_jump: bool,

    /// Settings of the assembly hook.
    settings: &'a AssemblyHookSettings<TRegister>,

    _reg: PhantomData<TRegister>,
    _rw: PhantomData<TRewriter>,
    _tj: PhantomData<TJit>,
    _tbf: PhantomData<TBufferFactory>,
    _tb: PhantomData<TBuffer>,
}

impl<
        'a,
        TRegister: Clone + RegisterInfo + Default + Copy,
        TBuffer: Buffer,
        TRewriter: CodeRewriter<TRegister>,
        TJit: Jit<TRegister>,
        TBufferFactory: BufferFactory<TBuffer>,
    > HookBuilderSettingsMixin<TRegister>
    for AssemblyHookMixin<'a, TRegister, TJit, TBuffer, TRewriter, TBufferFactory>
{
    fn get_orig_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>> {
        unsafe {
            TRewriter::rewrite_code_with_buffer(
                self.settings.hook_address as *const u8,
                self.orig_code_length,
                self.settings.hook_address,
                address,
                self.settings.scratch_register,
                code,
            )
            .map_err(|e| new_rewrite_error(OriginalCode, self.settings.hook_address, address, e))?;

            create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
                address.wrapping_add(code.len()),
                self.can_relative_jump,
                self.jump_back_address,
                self.settings.scratch_register,
                code,
            )
            .map_err(|e| HookBuilderError::JitError(e))?;
        }

        Ok(())
    }

    fn get_hook_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>> {
        unsafe {
            // Depending on the behaviour, include the original code before or after
            // hook is 'second'
            if self.settings.behaviour == AsmHookBehaviour::ExecuteAfter {
                // Include original code first
                TRewriter::rewrite_code_with_buffer(
                    self.settings.hook_address as *const u8,
                    self.orig_code_length,
                    self.settings.hook_address,
                    address,
                    self.settings.scratch_register,
                    code,
                )
                .map_err(|e| {
                    new_rewrite_error(OriginalCode, self.settings.hook_address, address, e)
                })?;
            }

            // Include hook code
            TRewriter::rewrite_code_with_buffer(
                self.settings.asm_code_ptr as *const u8,
                self.settings.asm_code_len,
                self.settings.asm_code_address,
                address + code.len(),
                self.settings.scratch_register,
                code,
            )
            .map_err(|e| {
                new_rewrite_error(CustomCode, self.settings.asm_code_address, address, e)
            })?;

            // Include original code after if required
            // hook is 'first'
            if self.settings.behaviour == AsmHookBehaviour::ExecuteFirst {
                TRewriter::rewrite_code_with_buffer(
                    self.settings.hook_address as *const u8,
                    self.orig_code_length,
                    self.settings.hook_address,
                    address + code.len(),
                    self.settings.scratch_register,
                    code,
                )
                .map_err(|e| {
                    new_rewrite_error(OriginalCode, self.settings.hook_address, address, e)
                })?;
            }

            // Add jump back to the end of the sequence
            create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
                address.wrapping_add(code.len()),
                self.can_relative_jump,
                self.jump_back_address,
                self.settings.scratch_register,
                code,
            )
            .map_err(|e| HookBuilderError::JitError(e))?;

            Ok(())
        }
    }
}

/// Retrieves the max possible ASM length for the hook code (i.e. 'hook enabled')
/// once emplaced in the 'Hook Function'.
///
/// 'Hook Function': See diagram in docs/dev/design/assembly-hooks/overview.md
///
/// # Parameters
/// - `settings`: The settings for the assembly hook.
/// - `max_orig_code_length`: The maximum possible length of the original code.
fn hookfunction_max_len<TDisassembler, TRewriter, TRegister: Clone, TJit>(
    settings: &AssemblyHookSettings<TRegister>,
    max_orig_code_length: usize,
) -> usize
where
    TRegister: RegisterInfo + Copy,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TJit: Jit<TRegister>,
{
    // New code length + extra max possible length + jmp back to original code
    let hook_code_max_length = get_relocated_code_length::<TDisassembler, TRewriter, TRegister>(
        settings.asm_code_ptr,
        settings.asm_code_len,
    )
    .0;

    let result = hook_code_max_length + TJit::max_branch_bytes() as usize;
    if settings.behaviour == AsmHookBehaviour::DoNotExecuteOriginal {
        result
    } else {
        result + max_orig_code_length
    }
}
