extern crate alloc;

use core::any::Any;

use alloc::boxed::Box;

/// This trait defines a factory for retrieving buffers which satisfy given constraints.
/// This factory is used by the Wrapper and Hook generator to insert new code
/// within a given proximity of a target address.
pub trait BufferFactory {
    /// Returns a buffer which satisfies the given constraints.
    /// If no such buffer exists, returns None.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes buffer must have free.
    /// - `target`: The target address of the buffer.
    /// - `proximity`: The maximum distance between the target address and the buffer.
    /// - `alignment`: The alignment of the buffer returned.
    ///
    /// # Returns
    ///
    /// A buffer which satisfies the given constraints, or None if no such buffer exists.
    ///
    /// # Thread Safety
    ///
    /// Returned buffers must be locked and not returned to the pool until they are dropped.
    fn get_buffer(
        &mut self,
        size: u32,
        target: usize,
        proximity: usize,
        alignment: u32,
    ) -> Option<Box<dyn Buffer + '_>>;

    /// Returns any available buffer.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of bytes buffer must have free.
    /// - `alignment`: The alignment of the buffer returned.
    ///
    /// # Returns
    ///
    /// A buffer to which the user can write to.
    ///
    /// # Thread Safety
    ///
    /// Returned buffers must be locked and not returned to the pool until they are dropped.
    fn get_any_buffer(&mut self, size: u32, alignment: u32) -> Option<Box<dyn Buffer + '_>>;
}

pub trait Buffer {
    /// Returns the address to which the data will be written to when
    /// you call the `write` method. This pointer is advanced for every write.
    fn get_address(&self) -> *const u8;

    /// Writes the specified data to the buffer; advancing the buffer pointer.
    fn write(&mut self, buffer: &[u8]);

    // This is to enable downcasting from dyn Buffer to LockedBuffer
    fn as_any(&self) -> &dyn Any;
}
