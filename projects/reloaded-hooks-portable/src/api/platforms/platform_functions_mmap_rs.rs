extern crate alloc;
use alloc::string::String;
use mmap_rs::{MemoryArea, Mmap, MmapOptions};

/// Removes protection from a memory region.
/// This makes it such that existing game code can be safely overwritten.
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
///
/// # Returns
///
/// The old memory protection (if needed for call to [`self::reprotect_memory`]).
/// If the value returns `None`, then reprotect_memory will not be called.
///
/// # Returns
///
/// Success or error.
pub fn unprotect_memory_mmap_rs(address: *const u8, size: usize) -> Result<Option<usize>, String> {
    /*
    let mut mapping = MmapOptions::new(size).unwrap().with_address(1111);
    let mut mmap = mapping.map_none().unwrap();

    // Implement your logic to unprotect the memory here.
    // Returning an example Result
    Ok(Some(protection))
    */
    Ok(Some(0 as usize))
}

/// Removes protection from a memory region.
/// This makes it such that existing game code can be safely overwritten.
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
/// - `protection`: The protection returned in the result of the call to [`self::disable_write_xor_execute`].
///
/// # Returns
///
/// Success or error.
pub fn reprotect_memory_mmap_rs(
    address: *const u8,
    size: usize,
    protection: usize,
) -> Result<(), String> {
    Ok(())
}
