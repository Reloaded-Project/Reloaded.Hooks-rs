extern crate alloc;

use alloc::string::String;
use core::ptr::null;
use core::{cell::UnsafeCell, mem::transmute};

use crate::api::buffers::{
    buffer_abstractions::BufferFactory, default_buffer_factory::DefaultBufferFactory,
};

use alloc::boxed::Box;

pub struct PlatformFunctions {
    /// The factory for creating read/write/execute buffers used by the library.
    pub buffer_factory: UnsafeCell<Box<dyn BufferFactory>>,

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
    /// The old memory protection (if needed for call to [`self::reprotect_memory`]).
    /// If the value returns `None`, then reprotect_memory will not be called.
    ///
    /// # Returns
    ///
    /// Success or error.
    pub unprotect_memory: fn(*const u8, usize, usize) -> Result<Option<usize>, String>,

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
    pub reprotect_memory: fn(*const u8, usize, usize) -> Result<(), String>,

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
    pub disable_write_xor_execute: fn(*const u8, usize) -> Result<Option<usize>, String>,

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
    pub restore_write_xor_execute: fn(*const u8, usize) -> Result<(), String>,
}

impl Default for PlatformFunctions {
    fn default() -> Self {
        PlatformFunctions {
            buffer_factory: UnsafeCell::new(Box::new(DefaultBufferFactory::new())),
            unprotect_memory: unsafe { transmute(null::<()>()) },
            reprotect_memory: unsafe { transmute(null::<()>()) },
            disable_write_xor_execute: unsafe { transmute(null::<()>()) },
            restore_write_xor_execute: unsafe { transmute(null::<()>()) },
        }
    }
}
