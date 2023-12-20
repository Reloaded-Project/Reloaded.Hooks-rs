extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, jump_relative_operation::JumpRelativeOperation,
};

pub fn encode_jump_relative<T>(
    x: &JumpRelativeOperation<T>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<T>> {
    // Calculate the offset from the end of the jump instruction
    let offset_short = (x.target_address as isize).wrapping_sub((*pc as isize).wrapping_add(2)); // 2 bytes for short jump
    let offset_near = (x.target_address as isize).wrapping_sub((*pc as isize).wrapping_add(5)); // 5 bytes for near jump

    if offset_short >= i8::MIN as isize && offset_short <= i8::MAX as isize {
        // Short jump with 8-bit offset
        buf.push(0xEB);
        buf.push(offset_short as u8);
        *pc = pc.wrapping_add(2);
    } else if offset_near >= i32::MIN as isize && offset_near <= i32::MAX as isize {
        // Near jump with 32-bit offset
        buf.push(0xE9);
        buf.extend_from_slice(&(offset_near as i32).to_le_bytes());
        *pc = pc.wrapping_add(5);
    } else {
        // Offset is too large for a relative jump
        return Err(JitError::OperandOutOfRange(
            "Jump offset out of range".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut pc = pc;
        let mut buf = Vec::new();
        let operation = JumpRelativeOperation::<Register>::new(offset);

        encode_jump_relative(&operation, &mut pc, &mut buf).unwrap();
        assert_eq!(expected_encoded, hex::encode(buf));
    }
}
