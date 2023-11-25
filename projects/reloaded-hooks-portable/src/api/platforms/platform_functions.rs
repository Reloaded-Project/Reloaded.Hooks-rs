extern crate alloc;

use crate::api::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};
use alloc::boxed::Box;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
use super::platform_functions_unix;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::platform_functions_apple;

#[cfg(target_os = "windows")]
use crate::api::platforms::platform_functions_windows;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use super::platform_functions_mmap_rs::unprotect_memory_mmap_rs;

pub(crate) static MUTUAL_EXCLUSOR: Mutex<()> = Mutex::new(());
static mut BUFFER_FACTORY: Option<Box<dyn BufferFactory>> = None;

/// Getter function for the BUFFER_FACTORY
pub(crate) fn get_buf_factory() -> &'static mut Box<dyn BufferFactory> {
    unsafe {
        BUFFER_FACTORY
            .as_mut()
            .expect("Buffer factory is not initialized")
    }
}

/// Setter function for the BUFFER_FACTORY
pub(crate) fn set_buf_factory(factory: Option<Box<dyn BufferFactory>>) {
    unsafe {
        BUFFER_FACTORY = factory;
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
/// This function is crucial to the operation of the library. On failure, we panic.
pub fn unprotect_memory(address: *const u8, size: usize) {
    // Implement your logic to unprotect the memory here.
    // Returning an example Result

    // Windows uses VirtualProtect
    #[cfg(target_os = "windows")]
    platform_functions_windows::unprotect_memory(address, size);

    // Non-apple unix platforms use mprotect
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
    platform_functions_unix::unprotect_memory(address, size);

    // I don't trust Apple to keep mmap working, so I'm doing manual implementation with mach_ APIs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    platform_functions_apple::unprotect_memory(address, size);

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    unprotect_memory_mmap_rs(address, size);
}

/// Temporarily disables write XOR execute protection with an OS specialized
/// API call (if available).
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
///
/// # Returns
///
/// - `usize`: The old memory protection (if needed for call to [`self::restore_write_xor_execute`]).
/// Or we panic.       
///
/// # Remarks
///
/// This is not currently used on any platform, but is intended for environments
/// which enforce write XOR execute, such as M1 macs.
///
/// The idea is that you use memory which is read_write_execute (MAP_JIT if mmap),
/// then disable W^X for the current thread. Then we write the code, and re-enable W^X.
pub fn disable_write_xor_execute(address: *const u8, size: usize) -> Result<Option<usize>, String> {
    // I don't trust Apple to keep mmap working, so I'm doing manual implementation with mach_ APIs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    platform_functions_apple::disable_write_xor_execute(address, size);

    Ok(None)
}

/// Restores write XOR execute protection.
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
/// - `protection`: The protection returned in the result of the call to [`self::disable_write_xor_execute`].
///
/// # Returns
///
/// Success or panic.
pub fn restore_write_xor_execute(
    address: *const u8,
    size: usize,
    protection: usize,
) -> Result<(), String> {
    // I don't trust Apple to keep mmap working, so I'm doing manual implementation with mach_ APIs.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    platform_functions_apple::restore_write_xor_execute(address, size, protection);

    Ok(())
}
