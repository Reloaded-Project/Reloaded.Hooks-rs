extern crate alloc;

use crate::buffer::StaticLinkedBuffer;
use alloc::boxed::Box;
use alloc::string::String;
#[allow(unused_imports)]
use reloaded_hooks_portable::api::buffers::buffer_abstractions::{Buffer, BufferFactory};
use reloaded_memory_buffers::{buffers::Buffers, structs::params::BufferSearchSettings};

pub struct BuffersFactory {}

impl BufferFactory<StaticLinkedBuffer> for BuffersFactory {
    fn get_buffer(
        size: u32,
        target: usize,
        proximity: usize,
        alignment: u32,
    ) -> Result<Box<StaticLinkedBuffer>, String> {
        let buf = Buffers::get_buffer_aligned(
            &BufferSearchSettings::from_proximity(proximity, target, size as usize),
            alignment,
        )
        .map_err(|x| x.to_string())?;

        Ok(Box::new(StaticLinkedBuffer::new(buf)))
    }

    fn get_any_buffer(size: u32, alignment: u32) -> Result<Box<StaticLinkedBuffer>, String> {
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
        let item: *mut LocatorItem;

        // Acquire a buffer and ensure it's locked.
        {
            let buffer = BuffersFactory::get_any_buffer(10, 4).unwrap();
            unsafe {
                item = buffer.buf.item.get();
                assert!((*item).is_taken());
            }
        } // _buffer is dropped here, so the buffer should be unlocked

        // Ensure the buffer is unlocked after being dropped.
        // Disabled because buffer becomes available for other tests, and this assert can't be done in a thread safe way.
        /*
        unsafe {
            assert!(!(*item).is_taken());
        }
        */
    }

    #[test]
    fn write_to_buffer() {
        let mut buffer = BuffersFactory::get_any_buffer(10, 4).unwrap();
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
    fn advance_buffer() {
        let mut buffer = BuffersFactory::get_any_buffer(10, 4).unwrap();

        let old_position = buffer.get_address();
        let new_position = buffer.advance(2);

        // Ensure position is advanced correctly.
        assert_eq!(buffer.get_address(), new_position);
        assert_eq!(buffer.get_address(), old_position.wrapping_add(2));
    }

    #[test]
    fn buffer_address_check() {
        let buffer = BuffersFactory::get_any_buffer(10, 4).unwrap();

        assert!(!buffer.get_address().is_null());
    }
}
