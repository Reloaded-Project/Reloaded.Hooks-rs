extern crate alloc;

use crate::api::{
    buffers::buffer_abstractions::{Buffer, BufferFactory},
    jit::compiler::Jit,
    settings::proximity_target::ProximityTarget,
    traits::register_info::RegisterInfo,
};
use alloc::boxed::Box;

/// Allocates a buffer with a specified proximity to a target address.
/// Returns a tuple of (is_proximity, buffer).
///
/// # Arguments
///
/// * `target_address` - The target address to allocate a buffer for.
/// * `target_size` - The size of the target address.
///
/// # Returns
///
/// Returns a tuple of (is_proximity, buffer).
/// * `is_proximity` - Whether the buffer was allocated with any JIT specified proximity.
///                    If false, it was allocated in random place in RAM.
/// * `buffer` - The allocated buffer.
pub(crate) fn allocate_with_proximity<
    TJit,
    TRegister,
    TBufferFactory: BufferFactory<TBuffer>,
    TBuffer: Buffer,
>(
    target_address: usize,
    target_size: u32,
) -> (bool, Box<TBuffer>)
where
    TJit: Jit<TRegister>,
    TRegister: RegisterInfo,
{
    // Try known relative jump ranges.
    for &requested_proximity in TJit::max_relative_jump_distances() {
        let proximity_target =
            ProximityTarget::new(target_address, target_size, requested_proximity);

        let buf_opt = TBufferFactory::get_buffer(
            target_size,
            proximity_target.target_address,
            proximity_target.requested_proximity,
            TJit::code_alignment(),
        );

        if let Ok(buffer) = buf_opt {
            return (true, buffer);
        }
    }

    let buf_boxed = TBufferFactory::get_any_buffer(target_size, TJit::code_alignment()).unwrap();
    (false, buf_boxed)
}
