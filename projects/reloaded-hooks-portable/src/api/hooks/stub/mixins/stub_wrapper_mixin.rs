extern crate alloc;
use crate::{
    api::{
        errors::hook_builder_error::{HookBuilderError, RewriteErrorSource::OriginalCode},
        jit::{compiler::Jit, operation::Operation},
        rewriter::code_rewriter::CodeRewriter,
        traits::register_info::RegisterInfo,
    },
    internal::{stub_builder::new_rewrite_error, stub_builder_settings::HookBuilderSettingsMixin},
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use derive_new::new;

/// Mixin that provides the user with the ability to have a 'function wrapper' as the 'hook' function.
#[derive(new)]
pub struct StubWrapperMixin<
    'a,
    TRegister: Clone + RegisterInfo + Default + Copy,
    TJit: Jit<TRegister>,
    TRewriter: CodeRewriter<TRegister>,
> {
    /// Contains the bytes of the 'original function'.
    orig_function: &'a [u8],

    /// The memory address associated wioth the original function.
    orig_function_address: usize,

    /// Jitter operation.
    jit_ops: &'a [Operation<TRegister>],

    /// An optional 'scratch register' that can be used to re-encode the original code to a new location.
    /// This is not required for x86, others require it.
    ///
    /// This is only required if platform does not support 'Targeted Memory Allocation', i.e. more
    /// esoteric platforms.
    pub scratch_register: Option<TRegister>,

    _reg: PhantomData<TRegister>,
    _rw: PhantomData<TRewriter>,
    _tj: PhantomData<TJit>,
}

impl<
        'a,
        TRegister: Clone + RegisterInfo + Default + Copy,
        TRewriter: CodeRewriter<TRegister>,
        TJit: Jit<TRegister>,
    > HookBuilderSettingsMixin<TRegister> for StubWrapperMixin<'a, TRegister, TJit, TRewriter>
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
        TJit::compile_with_buf(address, self.jit_ops, code)?;
        Ok(())
    }
}
