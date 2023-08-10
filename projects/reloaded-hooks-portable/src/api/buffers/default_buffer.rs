extern crate alloc;
use alloc::alloc::{dealloc, Layout};
use alloc::sync::Arc;
use core::any::Any;
use core::cell::RefCell;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use super::buffer_abstractions::Buffer;

pub struct AllocatedBuffer {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) write_offset: RefCell<u32>,
    pub(crate) size: u32,
    pub(crate) layout: Layout,
    pub(crate) locked: AtomicBool,
}

impl AllocatedBuffer {
    /// Returns the number of bytes remaining in the buffer.
    pub fn remaining_bytes(&self) -> u32 {
        self.size - *self.write_offset.borrow()
    }
}

pub struct LockedBuffer {
    pub(crate) buffer: Arc<AllocatedBuffer>,
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

    fn write(&mut self, data: &[u8]) {
        let current_offset = *self.buffer.write_offset.borrow();
        let end = data.len() as u32 + current_offset;
        debug_assert!(end <= self.buffer.size, "Buffer overflow");

        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.buffer.ptr.as_ptr().add(current_offset as usize),
                data.len(),
            );
        }

        *self.buffer.write_offset.borrow_mut() = end; // Mutable borrow to update
    }

    // This is to enable downcasting from dyn Buffer to LockedBuffer
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        self.buffer.locked.store(false, Ordering::Release); // unlock buffer when done
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr.as_ptr(), self.layout) }
    }
}
