extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use core::mem;
use mmap_rs_with_map_from_existing::{MemoryArea, Mmap, MmapOptions};

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
pub fn unprotect_memory_mmap_rs(address: *const u8, size: usize) {
    // Make map object from existing memory region
    let map = unsafe {
        MmapOptions::new(size)
            .unwrap()
            .with_address(address as usize)
            .map_from_existing()
            .unwrap()
    };

    // Call change permission API
    let newmap = unsafe {
        map.make_exec_mut();
    };

    // Don't dealloc it!
    mem::forget(newmap);
}
