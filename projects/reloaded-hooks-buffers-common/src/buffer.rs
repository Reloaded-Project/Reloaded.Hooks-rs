use core::ptr::copy_nonoverlapping;

use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;
use reloaded_memory_buffers::{
    buffers::Buffers, c::buffers_c_buffers::overwrite_allocated_code, structs::SafeLocatorItem,
};

pub(crate) struct StaticLinkedBuffer {
    pub(crate) buf: SafeLocatorItem,
}

impl StaticLinkedBuffer {
    pub fn new(buf: SafeLocatorItem) -> Self {
        Self { buf }
    }
}

impl Buffer for StaticLinkedBuffer {
    fn get_address(&self) -> *const u8 {
        unsafe {
            let item = self.buf.item.get();
            ((*item).base_address.value as *const u8).add((*item).position as usize)
        }
    }

    fn write(&mut self, buffer: &[u8]) -> *const u8 {
        debug_assert!(
            {
                let left = unsafe { (*self.buf.item.get()).bytes_left() };
                left >= buffer.len() as u32
            },
            "Buffer overflow"
        );
        unsafe { self.buf.append_bytes(buffer) as *const u8 }
    }

    fn overwrite(address: usize, buffer: &[u8])
    where
        Self: Sized,
    {
        unsafe {
            Buffers::overwrite_allocated_code(buffer.as_ptr(), address as *mut u8, buffer.len())
        }
    }
}
