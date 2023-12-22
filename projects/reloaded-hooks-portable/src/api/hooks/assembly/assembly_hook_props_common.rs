extern crate alloc;
use alloc::vec::Vec;
use core::{
    mem::size_of,
    ptr::{copy_nonoverlapping, NonNull},
    slice::from_raw_parts_mut,
};

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
))]
use super::assembly_hook_props_4byteins::AssemblyHookPackedProps;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
use super::assembly_hook_props_other::AssemblyHookPackedProps;

/*
    Memory Layout:
    - AssemblyHookPackedProps
        - Enabled Flag
        - Offset of Hook Function (Also length of HookFunction/OriginalCode block)
        - Offset of Original Code
    - enabled_code_instructions / disabled_code_instructions
*/

// Implement common methods on AssemblyHookPackedProps
impl AssemblyHookPackedProps {
    pub fn get_swap_buffer<'a>(&self) -> &'a mut [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>();
        unsafe { from_raw_parts_mut(start_addr.add(offset) as *mut u8, self.get_swap_size()) }
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
