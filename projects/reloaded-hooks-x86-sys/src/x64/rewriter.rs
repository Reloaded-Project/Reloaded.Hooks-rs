extern crate alloc;

use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

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

pub(crate) fn rewrite_code_x64(
    _old_address: *const u8,
    _old_address_size: usize,
    _new_address: *const u8,
    _scratch_register: Option<u8>,
) -> Result<Vec<u8>, CodeRewriterError> {
    todo!();
}
