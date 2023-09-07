use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;
use reloaded_memory_buffers::structs::SafeLocatorItem;

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
}
