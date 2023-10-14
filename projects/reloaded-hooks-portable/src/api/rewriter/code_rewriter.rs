extern crate alloc;
use crate::api::buffers::buffer_abstractions::BufferFactory;
use alloc::boxed::Box;
use alloc::string::String;
use thiserror_no_std::Error;

/// The trait for a Just In Time Compiler used for emitting
/// wrappers assembled for a given address.
pub trait CodeRewriter {
    /// Rewrites the code from one address to another.
    ///
    /// Given an original block of code starting at `old_address`, this function
    /// will modify any relative addressing instructions to make them compatible
    /// with a new location starting at `new_address`. This is useful, for example,
    /// when code is being moved or injected into a new location in memory and any
    /// relative jumps or calls within the code need to be adjusted to the new location.
    ///
    /// # Parameters
    ///
    /// * `old_address`: A pointer to the start of the original block of code.
    /// * `old_address_size`: A pointer to the start of the original block of code.
    /// * `new_address`: A pointer to the start of the location where the code will be moved.
    ///                  New code will be written to this address.
    /// * `out_address`: A pointer to where the new data will be written to.
    /// * `out_address_size`: Size of data at out_address.
    /// * `buf`: The buffer to use for writing the new code.
    ///
    /// # Behavior
    ///
    /// The function will iterate over the block of code byte by byte, identifying any
    /// instructions that use relative addressing. When such an instruction is identified,
    /// its offset is adjusted to account for the difference between `old_address` and `new_address`.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes written to `new_address`. Otherwise an error.
    fn rewrite_code(
        old_address: *const u8,
        old_address_size: usize,
        new_address: *const u8,
        out_address: *mut u8,
        out_address_size: usize,
        buf: Box<dyn BufferFactory>,
    ) -> Result<i32, CodeRewriterError>;
}

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error)]
pub enum CodeRewriterError {
    /// Instruction cannot be re-encoded at this range.
    #[error(
        "The instruction cannot be re-encoded. Instruction offset: {0:?}, Instruction Name: {1:?}"
    )]
    OurOfRange(i32, String),
}
