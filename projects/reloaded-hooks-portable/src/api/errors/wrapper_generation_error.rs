extern crate alloc;

use alloc::string::String;
use thiserror_no_std::Error;

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error)]
pub enum WrapperGenerationError {
    /// Failed to initialize 3rd party assembler
    #[error("A Scratch Register Was Needed, But Was Not Found: {0:?}")]
    NoScratchRegister(String),
}
