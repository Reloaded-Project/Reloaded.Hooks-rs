extern crate alloc;
use crate::api::errors::hook_builder_error::RewriteErrorSource::CustomCode;
use crate::{
    api::{
        errors::hook_builder_error::{HookBuilderError, RewriteErrorSource::OriginalCode},
        rewriter::code_rewriter::CodeRewriter,
        traits::register_info::RegisterInfo,
    },
    internal::{stub_builder::new_rewrite_error, stub_builder_settings::HookBuilderSettingsMixin},
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use derive_new::new;

/// Mixin that provides the user with the ability to have a precompiled assembly for both
/// the 'hook' and 'original' function.
#[derive(new)]
pub struct AssemblyMixin<
    'a,
    TRegister: Clone + RegisterInfo + Default + Copy,
    TRewriter: CodeRewriter<TRegister>,
> {
    /// Contains the bytes of the 'original function'.
    orig_function: &'a [u8],

    /// The memory address associated wioth the original function.
    orig_function_address: usize,

    /// Contains the bytes of the 'hook function'.
    hook_function: &'a [u8],

    /// The memory address associated wioth the original function.
    hook_function_address: usize,

    /// An optional 'scratch register' that can be used to re-encode the original code to a new location.
    /// This is not required for x86, others require it.
    ///
    /// This is only required if platform does not support 'Targeted Memory Allocation', i.e. more
    /// esoteric platforms.
    pub scratch_register: Option<TRegister>,

    _reg: PhantomData<TRegister>,
    _rw: PhantomData<TRewriter>,
}

impl<'a, TRegister: Clone + RegisterInfo + Default + Copy, TRewriter: CodeRewriter<TRegister>>
    HookBuilderSettingsMixin<TRegister> for AssemblyMixin<'a, TRegister, TRewriter>
{
    fn get_orig_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>> {
        unsafe {
            TRewriter::rewrite_code_with_buffer(
                self.orig_function.as_ptr(),
                self.orig_function.len(),
                self.orig_function_address,
                address,
                self.scratch_register,
                code,
            )
            .map_err(|e| new_rewrite_error(OriginalCode, self.orig_function_address, address, e))?;
        }

        Ok(())
    }

    fn get_hook_function(
        &mut self,
        address: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), HookBuilderError<TRegister>> {
        unsafe {
            TRewriter::rewrite_code_with_buffer(
                self.hook_function.as_ptr(),
                self.hook_function.len(),
                self.hook_function_address,
                address,
                self.scratch_register,
                code,
            )
            .map_err(|e| new_rewrite_error(CustomCode, self.hook_function_address, address, e))?;
        }

        Ok(())
    }
}
