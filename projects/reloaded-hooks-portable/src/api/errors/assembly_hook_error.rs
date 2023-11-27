use crate::api::rewriter::code_rewriter::CodeRewriterError;
use derive_new::new;
use thiserror_no_std::Error;

/// Errors that can occur during assembly hook creation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssemblyHookError {
    /// Instruction cannot be re-encoded at this range.
    /// Note: This error should be unreachable for x86 and ARM64, which can re-encode any address.
    /// Parameters: (actual_bytes, max_bytes)
    #[error(
        "Too many bytes were required {0:?} to encode the instruction. Maximum permitted: {1:?}"
    )]
    TooManyBytes(usize, usize),

    /// Failed to rewrite code from old address to new address.
    /// Usually because a scratch register is missing, in practice.
    ///
    /// Parameters: (actual_bytes, max_bytes)
    #[error("Failed to rewrite code. Source: {0:?}, Error: {1:?}")]
    RewriteError(RewriteErrorDetails, CodeRewriterError),
}

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RewriteErrorSource {
    /// Failed to re-encode original code.
    OriginalCode,

    /// Failed to re-encode new code
    CustomCode,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, new)]
pub struct RewriteErrorDetails {
    pub source: RewriteErrorSource,
    pub original_location: usize,
    pub new_location: usize,
}
