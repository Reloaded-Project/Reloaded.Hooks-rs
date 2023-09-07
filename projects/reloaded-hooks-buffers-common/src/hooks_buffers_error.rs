extern crate alloc;

use alloc::string::String;
use thiserror_no_std::Error;

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error)]
pub enum BuffersError {
    /// Reloaded.Memory.Buffers (dynamic) error
    #[error("Buffers (Dynamic Linked) Error: {0:?}")]
    DynamicLinkError(String),

    /// Reloaded.Memory.Buffers (static) error
    #[error("Buffers (Static Linked) Error: {0:?}")]
    StaticLinkError(String),
}
