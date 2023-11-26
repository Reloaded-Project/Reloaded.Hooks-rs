extern crate alloc;
use libc::c_void;

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
/// Success or error.
pub fn unprotect_memory(address: *const u8, size: usize) {
    unsafe {
        let result = libc::mprotect(
            address as *mut c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        );

        if result != 0 {
            panic!("Failed to unprotect memory");
        }
    }
}
