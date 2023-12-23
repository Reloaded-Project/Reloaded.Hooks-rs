extern crate alloc;

use super::hook_builder_error::HookBuilderError;
use crate::api::jit::compiler::JitError;
use thiserror_no_std::Error;

/// Errors that can occur during assembly hook creation.
#[derive(Debug, Error)]
pub enum AssemblyHookError<TRegister> {
    /// Too many bytes required to jump to stub.
    #[error("Too many bytes were required {0:?} to jump to stub. Maximum permitted: {1:?}")]
    TooManyBytes(usize, usize),

    /// Failed to rewrite code from old address to new address.
    /// Usually because a scratch register is missing, in practice.
    ///
    /// Parameters: (actual_bytes, max_bytes)
    #[error("Hook Builder Error: {0:?}")]
    HookBuilderError(#[from] HookBuilderError<TRegister>),

    /// JIT related error.
    #[error("Error in JIT: {0:?}")]
    JitError(JitError<TRegister>),
}
