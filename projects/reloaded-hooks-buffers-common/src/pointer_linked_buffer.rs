use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;
use reloaded_memory_buffers::structs::internal::LocatorItem;

use crate::dynamic_link::pointer_functions::LOCATORITEM_APPEND_BYTES;

pub(crate) struct PointerBuffer {
    pub(crate) buf: *mut LocatorItem,
}

impl PointerBuffer {
    pub fn new(buf: *mut LocatorItem) -> Self {
        Self { buf }
    }
}

impl Buffer for PointerBuffer {
    fn get_address(&self) -> *const u8 {
        unsafe {
            let item = self.buf;
            ((*item).base_address.value as *const u8).add((*item).position as usize)
        }
    }

    fn write(&mut self, buffer: &[u8]) -> *const u8 {
        unsafe { LOCATORITEM_APPEND_BYTES(self.buf, buffer.as_ptr(), buffer.len()) as *const u8 }
    }
}

impl Drop for PointerBuffer {
    fn drop(&mut self) {
        unsafe {
            (*self.buf).unlock();
        }
    }
}
