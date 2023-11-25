extern crate alloc;

use alloc::string::String;
use core::mem::size_of;
use mach::port::mach_port_name_t;
use mach::traps::*;
use mach::vm::*;
use mach::vm_prot::*;
use mach::vm_region::*;
use mach::vm_types::*;

/// Temporarily disables write XOR execute protection with an OS specialized
/// API call (if available).
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
///
/// # Returns
///
/// - `usize`: The old memory protection (if needed for call to [`self::restore_write_xor_execute`]).
///            
///
/// # Remarks
///
/// This is not currently used on any platform, but is intended for environments
/// which enforce write XOR execute, such as M1 macs.
///
/// The idea is that you use memory which is read_write_execute (MAP_JIT if mmap),
/// then disable W^X for the current thread. Then we write the code, and re-enable W^X.
pub fn disable_write_xor_execute(address: *const u8, size: usize) -> Result<Option<usize>, String> {
    unsafe {
        let mut region_address = address as mach_vm_address_t;
        let mut region_size = size as mach_vm_size_t;
        let mut region_info = core::mem::zeroed::<vm_region_basic_info_data_t>();
        let mut object_name: mach_port_name_t = 0;
        let mut count = (size_of::<vm_region_basic_info_data_t>() / size_of::<integer_t>()) as u32;
        let result = mach_vm_region(
            mach_task_self(),
            &mut region_address,
            &mut region_size,
            VM_REGION_BASIC_INFO,
            &mut region_info as *mut _ as vm_region_info_t,
            &mut count,
            &mut object_name,
        );

        mach_vm_protect(
            mach_task_self(),
            address as u64,
            size as mach_vm_size_t,
            0,
            VM_PROT_READ | VM_PROT_WRITE,
        );

        Ok(Some(region_info.protection as usize))
    }
}

/// Restores write XOR execute protection.
///
/// # Parameters
///
/// - `address`: The address of the memory to disable write XOR execute protection for.
/// - `size`: The size of the memory to disable write XOR execute protection for.
/// - `protection`: The protection returned in the result of the call to [`self::disable_write_xor_execute`].
///
/// # Returns
///
/// Success or error.
pub fn restore_write_xor_execute(
    address: *const u8,
    size: usize,
    protection: usize,
) -> Result<(), String> {
    unsafe {
        mach_vm_protect(
            mach_task_self(),
            address as u64,
            size as mach_vm_size_t,
            0,
            protection as i32,
        );
    }

    Ok(())
}
