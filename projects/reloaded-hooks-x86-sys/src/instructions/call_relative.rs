extern crate alloc;
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation, compiler::JitError,
};

pub fn encode_call_relative<T>(
    x: &CallRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<T>> {
    let offset = (x.target_address as isize).wrapping_sub((*pc as isize).wrapping_add(5)); // 5 bytes for CALL instruction

    unsafe {
        let len = 5; // Length of CALL instruction with 32-bit offset
        let old_len = buf.len();
        buf.reserve(len);
        let ptr = buf.as_mut_ptr().add(old_len);

        ptr.write(0xE8); // Near call with 32-bit offset
        write_unaligned(ptr.add(1) as *mut i32, offset as i32);

        *pc = pc.wrapping_add(len);
        buf.set_len(old_len + len);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode_with_initial_pc;
    use crate::x86::Register;
    use rstest::rstest;

    #[rstest]
    // Regular relative call
    #[case(0x7FFFFFFF, "e8faffff7f", 0)]
    // Call into low memory, by overflow
    #[case(1, "e8fc0f0000", usize::MAX - 0xFFF)]
    // Call into high memory by underflow
    #[case(usize::MAX - 0xF, "e8ebffffff", 0)]
    // Call into high memory by underflow, max offset
    #[case(usize::MAX - 0x7FFFFFFA, "e800000080", 0)]
    fn call_relative(#[case] offset: usize, #[case] expected_encoded: &str, #[case] pc: usize) {
        let mut new_pc = pc;
        let mut buf = Vec::new();
        let operation = CallRelativeOperation::new(offset);

        encode_call_relative::<Register>(&operation, &mut new_pc, &mut buf).unwrap();
        assert_encode_with_initial_pc(expected_encoded, &buf, pc, new_pc);
    }
}
