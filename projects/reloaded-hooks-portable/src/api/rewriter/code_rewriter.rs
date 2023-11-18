extern crate alloc;
use alloc::string::String;
use thiserror_no_std::Error;

/// The trait for a Just In Time Compiler used for translating code
/// from one address to another.
pub trait CodeRewriter {
    /// Rewrites the code from one address to another.
    ///
    /// Given an original block of code starting at `old_address`, this function
    /// will modify any relative addressing instructions to make them compatible
    /// with a new location starting at `new_address`.
    ///
    /// This is useful, for example, when code is being moved or injected into a new
    /// location in memory and any relative jumps or calls within the code need to be
    /// adjusted to the new location.
    ///
    /// # Parameters
    ///
    /// * `old_address`: A pointer to the start of the original block of code.
    /// * `old_address_size`: Size/amount of bytes to encode for the new address.
    /// * `new_address`: The new address for the instructions.
    ///
    /// # Behaviour
    ///
    /// The function will iterate over the block of code byte by byte, identifying any
    /// instructions that use relative addressing. When such an instruction is identified,
    /// its offset is adjusted to account for the difference between `old_address` and `new_address`.
    ///
    /// # Returns
    ///
    /// Either a re-encode error, in which case the operation fails, or a slice of bytes to be written.
    /// If there is not sufficient space for the slice of bytes, the function will be called again
    /// (with a larger space available at [`new_address`]).
    fn rewrite_code<'a>(
        old_address: *const u8,
        old_address_size: usize,
        new_address: *const u8,
    ) -> Result<&'a [u8], CodeRewriterError>;
}

/// Errors that can occur during JIT compilation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CodeRewriterError {
    /// Instruction cannot be re-encoded at this range.
    /// Note: This error should be unreachable for x86 and ARM64, which can re-encode any address.
    #[error(
        "The instruction cannot be re-encoded. Instruction offset: {0:?}, Instruction Name: {1:?}"
    )]
    OutOfRange(isize, String),

    /// Failed to disassemble instruction. Unknown or invalid.
    #[error("Failed to Disasemble Instruction. Instruction offset: {0:?}, Remaining Bytes (Starting with Instruction): {1:?}")]
    FailedToDisasm(String, String),

    /// Insufficient bytes to disassemble a single instruction.
    #[error("Insufficient bytes to disassemble a single instruction.")]
    InsufficientBytes,

    /// Missing a scratch register.
    #[error("Missing scratch register, required by function: {0:?}")]
    NoScratchRegister(String),

    #[error("Third party assembler error: {0:?}")]
    ThirdPartyAssemblerError(String),
}
