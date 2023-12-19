use core::mem::size_of;
use reloaded_hooks_portable::{
    api::buffers::buffer_abstractions::Buffer, helpers::atomic_write::atomic_write,
};
use reloaded_memory_buffers::{buffers::Buffers, structs::SafeLocatorItem};

pub struct StaticLinkedBuffer {
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

    fn overwrite_atomic<TInteger>(address: usize, buffer: TInteger)
    where
        Self: Sized,
    {
        Buffers::overwrite_allocated_code_ex(
            (&buffer) as *const TInteger as *const u8,
            address as *mut u8,
            size_of::<TInteger>(),
            atom_write,
        )
    }

    fn advance(&mut self, num_bytes: usize) -> *const u8 {
        let locator_item = self.buf.item.get();
        let current_offset = unsafe { (*locator_item).position };
        unsafe {
            (*locator_item).position = current_offset + num_bytes as u32;
            ((*locator_item).base_address.value as *const u8).add((*locator_item).position as usize)
        }
    }
}

fn atom_write(src: *const u8, tgt: *mut u8, size: usize) {
    unsafe { atomic_write(src, tgt, size) }
}
