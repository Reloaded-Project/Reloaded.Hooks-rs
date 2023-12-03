use core::ptr::copy_nonoverlapping;

use super::icache_clear::clear_instruction_cache;
use crate::api::platforms::platform_functions::{
    disable_write_xor_execute, restore_write_xor_execute, unprotect_memory,
};

/// Overwrites existing code in (.text) or equivalent region of native memory.
///
/// #  Remarks
///
/// - Assumes existing code is in region that may not currently have write permission.
///
/// handling edge cases such as write xor execute.
/// and instruction cache invalidation.
pub(crate) fn overwrite_code(address: usize, buffer: &[u8]) {
    // No-op on non W^X platforms thanks to compiler optimizations.
    let orig = disable_write_xor_execute(address as *const u8, buffer.len());

    // If this is not a W^X platform (none is returned), we unprotect the code region
    if orig.is_none() {
        unprotect_memory(address as *const u8, buffer.len());
    }

    unsafe {
        copy_nonoverlapping(buffer.as_ptr(), address as *mut u8, buffer.len());
    }

    if let Some(orig_val) = orig {
        restore_write_xor_execute(address as *const u8, buffer.len(), orig_val);
    }

    // No-op on x86 platforms
    clear_instruction_cache(address as *const u8, (address + buffer.len()) as *const u8);
}
