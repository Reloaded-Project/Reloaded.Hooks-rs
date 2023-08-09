extern crate alloc;
use alloc::alloc::{alloc, Layout};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use super::buffer_abstractions::{Buffer, BufferFactory};
use super::default_buffer::{AllocatedBuffer, LockedBuffer};

// TODO: Fix alignment here.
pub struct DefaultBufferFactory {
    buffers: Vec<Arc<AllocatedBuffer>>,
}

impl DefaultBufferFactory {
    pub fn new() -> Self {
        DefaultBufferFactory {
            buffers: Vec::new(),
        }
    }
}

impl Default for DefaultBufferFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferFactory for DefaultBufferFactory {
    fn get_buffer(
        &mut self,
        _size: u32,
        _target: usize,
        _proximity: usize,
        _alignment: u32,
    ) -> Option<Box<dyn Buffer>> {
        None
    }

    fn get_any_buffer(&mut self, size: u32, alignment: u32) -> Option<Box<dyn Buffer>> {
        for buffer in &self.buffers {
            if !buffer.locked.load(Ordering::Acquire)
                && buffer.size == size
                && buffer.layout.align() == alignment as usize
                && buffer
                    .locked
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                    == Ok(false)
            {
                return Some(Box::new(LockedBuffer {
                    buffer: buffer.clone(),
                }));
            }
        }

        // If no buffer was found, create a new one
        let layout = Layout::from_size_align(size as usize, alignment as usize).ok()?;
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            None
        } else {
            let buffer = Arc::new(AllocatedBuffer {
                ptr: NonNull::new(ptr).unwrap(),
                write_offset: RefCell::new(0),
                size,
                layout,
                locked: AtomicBool::new(true),
            });

            self.buffers.push(buffer.clone());
            Some(Box::new(LockedBuffer { buffer }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn create_factory() {
        let factory = DefaultBufferFactory::new();
        assert_eq!(factory.buffers.len(), 0);
    }

    #[test]
    fn acquire_and_release_buffer() {
        let mut factory = DefaultBufferFactory::new();

        // Acquire a buffer and ensure it's locked.
        {
            let buffer = factory.get_any_buffer(10, 4).unwrap();
            let concrete = buffer.as_any().downcast_ref::<LockedBuffer>().unwrap();

            assert!(concrete.buffer.as_ref().locked.load(Ordering::Acquire));
        } // _buffer is dropped here, so the buffer should be unlocked

        // Ensure the buffer is unlocked after being dropped.
        assert!(!factory.buffers[0].locked.load(Ordering::Acquire));
    }

    #[test]
    fn write_to_buffer() {
        let mut factory = DefaultBufferFactory::new();
        let mut buffer = factory.get_any_buffer(10, 4).unwrap();
        let data = vec![1u8, 2u8, 3u8];

        buffer.write(&data);

        // Ensure data is written correctly.
        unsafe {
            assert_eq!(*buffer.get_address().sub(3), 1u8);
            assert_eq!(*buffer.get_address().sub(2), 2u8);
            assert_eq!(*buffer.get_address().sub(1), 3u8);
        }
    }

    #[test]
    fn buffer_address_check() {
        let mut factory = DefaultBufferFactory::new();
        let buffer = factory.get_any_buffer(10, 4).unwrap();

        assert!(!buffer.get_address().is_null());
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Buffer overflow")]
    fn buffer_overflow() {
        let mut factory = DefaultBufferFactory::new();
        let mut buffer = factory.get_any_buffer(10, 4).unwrap();
        let data = vec![1u8; 11]; // One byte larger than buffer size

        buffer.write(&data);
    }
}
