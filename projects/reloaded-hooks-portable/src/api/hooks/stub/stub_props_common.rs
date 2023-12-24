extern crate alloc;
use crate::{
    api::{
        buffers::buffer_abstractions::{Buffer, BufferFactory},
        jit::compiler::Jit,
        traits::register_info::RegisterInfo,
    },
    helpers::{
        atomic_write_masked::atomic_write_masked, jit_jump_operation::create_jump_operation,
    },
};
use alloc::format;
use alloc::vec::Vec;
use core::{
    mem::{size_of, transmute},
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
use super::stub_props_4byteins::StubPackedProps;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "powerpc",
    target_arch = "riscv32",
    target_arch = "riscv64"
)))]
use super::stub_props_other::StubPackedProps;

/*
    Memory Layout:
    - StubPackedProps
        - Enabled Flag
        - Offset of Hook Function (Also length of HookFunction/OriginalCode block)
        - Offset of Original Code
    - enabled_code_instructions / disabled_code_instructions
*/

// Implement common methods on StubPackedProps
impl StubPackedProps {
    /// Enables the hook at `stub_address`.
    ///
    /// If the hook is already enabled, this function does nothing.
    /// If the hook is disabled, this function will perform a thread safe enabling of the hook.
    ///
    /// # Arguments
    ///
    /// * `stub_address` - The address of the stub containing the code described by this properties structure.
    pub fn enable<
        TRegister: RegisterInfo + Clone + Default,
        TJit: Jit<TRegister>,
        TBufferFactory: BufferFactory<TBuffer>,
        TBuffer: Buffer,
    >(
        &mut self,
        stub_address: usize,
    ) {
        unsafe {
            if self.is_enabled() {
                return;
            };

            self.swap_hook::<TRegister, TJit, TBufferFactory, TBuffer>(
                self.get_swap_size(),
                stub_address,
            );
            self.set_is_enabled(true);
        }
    }

    /// Disables the hook at `stub_address`.
    ///
    /// If the hook is already disabled, this function does nothing.
    /// If the hook is enabled, this function will perform a thread safe disabling of the hook.
    ///
    /// # Arguments
    ///
    /// * `stub_address` - The address of the stub containing the code described by this properties structure.
    pub fn disable<
        TRegister: RegisterInfo + Clone + Default,
        TJit: Jit<TRegister>,
        TBufferFactory: BufferFactory<TBuffer>,
        TBuffer: Buffer,
    >(
        &mut self,
        stub_address: usize,
    ) {
        unsafe {
            if !self.is_enabled() {
                return;
            };

            self.swap_hook::<TRegister, TJit, TBufferFactory, TBuffer>(
                self.get_swap_size() + self.get_hook_fn_size(),
                stub_address,
            );
            self.set_is_enabled(false);
        }
    }

    pub fn get_swap_buffer<'a>(&self) -> &'a mut [u8] {
        let start_addr = self as *const Self as *const u8;
        let offset = size_of::<Self>();
        unsafe { from_raw_parts_mut(start_addr.add(offset) as *mut u8, self.get_swap_size()) }
    }

    /// Writes the hook to memory, either enabling or disabling it based on the provided parameters.
    unsafe fn swap_hook<
        TRegister: RegisterInfo + Clone + Default,
        TJit: Jit<TRegister>,
        TBufferFactory: BufferFactory<TBuffer>,
        TBuffer: Buffer,
    >(
        &self,
        temp_branch_offset: usize,
        stub_address: usize,
    ) {
        // Backup current code from swap buffer.
        let buf_swap = self.get_swap_buffer();
        let buf_swap_copy = buf_swap.to_vec();

        // Copy current code into swap buffer
        let stub_swap = from_raw_parts_mut(stub_address as *mut u8, self.get_swap_size());
        let buf_swap_str = format!("buf_swap: {:x}\n", buf_swap.as_ptr() as usize);
        libc::printf(transmute(buf_swap_str.as_ptr()));
        let stub_swap_str = format!("stub_swap: {:x}\n", stub_swap.as_ptr() as usize);
        libc::printf(transmute(stub_swap_str.as_ptr()));
        buf_swap.copy_from_slice(stub_swap);

        // JIT temp branch to hook/orig code.
        let mut vec = Vec::<u8>::with_capacity(8);
        _ = create_jump_operation::<TRegister, TJit, TBufferFactory, TBuffer>(
            stub_address,
            true,
            stub_address + temp_branch_offset,
            None,
            &mut vec,
        );
        let branch_opcode = &vec;
        let branch_bytes = branch_opcode.len();

        // Now write the swapped code into the stub.

        // Write the temp branch first, as per docs
        // This also overwrites some extra code afterwards, but that's a-ok for now.
        unsafe {
            atomic_write_masked::<TBuffer>(stub_address, branch_opcode, branch_bytes);
        }

        // Now write the remaining code
        TBuffer::overwrite(stub_address + branch_bytes, &buf_swap_copy[branch_bytes..]);

        // And now re-insert the code we temp overwrote with the branch
        unsafe {
            atomic_write_masked::<TBuffer>(stub_address, &buf_swap_copy, branch_bytes);
        }
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
pub unsafe fn alloc_and_copy_packed_props(data: &Vec<u8>) -> NonNull<StubPackedProps> {
    let size = data.len();
    let ptr = libc::malloc(size) as *mut u8;

    if !ptr.is_null() {
        copy_nonoverlapping(data.as_ptr(), ptr, size);
    }

    NonNull::new(ptr as *mut StubPackedProps).unwrap()
}
