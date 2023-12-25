/*
!! IMPORTANT NOTE !!

atomic_write_masked uses this file under the hood!!

If you need to change this file to implement a new architecture, thus changing the maximum
width of atomic write you MUST also override atomic_write_masked.MAX_ATOMIC_WRITE_BYTES
for your architecture!!

I am putting this not here, because you will likely run into compiler error, thus need to edit this file.
*/
use core::{
    ptr::read_unaligned,
    sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering},
};
use portable_atomic::AtomicU128;

use crate::api::buffers::buffer_abstractions::Buffer;

/// Performs an atomic write of value in `src` to `tgt`.
/// Size must be 1/2/4/8/16 bytes.
///
/// # Safety
///
/// Function assumes that `src` and `tgt` are valid pointers to a memory location with at least `size` bytes.
#[inline(always)]
pub unsafe fn atomic_write(src: *const u8, tgt: *mut u8, size: usize) {
    // Read the note at the top of this file before changing this function.

    match size {
        1 => {
            let atomic = (tgt as *mut AtomicU8).as_ref().unwrap_unchecked();
            atomic.store(*src, Ordering::Relaxed);
        }
        2 => {
            let atomic = (tgt as *mut AtomicU16).as_ref().unwrap_unchecked();
            atomic.store(read_unaligned(src as *const u16), Ordering::Relaxed);
        }
        4 => {
            let atomic = (tgt as *mut AtomicU32).as_ref().unwrap_unchecked();
            atomic.store(read_unaligned(src as *const u32), Ordering::Relaxed);
        }
        8 => {
            let atomic = (tgt as *mut AtomicU64).as_ref().unwrap_unchecked();
            atomic.store(read_unaligned(src as *const u64), Ordering::Relaxed);
        }
        16 => {
            let atomic = (tgt as *mut AtomicU128).as_ref().unwrap_unchecked();
            atomic.store(read_unaligned(src as *const u128), Ordering::Relaxed);
        }
        _ => panic!("Unsupported size for atomic write."),
    }
}

/// Performs an atomic swap of values between `src` and `tgt`.
/// Where 'tgt' is a memory location inside an executable buffer.
///
/// Size must be 1/2/4/8/16 bytes.
///
/// # Parameters
///
/// * `src` - The source memory location to swap with `tgt`.
/// * `buf_tgt` - The target memory location to swap with `src`. Must belong to TBuffer.
/// * `size` - The size of the swap. Must be 1/2/4/8/16 bytes.
///
/// # Safety
///
/// Function assumes that `src` and `tgt` are valid pointers to a memory location with at least `size` bytes.
#[inline(always)]
pub unsafe fn atomic_swap<TBuffer: Buffer>(src: *mut u8, buf_tgt: *mut u8, size: usize) {
    match size {
        1 => {
            let src_atomic = (src as *mut AtomicU8).as_ref().unwrap_unchecked();
            let src_val = src_atomic.load(Ordering::Relaxed);
            src_atomic.store(read_unaligned(buf_tgt as *const u8), Ordering::Relaxed);
            TBuffer::overwrite_atomic(buf_tgt as usize, src_val);
        }
        2 => {
            let src_atomic = (src as *mut AtomicU16).as_ref().unwrap_unchecked();
            let src_val = src_atomic.load(Ordering::Relaxed);
            src_atomic.store(read_unaligned(buf_tgt as *const u16), Ordering::Relaxed);
            TBuffer::overwrite_atomic(buf_tgt as usize, src_val);
        }
        4 => {
            let src_atomic = (src as *mut AtomicU32).as_ref().unwrap_unchecked();
            let src_val = src_atomic.load(Ordering::Relaxed);
            src_atomic.store(read_unaligned(buf_tgt as *const u32), Ordering::Relaxed);
            TBuffer::overwrite_atomic(buf_tgt as usize, src_val);
        }
        8 => {
            let src_atomic = (src as *mut AtomicU64).as_ref().unwrap_unchecked();
            let src_val = src_atomic.load(Ordering::Relaxed);
            src_atomic.store(read_unaligned(buf_tgt as *const u64), Ordering::Relaxed);
            TBuffer::overwrite_atomic(buf_tgt as usize, src_val);
        }
        16 => {
            let src_atomic = (src as *mut AtomicU128).as_ref().unwrap_unchecked();
            let src_val = src_atomic.load(Ordering::Relaxed);
            src_atomic.store(read_unaligned(buf_tgt as *const u128), Ordering::Relaxed);
            TBuffer::overwrite_atomic(buf_tgt as usize, src_val);
        }
        _ => panic!("Unsupported size for atomic swap."),
    }
}
