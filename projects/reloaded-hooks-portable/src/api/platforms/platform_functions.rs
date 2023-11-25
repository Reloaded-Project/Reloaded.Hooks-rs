extern crate alloc;

use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::api::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};

use alloc::boxed::Box;

use super::{
    platform_functions_apple,
    platform_functions_mmap_rs::{reprotect_memory_mmap_rs, unprotect_memory_mmap_rs},
};

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

/// See [`unprotect_memory`].
pub static mut UNPROTECT_MEMORY: fn(*const u8, usize) -> Result<Option<usize>, String> =
    unprotect_memory;

/// See [`reprotect_memory`].
pub static mut REPROTECT_MEMORY: fn(*const u8, usize, usize) -> Result<(), String> =
    reprotect_memory;

/// See [`disable_write_xor_execute`].
pub static mut DISABLE_WRITE_XOR_EXECUTE: fn(*const u8, usize) -> Result<Option<usize>, String> =
    disable_write_xor_execute;

/// See [`restore_write_xor_execute`].
pub static mut RESTORE_WRITE_XOR_EXECUTE: fn(*const u8, usize, usize) -> Result<(), String> =
    restore_write_xor_execute;

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
pub fn unprotect_memory(address: *const u8, size: usize) -> Result<Option<usize>, String> {
    // Implement your logic to unprotect the memory here.
    // Returning an example Result
    unprotect_memory_mmap_rs(address, size)
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
pub fn reprotect_memory(address: *const u8, size: usize, protection: usize) -> Result<(), String> {
    reprotect_memory_mmap_rs(address, size, protection)
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
///            
///
/// # Remarks
///
/// This is not currently used on any platform, but is intended for environments
/// which enforce write XOR execute, such as M1 macs.
///
/// The idea is that you use memory which is read_write_execute (MAP_JIT if mmap),
/// then disable W^X for the current thread. Then we write the code, and re-enable W^X.
pub fn disable_write_xor_execute(address: *const u8, size: usize) -> Result<Option<usize>, String> {
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
/// Success or error.
pub fn restore_write_xor_execute(
    address: *const u8,
    size: usize,
    protection: usize,
) -> Result<(), String> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    platform_functions_apple::restore_write_xor_execute(address, size, protection);

    Ok(())
}
