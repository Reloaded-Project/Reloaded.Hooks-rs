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
pub trait BufferFactory<TBuffer: Buffer>: Sync + Send {
    /// Returns a buffer which satisfies the given constraints.
    /// If no such buffer exists, returns None.
    ///
    /// Returned buffer must be suitable for writing executable code to.
    /// Calls to 'write' on the returned buffer must write executable code.
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
        size: u32,
        target: usize,
        proximity: usize,
        alignment: u32,
    ) -> Result<Box<TBuffer>, String>;

    /// Returns any available buffer. This buffer must be suitable for writing executable code to.
    /// Calls to 'write' on the returned buffer must write executable code.
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
    fn get_any_buffer(size: u32, alignment: u32) -> Result<Box<TBuffer>, String>;
}

pub trait Buffer {
    /// Returns the address to which the data will be written to when
    /// you call the `write` method. This pointer is advanced for every write.
    fn get_address(&self) -> *const u8;

    /// Writes the specified data to the buffer; advancing the buffer pointer.
    ///
    /// # Returns.
    ///
    /// The new write address. Same as you would get calling [`self::get_address`].
    ///
    /// # Remarks
    ///
    /// Note to implementers: The buffer may be executable code. Care must be taken to ensure that the
    /// data written may be executed and read.
    fn write(&mut self, buffer: &[u8]) -> *const u8;

    /// Advances the buffer by a specified number of bytes.
    /// Effectively, this is a write operation, but without any data being written to the buffer.
    ///
    /// # Returns.
    ///
    /// The new write address. Same as you would get calling [`self::get_address`].
    fn advance(&mut self, num_bytes: usize) -> *const u8;

    /// Overwrites data written to a buffer created by this trait at a given address.
    /// Use this method to safely overwrite data written to a buffer.
    ///
    /// # Remarks
    ///
    /// This method works around the complicated tidbits of writing to buffer, such as instruction
    /// cache invalidation and permission changes on W^X systems where applicable.
    ///
    /// # Parameters
    ///
    /// - `address`: The address to overwrite.
    /// - `buffer`: The buffer to overwrite with.
    fn overwrite(address: usize, buffer: &[u8])
    where
        Self: Sized;

    /// Overwrites data written to a buffer created by this trait at a given address.
    /// Using a native integer type, such that an atomic operation is performed.
    ///
    /// # Remarks
    ///
    /// This method works around the complicated tidbits of writing to buffer, such as instruction
    /// cache invalidation and permission changes on W^X systems where applicable.
    ///
    /// `TInteger` must be a native integer type, such as `u8`, `u16`, `u32`, `u64`, `u128` which
    /// can be written using a single instruction.
    ///
    /// # Parameters
    ///
    /// - `address`: The address to overwrite.
    /// - `buffer`: The native integer type to write.
    fn overwrite_atomic<TInteger>(address: usize, buffer: TInteger)
    where
        Self: Sized;
}
