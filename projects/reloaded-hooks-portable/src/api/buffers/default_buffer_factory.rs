extern crate alloc;
use super::buffer_abstractions::BufferFactory;
use super::default_buffer::{AllocatedBuffer, LockedBuffer};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use mmap_rs_with_map_from_existing::UnsafeMmapFlags;
use spin::RwLock;

pub(crate) static BUFFERS: BuffersWrapper = BuffersWrapper {
    buffers: RwLock::new(Vec::new()),
};

pub struct BuffersWrapper {
    pub buffers: RwLock<Vec<Rc<AllocatedBuffer>>>,
}

// Safety: DefaultBufferFactory is thread safe.
// RwLock on buffers ensures that only one thread can update the vector at a given time.
// The buffers use `compare_exchange` for availability, ensuring thread safety in grabbing them.
// The Rc is irrelevant because the buffer can only be held by 1 thread due to `compare_exchange`.
unsafe impl Send for BuffersWrapper {}
unsafe impl Sync for BuffersWrapper {}

pub struct DefaultBufferFactory {}

// Safety: DefaultBufferFactory is thread safe.
// RwLock on buffers ensures that only one thread can update the vector at a given time.
// The buffers use `compare_exchange` for availability, ensuring thread safety in grabbing them.
// The Rc is irrelevant because the buffer can only be held by 1 thread due to `compare_exchange`.
unsafe impl Send for DefaultBufferFactory {}
unsafe impl Sync for DefaultBufferFactory {}

impl BufferFactory<LockedBuffer> for DefaultBufferFactory {
    fn get_buffer(
        _size: u32,
        _target: usize,
        _proximity: usize,
        _alignment: u32,
    ) -> Result<Box<LockedBuffer>, String> {
        Err("Not Supported".to_string())
    }

    fn get_any_buffer(size: u32, alignment: u32) -> Result<Box<LockedBuffer>, String> {
        let read_lock = BUFFERS.buffers.read();
        for buffer in read_lock.iter() {
            // Try to lock the buffer temporarily, to ensure thread safety.
            if buffer
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                == Ok(false)
            {
                let current_address =
                    (buffer.ptr.as_ptr() as usize) + *buffer.write_offset.borrow() as usize;
                let adjustment = align_offset(current_address, alignment as usize);
                let aligned_offset = *buffer.write_offset.borrow() + adjustment as u32;
                let new_bytes_remaining = buffer.size - aligned_offset;

                if new_bytes_remaining >= size {
                    // Adjust the write_offset of buffer to ensure alignment
                    *buffer.write_offset.borrow_mut() += adjustment as u32;

                    return Ok(Box::new(LockedBuffer {
                        buffer: buffer.clone(),
                    }));
                } else {
                    // Buffer is not eligible, unlock it
                    buffer.locked.store(false, Ordering::Release);
                }
            }
        }

        drop(read_lock);

        // If no buffer was found, create a new one
        let mut write_lock = BUFFERS.buffers.write();

        let mut map = if create_page_as_rx() {
            mmap_rs_with_map_from_existing::MmapOptions::new(size as usize)
                .unwrap()
                .map_mut()
                .unwrap()
        } else {
            unsafe {
                mmap_rs_with_map_from_existing::MmapOptions::new(size as usize)
                    .unwrap()
                    .with_unsafe_flags(UnsafeMmapFlags::JIT)
                    .map_exec_mut()
                    .unwrap()
            }
        };

        // Don't drop the map!
        let ptr = map.as_mut_ptr();
        mem::forget(map);

        let buffer = Rc::new(AllocatedBuffer {
            ptr: NonNull::new(ptr).unwrap(),
            write_offset: RefCell::new(0),
            size,
            locked: AtomicBool::new(true),
        });

        write_lock.push(buffer.clone());
        Ok(Box::new(LockedBuffer {
            buffer: buffer.clone(),
        }))
    }
}

/// Returns the required number of bytes to align 'address' to 'alignment'.
fn align_offset(address: usize, alignment: usize) -> usize {
    (alignment - (address % alignment)) % alignment
}

/// Returns true if the platform should crate memory pages as R^X instead of RWX.
/// Use this when platform enforces strict W^X policy. This is intended to be used when testing new platforms.
fn create_page_as_rx() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        true
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::buffers::buffer_abstractions::Buffer;
    use alloc::vec;

    #[test]
    fn acquire_and_release_buffer() {
        // Acquire a buffer and ensure it's locked.
        {
            let locked = DefaultBufferFactory::get_any_buffer(10, 4).unwrap();
            assert!(locked.buffer.locked.load(Ordering::Acquire));
        } // _buffer is dropped here, so the buffer should be unlocked

        // Ensure the buffer is unlocked after being dropped.

        // Disabled because buffer becomes available for other tests, and this assert can't be done in a thread safe way.
        // assert!(!BUFFERS.buffers.read()[0].locked.load(Ordering::Acquire));
    }

    #[test]
    fn write_to_buffer() {
        let mut buffer = DefaultBufferFactory::get_any_buffer(10, 4).unwrap();
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
    fn advance_default_buffer() {
        let mut buffer = DefaultBufferFactory::get_any_buffer(10, 4).unwrap();

        let old_position = buffer.get_address();
        let new_position = buffer.advance(2);

        // Ensure position is advanced correctly.
        assert_eq!(buffer.get_address(), new_position);
        assert_eq!(buffer.get_address(), old_position.wrapping_add(2));
    }

    #[test]
    fn buffer_address_check() {
        let buffer = DefaultBufferFactory::get_any_buffer(10, 4).unwrap();

        assert!(!buffer.get_address().is_null());
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Buffer overflow")]
    fn buffer_overflow() {
        let mut buffer = DefaultBufferFactory::get_any_buffer(10, 4).unwrap();
        let data = vec![1u8; 11]; // One byte larger than buffer size

        buffer.write(&data);
    }
}
