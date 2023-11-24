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
    TooManyBytes(isize, isize),
}
