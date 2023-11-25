extern crate alloc;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::ptr::NonNull;
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
        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                buffer_ptr.add(current_offset as usize),
                data.len(),
            );
        }

        *self.buffer.write_offset.borrow_mut() = end; // Mutable borrow to update
        unsafe { buffer_ptr.add(end as usize) }
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
