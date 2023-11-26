extern crate alloc;

use crate::buffer::StaticLinkedBuffer;
use alloc::boxed::Box;
use alloc::string::String;
use reloaded_hooks_portable::api::buffers::buffer_abstractions::{Buffer, BufferFactory};
use reloaded_memory_buffers::{buffers::Buffers, structs::params::BufferSearchSettings};

struct BuffersFactory {}

impl BufferFactory for BuffersFactory {
    fn get_buffer(
        &mut self,
        size: u32,
        target: usize,
        proximity: usize,
        alignment: u32,
    ) -> Result<Box<dyn Buffer>, String> {
        let buf = Buffers::get_buffer_aligned(
            &BufferSearchSettings::from_proximity(proximity, target, size as usize),
            alignment,
        )
        .map_err(|x| x.to_string())?;

        Ok(Box::new(StaticLinkedBuffer::new(buf)))
    }

    fn get_any_buffer(&mut self, size: u32, alignment: u32) -> Result<Box<dyn Buffer>, String> {
        let buf = Buffers::get_buffer_aligned(
            &BufferSearchSettings {
                min_address: 0,
                max_address: usize::MAX,
                size,
            },
            alignment,
        )
        .map_err(|x| x.to_string())?;

        Ok(Box::new(StaticLinkedBuffer::new(buf)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use reloaded_memory_buffers::structs::internal::LocatorItem;

    #[test]
    fn acquire_and_release_buffer() {
        let mut factory = BuffersFactory {};
        let item: *mut LocatorItem;

        // Acquire a buffer and ensure it's locked.
        {
            let buffer = factory.get_any_buffer(10, 4).unwrap();
            let concrete = *unsafe {
                Box::<StaticLinkedBuffer>::from_raw(Box::into_raw(buffer) as *mut StaticLinkedBuffer)
            };

            unsafe {
                item = concrete.buf.item.get();
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
        let mut factory = BuffersFactory {};
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
        let mut factory = BuffersFactory {};
        let buffer = factory.get_any_buffer(10, 4).unwrap();

        assert!(!buffer.get_address().is_null());
    }
}
