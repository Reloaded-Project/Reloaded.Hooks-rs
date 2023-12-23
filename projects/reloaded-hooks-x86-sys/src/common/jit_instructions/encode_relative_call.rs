extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation, compiler::JitError,
};

pub fn encode_call_relative<T>(
    x: &CallRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<T>> {
    // Calculate the offset from the end of the call instruction
    let offset = (x.target_address as isize).wrapping_sub((*pc as isize).wrapping_add(5)); // 5 bytes for CALL instruction

    if offset >= i32::MIN as isize && offset <= i32::MAX as isize {
        // Near call with 32-bit offset
        buf.push(0xE8);
        buf.extend_from_slice(&(offset as i32).to_le_bytes());
        *pc = pc.wrapping_add(5);
    } else {
        // Offset is too large for a relative call
        return Err(JitError::OperandOutOfRange(
            "Call offset out of range".to_string(),
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
    // Regular relative call
    #[case(0x7FFFFFFF, "e8faffff7f", 0)]
    // Call into low memory, by overflow
    #[case(1, "e8fc0f0000", usize::MAX - 0xFFF)]
    // Call into high memory by underflow
    #[case(usize::MAX - 0xF, "e8ebffffff", 0)]
    // Call into high memory by underflow, max offset
    #[case(usize::MAX - 0x7FFFFFFA, "e800000080", 0)]
    fn call_relative(#[case] offset: usize, #[case] expected_encoded: &str, #[case] pc: usize) {
        let mut pc = pc;
        let mut buf = Vec::new();
        let operation = CallRelativeOperation::new(offset);

        encode_call_relative::<Register>(&operation, &mut pc, &mut buf).unwrap();
        assert_eq!(expected_encoded, hex::encode(buf));
    }
}
