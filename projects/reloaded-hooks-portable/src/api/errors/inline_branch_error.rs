extern crate alloc;
use thiserror_no_std::Error;

/// Errors that can occur during assembly hook creation.
#[derive(Debug, Error)]
pub enum InlineBranchError {
    /// Array for branch was too short.
    /// Normally this should never happen, but is here as a sanity test.
    /// Used in debug configurations only.
    #[error("Array too short. Max length: {0:?}. Actual length: {1:?}")]
    ArrayTooShort(usize, usize),

    /// Array length was not expected for this architecture.
    /// This is likely indicative of a bug.
    #[error("Unexpected arr length. Length: {0:?}.")]
    UnexpectedArrayLength(usize),
}
