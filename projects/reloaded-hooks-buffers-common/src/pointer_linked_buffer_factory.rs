extern crate alloc;

use core::ffi::CStr;

use alloc::boxed::Box;
use alloc::string::String;

use reloaded_hooks_portable::api::buffers::buffer_abstractions::{Buffer, BufferFactory};
use reloaded_memory_buffers::structs::params::BufferSearchSettings;

use crate::{dynamic_link::pointer_functions::*, pointer_linked_buffer::PointerBuffer};

struct PointerBuffersFactory {}

impl BufferFactory for PointerBuffersFactory {
    fn get_buffer(
        &mut self,
        size: u32,
        target: usize,
        proximity: usize,
        alignment: u32,
    ) -> Result<Box<dyn Buffer>, String> {
        let buf = unsafe {
            BUFFERS_GET_BUFFER_ALIGNED(
                &BufferSearchSettings::from_proximity(proximity, target, size as usize),
                alignment,
            )
        };

        if buf.is_ok {
            Ok(Box::new(PointerBuffer::new(buf.ok)))
        } else {
            let err_text = unsafe { CStr::from_ptr(buf.err).to_string_lossy().into_owned() };
            let err = Err(err_text);
            unsafe { FREE_GET_BUFFER_RESULT(buf) }
            err
        }
    }

    fn get_any_buffer(&mut self, size: u32, alignment: u32) -> Result<Box<dyn Buffer>, String> {
        let buf = unsafe {
            BUFFERS_GET_BUFFER_ALIGNED(
                &BufferSearchSettings {
                    min_address: 0,
                    max_address: usize::MAX,
                    size,
                },
                alignment,
            )
        };

        if buf.is_ok {
            Ok(Box::new(PointerBuffer::new(buf.ok)))
        } else {
            let err_text = unsafe { CStr::from_ptr(buf.err).to_string_lossy().into_owned() };
            let err = Err(err_text);
            unsafe { FREE_GET_BUFFER_RESULT(buf) }
            err
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use reloaded_memory_buffers::structs::internal::LocatorItem;

    #[test]
    fn acquire_and_release_buffer() {
        let mut factory = PointerBuffersFactory {};
        let item: *mut LocatorItem;

        // Acquire a buffer and ensure it's locked.
        {
            let buffer = factory.get_any_buffer(10, 4).unwrap();
            let concrete = *unsafe {
                Box::<PointerBuffer>::from_raw(Box::into_raw(buffer) as *mut PointerBuffer)
            };

            unsafe {
                item = concrete.buf;
                assert!((*item).is_taken());
            }
        } // _buffer is dropped here, so the buffer should be unlocked

        // Ensure the buffer is unlocked after being dropped.
        unsafe {
            assert!(!(*item).is_taken());
        }
    }

    #[test]
    fn write_to_buffer() {
        let mut factory = PointerBuffersFactory {};
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
        let mut factory = PointerBuffersFactory {};
        let buffer = factory.get_any_buffer(10, 4).unwrap();

        assert!(!buffer.get_address().is_null());
    }
}
