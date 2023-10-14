extern crate alloc;
use alloc::boxed::Box;
use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;

pub(crate) fn rewrite_code_aarch64(
    _old_address: *const u8,
    _old_address_size: usize,
    _new_address: *const u8,
    _out_address: *mut u8,
    _out_address_size: usize,
    _buf: Box<dyn Buffer>,
) -> i32 {
    todo!()
}
