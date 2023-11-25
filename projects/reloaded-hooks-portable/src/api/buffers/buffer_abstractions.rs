extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

/// This trait defines a factory for retrieving buffers which satisfy given constraints.
/// This factory is used by the Wrapper and Hook generator to insert new code
/// within a given proximity of a target address.
///
/// # Remarks
///
/// The factory should be thread safe.
pub trait BufferFactory: Sync + Send {
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
    /// This buffer should not cross any page boundary, such that permission changes .
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
    ) -> Result<Box<dyn Buffer>, String>;

    /// Returns any available buffer (RWX)
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
    fn get_any_buffer(&mut self, size: u32, alignment: u32) -> Result<Box<dyn Buffer>, String>;
}

pub trait Buffer {
    /// Returns the address to which the data will be written to when
    /// you call the `write` method. This pointer is advanced for every write.
    fn get_address(&self) -> *const u8;

    /// Writes the specified data to the buffer; advancing the buffer pointer.
    /// # Returns.
    ///
    /// The new write address. Same as you would get calling [`self::get_address`].
    fn write(&mut self, buffer: &[u8]) -> *const u8;
}
