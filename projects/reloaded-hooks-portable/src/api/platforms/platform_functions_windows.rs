use core::ffi::c_void;

use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

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
    let mut oldprot: PAGE_PROTECTION_FLAGS = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        let result = VirtualProtect(
            address as *const c_void,
            size,
            PAGE_EXECUTE_READWRITE,
            (&mut oldprot) as *mut PAGE_PROTECTION_FLAGS,
        );

        if result == false {
            panic!("Failed to unprotect memory");
        }
    }
}
