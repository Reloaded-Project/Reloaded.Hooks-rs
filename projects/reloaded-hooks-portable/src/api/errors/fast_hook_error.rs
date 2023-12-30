extern crate alloc;
use crate::api::jit::compiler::JitError;
use thiserror_no_std::Error;

/// Errors that can occur during 'fast' hook creation.
#[derive(Debug, Error)]
pub enum FastHookError<TRegister> {
    /// Cannot decode an instruction at user provided address.
    #[error("Error: {0:?}")]
    StringError(#[from] &'static str),

    /// JIT related error.
    #[error("JitError: {0:?}")]
    JitError(#[from] JitError<TRegister>),
}
