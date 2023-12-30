extern crate alloc;

use alloc::vec::Vec;
use core::mem::{align_of, forget};

pub(crate) fn vec_u8_to_i32(mut buf: Vec<u8>) -> Vec<i32> {
    debug_assert!(buf.len() % 4 == 0, "Vec<u8> length is not a multiple of 4");
    debug_assert!(
        buf.is_empty() || buf.as_ptr() as usize % align_of::<u32>() == 0,
        "Vec<u8> is not properly aligned for i32"
    );

    let len = buf.len() / 4;
    let capacity = buf.capacity() / 4;
    let ptr = buf.as_mut_ptr() as *mut i32;
    forget(buf);
    unsafe { Vec::from_raw_parts(ptr, len, capacity) }
}

pub(crate) fn vec_u8_to_u32(mut buf: Vec<u8>) -> Vec<u32> {
    debug_assert!(buf.len() % 4 == 0, "Vec<u8> length is not a multiple of 4");
    debug_assert!(
        buf.is_empty() || buf.as_ptr() as usize % align_of::<u32>() == 0,
        "Vec<u8> is not properly aligned for u32"
    );

    let len = buf.len() / 4;
    let capacity = buf.capacity() / 4;
    let ptr = buf.as_mut_ptr() as *mut u32;
    forget(buf);
    unsafe { Vec::from_raw_parts(ptr, len, capacity) }
}

pub(crate) fn vec_i32_to_u8(buf_i32: Vec<i32>) -> Vec<u8> {
    let len = buf_i32.len() * 4;
    let capacity = buf_i32.capacity() * 4;
    let ptr = buf_i32.as_ptr() as *mut u8;
    forget(buf_i32);
    unsafe { Vec::from_raw_parts(ptr, len, capacity) }
}

pub(crate) fn vec_u32_to_u8(buf_u32: Vec<u32>) -> Vec<u8> {
    let len = buf_u32.len() * 4;
    let capacity = buf_u32.capacity() * 4;
    let ptr = buf_u32.as_ptr() as *mut u8;
    forget(buf_u32);
    unsafe { Vec::from_raw_parts(ptr, len, capacity) }
}
