extern crate alloc;
use alloc::vec::Vec;
use core::{
    mem::size_of,
    ptr::{copy_nonoverlapping, NonNull},
    slice::from_raw_parts,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use super::assembly_hook_props_x86::AssemblyHookPackedProps;

#[cfg(target_arch = "aarch64")]
use super::assembly_hook_props_4byteins::AssemblyHookPackedProps;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
use super::assembly_hook_props_unknown::AssemblyHookPackedProps;

/*
    Memory Layout:
    - AssemblyHookPackedProps
    - disabled_code_instructions
    - enabled_code_instructions
    - branch_to_orig_instructions
    - branch_to_hook_instructions
*/

// Implement common methods on AssemblyHookPackedProps
impl AssemblyHookPackedProps {
    pub fn get_disabled_code<'a>(&self) -> &'a [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>();
        unsafe { from_raw_parts(start_addr.add(offset), self.get_disabled_code_len()) }
    }

    pub fn get_enabled_code<'a>(&self) -> &'a [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>() + self.get_disabled_code_len();
        unsafe { from_raw_parts(start_addr.add(offset), self.get_enabled_code_len()) }
    }

    pub fn get_branch_to_orig_slice<'a>(&self) -> &'a [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>() + self.get_disabled_code_len() + self.get_enabled_code_len();
        unsafe { from_raw_parts(start_addr.add(offset), self.get_branch_to_hook_len()) }
    }

    pub fn get_branch_to_hook_slice<'a>(&self) -> &'a [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>()
            + self.get_disabled_code_len()
            + self.get_enabled_code_len()
            + self.get_branch_to_orig_len();
        unsafe { from_raw_parts(start_addr.add(offset), self.get_branch_to_orig_len()) }
    }

    /// Frees the memory allocated for this instance using libc's free.
    /// # Safety
    ///
    /// It's safe.
    pub unsafe fn free(&mut self) {
        libc::free(self as *mut Self as *mut libc::c_void);
    }
}

/// Allocates memory and copies data from a Vec<u8> into it.
///
/// # Arguments
///
/// * `data` - The data to be copied into the allocated memory.
///
/// # Returns
///
/// A pointer to the newly allocated memory containing the copied data.
///
/// # Safety
///
/// The caller is responsible for ensuring that the allocated memory is freed
/// when no longer needed. This function uses `libc::malloc`, so the memory
/// must be freed with `libc::free`.
pub unsafe fn alloc_and_copy_packed_props(data: &Vec<u8>) -> NonNull<AssemblyHookPackedProps> {
    let size = data.len();
    let ptr = libc::malloc(size) as *mut u8;

    if !ptr.is_null() {
        copy_nonoverlapping(data.as_ptr(), ptr, size);
    }

    NonNull::new(ptr as *mut AssemblyHookPackedProps).unwrap()
}
