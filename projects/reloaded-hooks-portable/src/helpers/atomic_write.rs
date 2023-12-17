use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

/// Performs an atomic write of value in `src` to `tgt`.
/// Size must be 1/2/4/8 bytes.
///
/// # Safety
///
/// Function assumes that `src` and `tgt` are valid pointers to a memory location with at least `size` bytes.
#[inline(always)]
pub unsafe fn atomic_write(src: *const u8, tgt: *mut u8, size: usize) {
    match size {
        1 => {
            let atomic = (tgt as *mut AtomicU8).as_ref().unwrap_unchecked();
            atomic.store(*src, Ordering::Relaxed);
        }
        2 => {
            let atomic = (tgt as *mut AtomicU16).as_ref().unwrap_unchecked();
            atomic.store(*(src as *const u16), Ordering::Relaxed);
        }
        4 => {
            let atomic = (tgt as *mut AtomicU32).as_ref().unwrap_unchecked();
            atomic.store(*(src as *const u32), Ordering::Relaxed);
        }
        8 => {
            let atomic = (tgt as *mut AtomicU64).as_ref().unwrap_unchecked();
            atomic.store(*(src as *const u64), Ordering::Relaxed);
        }
        _ => panic!("Unsupported size for atomic write."),
    }
}
