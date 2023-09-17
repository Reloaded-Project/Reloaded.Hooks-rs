use core::{mem::size_of_val, slice};

pub fn instruction_buffer_as_hex(buf: &[i32]) -> String {
    let ptr = buf.as_ptr() as *const u8;
    unsafe {
        let as_u8 = slice::from_raw_parts(ptr, size_of_val(buf));
        hex::encode(as_u8)
    }
}
