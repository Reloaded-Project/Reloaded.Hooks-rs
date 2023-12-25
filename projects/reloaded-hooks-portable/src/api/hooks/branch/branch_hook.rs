extern crate alloc;

use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        calling_convention_info::CallingConventionInfo,
        errors::function_hook_error::FunctionHookError,
        function_info::FunctionInfo,
        hooks::stub::mixins::{
            assembly_mixin::AssemblyMixin, stub_wrapper_mixin::StubWrapperMixin,
        },
        jit::{
            compiler::Jit,
            operation_aliases::{CallRel, JumpAbs, JumpRel},
        },
        length_disassembler::LengthDisassembler,
        platforms::platform_functions::MUTUAL_EXCLUSOR,
        rewriter::code_rewriter::CodeRewriter,
        settings::function_hook_settings::FunctionHookSettings,
        traits::register_info::RegisterInfo,
        wrapper_instruction_generator::{
            generate_wrapper_instructions, new_wrapper_instruction_generator_options,
            MAX_WRAPPER_LENGTH,
        },
    },
    helpers::{overwrite_code::overwrite_code, relative_branch_range_check::can_direct_branch},
    internal::{
        stub_builder::{create_hook_stub_buffer, create_stub},
        stub_builder_settings::{HookBuilderSettings, HookBuilderSettingsMixin},
    },
};
use alloc::vec::Vec;
use core::hash::Hash;
use core::{fmt::Debug, slice::from_raw_parts};

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
    TRegister: RegisterInfo + Clone + Default + Copy + Debug + Eq + Hash,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
    TFunctionInfo: FunctionInfo,
    TFunctionAttribute: CallingConventionInfo<TRegister>,
>(
    settings: &FunctionHookSettings<TRegister, TFunctionInfo, TFunctionAttribute>,
    original_fn_address: *mut usize,
) -> Result<(), FunctionHookError<TRegister>> {
    create_branch_hook_with_callback::<
        TJit,
        TRegister,
        TDisassembler,
        TRewriter,
        TBuffer,
        TBufferFactory,
        TFunctionInfo,
        TFunctionAttribute,
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
    TRegister: RegisterInfo + Clone + Default + Copy + Debug + Eq + Hash,
    TDisassembler: LengthDisassembler,
    TRewriter: CodeRewriter<TRegister>,
    TBuffer: Buffer,
    TBufferFactory: BufferFactory<TBuffer>,
    TFunctionInfo: FunctionInfo,
    TFunctionAttribute: CallingConventionInfo<TRegister>,
>(
    settings: &FunctionHookSettings<TRegister, TFunctionInfo, TFunctionAttribute>,
    original_val_receiver: impl FnOnce(usize),
) -> Result<(), FunctionHookError<TRegister>> {
    // Sufficient for relative/absolute jmp/call in any architecture
    const MAX_BRANCH_LENGTH: usize = 24;

    // Lock native function memory, to ensure we get accurate info at hook address.
    // This should make hooking operation thread safe provided no presence of 3rd party
    // library instances, which is a-ok for Reloaded3.
    let _guard = MUTUAL_EXCLUSOR.lock();

    // Decode the existing branch to be modified.
    // Assumption: 'on supported architectures jmp and call have the same length'
    let core_settings = settings.core_settings;
    let target = TJit::decode_call_target(
        core_settings.hook_address,
        TJit::standard_relative_call_bytes(),
    )?;

    // Get the original call, we will have this as the 'original code' in the stub.
    let orig_code = from_raw_parts(
        core_settings.hook_address as *const u8,
        TJit::standard_relative_call_bytes(),
    );

    original_val_receiver(target.target_address);

    // Generate wrapper if needed.
    let mut code = Vec::<u8>::with_capacity(MAX_BRANCH_LENGTH);

    if settings.needs_wrapper() {
        // Get stub buffer we will be using
        let mut alloc = create_hook_stub_buffer::<TJit, TRegister, TBuffer, TBufferFactory>(
            core_settings.hook_address,
            (MAX_WRAPPER_LENGTH * 2) + MAX_BRANCH_LENGTH,
        );
        debug_assert!(alloc.can_relative_jump);

        // Setup the mixin.
        let options = new_wrapper_instruction_generator_options::<TFunctionInfo, TRegister, TJit>(
            false,
            core_settings.new_target,
            &settings.function_info,
            settings.injected_parameter,
        );

        let wrap_instructions =
            generate_wrapper_instructions(settings.conv_target, settings.conv_source, &options)?;

        let mixin: &mut dyn HookBuilderSettingsMixin<TRegister> =
            &mut StubWrapperMixin::<TRegister, TJit, TRewriter>::new(
                orig_code,
                core_settings.hook_address,
                &wrap_instructions,
                core_settings.scratch_register,
            );

        let mut builder_settings = HookBuilderSettings::new(
            core_settings.hook_address,
            MAX_WRAPPER_LENGTH,
            settings.auto_activate,
        );

        // Create the stub
        let stub = create_stub::<TRegister, TBuffer>(&mut builder_settings, &mut alloc, mixin)?;

        // Lastly, write the branch to the buffer.
        let mut pc = core_settings.hook_address;
        let buf_ptr = alloc.buf.get_address() as usize;
        if target.is_call {
            TJit::encode_call(&CallRel::new(buf_ptr), &mut pc, &mut code)?;
        } else {
            TJit::encode_jump(&JumpRel::new(buf_ptr), &mut pc, &mut code)?;
        }

        overwrite_code(core_settings.hook_address, &code);

        // And return the good stuff.
        Ok(())
    } else {
        // Get stub buffer we will be using
        let mut alloc = create_hook_stub_buffer::<TJit, TRegister, TBuffer, TBufferFactory>(
            core_settings.hook_address,
            MAX_BRANCH_LENGTH * 3,
        );
        debug_assert!(alloc.can_relative_jump);

        // Make our 'hook code', being a branch to the new target.
        let buf_ptr = alloc.buf.get_address() as usize;
        let is_direct_branch = can_direct_branch(
            buf_ptr,
            core_settings.new_target,
            TJit::max_standard_relative_call_distance(),
            TJit::standard_relative_call_bytes(),
        );

        let mut pc = buf_ptr;
        if is_direct_branch {
            TJit::encode_jump(&JumpRel::new(core_settings.new_target), &mut pc, &mut code)?;
        } else {
            let reg = core_settings
                .scratch_register
                .ok_or("Scratch register is required for create_branch_hook_with_callback")?;

            TJit::encode_abs_jump(
                &JumpAbs::new_with_reg(core_settings.new_target, reg),
                &mut pc,
                &mut code,
            )?;
        }

        debug_assert!(code.len() <= MAX_BRANCH_LENGTH);
        let mixin: &mut dyn HookBuilderSettingsMixin<TRegister> =
            &mut AssemblyMixin::<TRegister, TRewriter>::new(
                orig_code,
                core_settings.hook_address,
                &code,
                buf_ptr,
                core_settings.scratch_register,
            );

        let mut builder_settings = HookBuilderSettings::new(
            core_settings.hook_address,
            MAX_BRANCH_LENGTH,
            settings.auto_activate,
        );

        // Create the stub
        let stub = create_stub::<TRegister, TBuffer>(&mut builder_settings, &mut alloc, mixin)?;

        // Lastly, write the branch to the buffer.
        let mut pc = core_settings.hook_address;
        let buf_ptr = alloc.buf.get_address() as usize;
        if target.is_call {
            TJit::encode_call(&CallRel::new(buf_ptr), &mut pc, &mut code)?;
        } else {
            TJit::encode_jump(&JumpRel::new(buf_ptr), &mut pc, &mut code)?;
        }

        overwrite_code(core_settings.hook_address, &code);

        // And return the good stuff.
        Ok(())
    }
}
