extern crate alloc;
use super::{
    hook_builder_error::HookBuilderError, wrapper_generation_error::WrapperGenerationError,
};
use crate::api::jit::compiler::JitError;
use thiserror_no_std::Error;

/// Errors that can occur during 'fast' hook creation.
#[derive(Debug, Error)]
pub enum FunctionHookError<TRegister> {
    /// Wrapper generation failed for some reason
    #[error("Failed to Generate Calling Convention Wrapper: {0:?}")]
    WrapperGenerationError(#[from] WrapperGenerationError),

    #[error("Error: {0:?}")]
    StringError(#[from] &'static str),

    #[error("Hook Builder Error: {0:?}")]
    HookBuilderError(#[from] HookBuilderError<TRegister>),

    /// JIT related error.
    #[error("JitError: {0:?}")]
    JitError(#[from] JitError<TRegister>),
}
