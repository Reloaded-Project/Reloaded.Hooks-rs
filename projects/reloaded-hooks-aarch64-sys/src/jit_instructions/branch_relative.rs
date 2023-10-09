use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation, compiler::JitError,
    jump_relative_operation::JumpRelativeOperation,
};
extern crate alloc;
use crate::all_registers::AllRegisters;
use alloc::string::ToString;
use alloc::vec::Vec;

macro_rules! encode_branch_relative {
    ($x:expr, $pc:expr, $buf:expr, $opcode:expr) => {{
        // Branch uses number of 4 byte instructions to jump, so we divide by 4.
        // i.e. value of 1 jumps 4 bytes.
        let offset = ($x.target_address as i32 - *$pc as i32) >> 2;

        if !(-0x02000000..=0x01FFFFFF).contains(&offset) {
            return Err(JitError::OperandOutOfRange(
                "Jump distance for Branch Instruction specified too great".to_string(),
            ));
        }

        // Convert the 32-bit offset into a 26-bit offset by shifting right 6 bits
        let imm26 = offset & 0x03FFFFFF;

        // Create the instruction encoding for the B instruction
        let instruction = $opcode << 26 | imm26;

        *$pc += 4;
        $buf.push(instruction.to_le());

        Ok(())
    }};
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/BL--Branch-with-Link-
pub fn encode_call_relative(
    x: &CallRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    encode_branch_relative!(x, pc, buf, 0b100101)
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/B--Branch-
pub fn encode_jump_relative(
    x: &JumpRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    encode_branch_relative!(x, pc, buf, 0b000101)
}

#[cfg(test)]
mod tests {
    use crate::assert_error;
    use crate::jit_instructions::branch_relative::encode_call_relative;
    use crate::jit_instructions::branch_relative::encode_jump_relative;
    use crate::test_helpers::assert_encode_with_initial_pc;
    use reloaded_hooks_portable::api::jit::compiler::JitError;

    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 4, "01000014")] // jump forward
    #[case(4, 0, "ffffff17")] // jump backward
    #[case(4, 4, "00000014")] // no jump
    fn can_encode_jump_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRel { target_address };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 1024 * 1024 * 128)] // Invalid forward jump
    #[case(1024 * 1024 * 128 + 1, 0)] // Invalid backward jump
    fn can_encode_jump_relative_out_of_range(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRel { target_address };

        let result = encode_jump_relative(&operation, &mut pc, &mut buf);
        assert_error!(
            result,
            JitError::OperandOutOfRange(_),
            initial_pc,
            0,
            pc,
            &buf
        );
    }

    #[rstest]
    #[case(0, 4, "01000094")] // jump forward
    #[case(4, 0, "ffffff97")] // jump backward
    #[case(4, 4, "00000094")] // no jump (endless loop)
    fn can_encode_call_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = CallRel { target_address };

        assert!(encode_call_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }
}
