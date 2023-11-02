extern crate alloc;

use crate::instructions::{add_immediate::AddImmediate, adr::Adr, mov_immediate::MovImmediate};
use alloc::{boxed::Box, vec::Vec};
use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;

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
/// # Behaviour
///
/// The function will iterate over the block of code byte by byte, identifying any
/// instructions that use relative addressing. When such an instruction is identified,
/// its offset is adjusted to account for the difference between `old_address` and `new_address`.
///
/// # Returns
///
/// Returns the number of bytes written to `out_address`. Otherwise an error.
pub(crate) fn rewrite_code_aarch64(
    _old_address: *const u8,
    _old_address_size: usize,
    _new_address: *const u8,
    _out_address: *mut u8,
    _out_address_size: usize,
    _buf: Box<dyn Buffer>,
) -> i32 {
    todo!()
}
