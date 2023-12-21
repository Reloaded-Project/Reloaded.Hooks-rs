extern crate alloc;
use libc::c_void;
use mmap_rs_with_map_from_existing::MmapOptions;

static mut PAGE_SIZE: Option<usize> = None;

fn page_size() -> usize {
    unsafe {
        if PAGE_SIZE.is_none() {
            PAGE_SIZE = Some(MmapOptions::page_size());
        }
        PAGE_SIZE.unwrap()
    }
}

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
        let rounded_down = address as usize / page_size() * page_size();
        let result = libc::mprotect(
            rounded_down as *mut c_void,
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        );

        if result != 0 {
            panic!("Failed to unprotect memory");
        }
    }
}
