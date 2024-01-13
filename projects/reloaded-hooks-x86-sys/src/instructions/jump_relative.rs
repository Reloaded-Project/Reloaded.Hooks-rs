extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, jump_relative_operation::JumpRelativeOperation,
};

pub fn encode_jump_relative<T>(
    x: &JumpRelativeOperation<T>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<T>> {
    let offset_short = (x.target_address as isize)
        .wrapping_sub((*pc as isize).wrapping_add(2))
        .to_le(); // 2 bytes for short jump
    let offset_near = (x.target_address as isize)
        .wrapping_sub((*pc as isize).wrapping_add(5))
        .to_le(); // 5 bytes for near jump

    unsafe {
        let len = if offset_short >= i8::MIN as isize && offset_short <= i8::MAX as isize {
            2
        } else if offset_near >= i32::MIN as isize && offset_near <= i32::MAX as isize {
            5
        } else {
            return throw_out_of_range::<T>();
        };

        let old_len = buf.len();
        buf.reserve(len);
        let ptr = buf.as_mut_ptr().add(old_len);

        if len == 2 {
            ptr.write(0xEB); // Short jump with 8-bit offset
            ptr.add(1).write(offset_short as u8);
            *pc = pc.wrapping_add(len);
        } else {
            ptr.write(0xE9); // Near jump with 32-bit offset
            write_unaligned(ptr.add(1) as *mut i32, offset_near as i32);
            *pc = pc.wrapping_add(len);
        }
        buf.set_len(old_len + len);
    }

    Ok(())
}

#[cold]
fn throw_out_of_range<T>() -> Result<(), JitError<T>> {
    Err(JitError::OperandOutOfRange(
        "Jump offset out of range".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode_with_initial_pc;
    use crate::x86::Register;
    use rstest::rstest;

    #[rstest]
    // Regular relative jump
    #[case(0x7FFFFFFF, "e9faffff7f", 0)]
    // Jump into low memory, by overflow
    #[case(1, "e9fc0f0000", usize::MAX - 0xFFF)]
    // Jump into high memory by underflow
    #[case(usize::MAX - 0xF, "ebee", 0)]
    // Jump into high memory by underflow, max offset
    #[case(usize::MAX - 0x7FFFFFFA, "e900000080", 0)]
    fn jmp_relative(#[case] offset: usize, #[case] expected_encoded: &str, #[case] pc: usize) {
        let mut new_pc = pc;
        let mut buf = Vec::new();
        let operation = JumpRelativeOperation::<Register>::new(offset);

        encode_jump_relative(&operation, &mut new_pc, &mut buf).unwrap();
        assert_encode_with_initial_pc(expected_encoded, &buf, pc, new_pc);
    }
}
