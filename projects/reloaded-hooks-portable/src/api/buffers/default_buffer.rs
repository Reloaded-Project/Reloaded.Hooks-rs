extern crate alloc;

use crate::api::platforms::platform_functions::{
    disable_write_xor_execute, restore_write_xor_execute,
};
use crate::helpers::icache_clear::clear_instruction_cache;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::ptr::{copy_nonoverlapping, NonNull};
use core::sync::atomic::{AtomicBool, Ordering};

use super::buffer_abstractions::Buffer;

pub struct AllocatedBuffer {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) write_offset: RefCell<u32>,
    pub(crate) size: u32,
    pub(crate) locked: AtomicBool,
}

impl AllocatedBuffer {
    /// Returns the number of bytes remaining in the buffer.
    pub fn remaining_bytes(&self) -> u32 {
        self.size - *self.write_offset.borrow()
    }
}

impl Clone for AllocatedBuffer {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            write_offset: self.write_offset.clone(),
            size: self.size,
            locked: AtomicBool::new(self.locked.load(Ordering::Relaxed)),
        }
    }
}

pub struct LockedBuffer {
    pub(crate) buffer: Rc<AllocatedBuffer>,
}

impl Buffer for LockedBuffer {
    fn get_address(&self) -> *const u8 {
        unsafe {
            self.buffer
                .ptr
                .as_ptr()
                .add(*self.buffer.write_offset.borrow() as usize)
        }
    }

    fn write(&mut self, data: &[u8]) -> *const u8 {
        let current_offset = *self.buffer.write_offset.borrow();
        let end = data.len() as u32 + current_offset;
        debug_assert!(end <= self.buffer.size, "Buffer overflow");

        let buffer_ptr = self.buffer.ptr.as_ptr();

        // Make buffer RW for W^X
        let orig = disable_write_xor_execute(buffer_ptr as *const u8, data.len());
        unsafe {
            copy_nonoverlapping(
                data.as_ptr(),
                buffer_ptr.add(current_offset as usize),
                data.len(),
            );
        }

        *self.buffer.write_offset.borrow_mut() = end; // Mutable borrow to update
        let result = unsafe { buffer_ptr.add(end as usize) };

        // Make code executable again for W^X
        if let Some(orig_val) = orig {
            restore_write_xor_execute(buffer_ptr as *const u8, data.len(), orig_val);
        }

        clear_instruction_cache(
            buffer_ptr as *const u8,
            (buffer_ptr as usize + data.len()) as *const u8,
        );
        result
    }

    fn overwrite(address: usize, buffer: &[u8]) {
        let orig = disable_write_xor_execute(address as *const u8, buffer.len());
        unsafe {
            copy_nonoverlapping(buffer.as_ptr(), address as *mut u8, buffer.len());
        }

        if let Some(orig_val) = orig {
            restore_write_xor_execute(address as *const u8, buffer.len(), orig_val);
        }

        clear_instruction_cache(address as *const u8, (address + buffer.len()) as *const u8);
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        self.buffer.locked.store(false, Ordering::Release); // unlock buffer when done
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        // These buffers are supposed to last the lifetime of the process.
    }
}
