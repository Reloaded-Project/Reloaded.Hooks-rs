extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use thiserror_no_std::Error;

/// The trait for a Just In Time Compiler used for translating code
/// from one address to another.
pub trait CodeRewriter<TRegister> {
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
    /// * `old_code`: A pointer to the start of the original block of code.
    /// * `old_code_size`: Amount of bytes to rewrite. This must not contain any partial instructions.
    /// * `old_address`: The address to assume as the source location of the old code.
    /// * `new_address`: The new address for the instructions.
    /// * `scratch_register`
    ///     - A scratch general purpose register that can be used for operations.
    ///     - This scratch register may or may not be used depending on the code being rewritten.
    ///
    /// # Behaviour
    ///
    /// The function will iterate over the block of code byte by byte, identifying any
    /// instructions that use relative addressing. When such an instruction is identified,
    /// its offset is adjusted to account for the difference between `old_address` and `new_address`.
    ///
    /// # Returns
    ///
    /// Either a re-encode error, in which case the operation fails, or a vector to consume.
    ///
    /// # Safety
    ///
    /// Dereferences raw pointers, please ensure that the pointers are valid.
    unsafe fn rewrite_code(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<TRegister>,
    ) -> Result<Vec<u8>, CodeRewriterError> {
        // Calculate the capacity for the buffer
        let mut buffer = Vec::with_capacity(old_code_size * 2);

        // Call rewrite_code_with_buffer with the created buffer
        Self::rewrite_code_with_buffer(
            old_code,
            old_code_size,
            old_address,
            new_address,
            scratch_register,
            &mut buffer,
        )?;

        Ok(buffer)
    }

    /// Rewrites the code from one address to another, using an existing buffer.
    ///
    /// This function behaves similarly to `rewrite_code`, but instead of returning a new `Vec<u8>`,
    /// it accepts a mutable reference to an existing `Vec<u8>` and modifies it in place.
    ///
    /// # Parameters
    ///
    /// * `old_code`: A pointer to the start of the original block of code.
    /// * `old_code_size`: The size, in bytes, of the block of code to rewrite.
    /// * `old_address`: The address assumed to be the original location of the code.
    /// * `new_address`: The new address where the rewritten code will be executed.
    /// * `scratch_register`:
    ///     - An optional scratch general purpose register that can be used for operations.
    ///     - This register may or may not be used, depending on the specifics of the code being rewritten.
    /// * `existing_buffer`: The rewritten code is appended here.
    ///
    /// # Behavior
    ///
    /// The function will iterate over the block of code byte by byte, identifying any
    /// instructions that use relative addressing. When such an instruction is identified,
    /// its offset is adjusted to account for the difference between `old_address` and `new_address`.
    ///
    /// The existing buffer is not automatically cleared before use. If a fresh buffer is needed,
    /// it should be cleared or appropriately prepared by the caller before passing it to this function.
    ///
    /// # Returns
    ///
    /// If successful, the function returns `Ok(())`. In case of a re-encode error, it returns
    /// `Err(CodeRewriterError)`, indicating the operation failed.
    ///
    /// # Safety
    ///
    /// - Dereferences raw pointers. Ensure that `old_code` points to a valid block of code
    ///   of size `old_code_size` and that it is safe to read from.
    unsafe fn rewrite_code_with_buffer(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<TRegister>,
        existing_buffer: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError>;

    /// Returns the maximum number of bytes that a single instruction can increase in size
    fn max_ins_size_increase() -> usize;
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
