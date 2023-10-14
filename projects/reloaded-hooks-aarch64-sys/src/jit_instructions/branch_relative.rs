use reloaded_hooks_portable::api::jit::{
    call_relative_operation::CallRelativeOperation, compiler::JitError,
    jump_relative_operation::JumpRelativeOperation,
};
extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::instructions::branch_register::BranchRegister;
use alloc::string::ToString;
use alloc::vec::Vec;

use super::load_pc_relative_address::load_pc_rel_address;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/BL--Branch-with-Link-
pub fn encode_call_relative(
    x: &CallRelativeOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    {
        let offset = (x.target_address as i32 - *pc as i32) >> 2;

        if !(-0x02000000..=0x01FFFFFF).contains(&offset) {
            return Err(JitError::OperandOutOfRange(
                "Jump distance for Branch Instruction specified too great".to_string(),
            ));
        }

        let imm26 = offset & 0x03FFFFFF;
        let instruction = 0b100101 << 26 | imm26;
        *pc += 4;
        buf.push(instruction.to_le());
        Ok(())
    }
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/B--Branch-
pub fn encode_jump_relative(
    x: &JumpRelativeOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let offset = ((x.target_address as isize - *pc as isize) >> 2) as i32;

    if !(-0x02000000..=0x01FFFFFF).contains(&offset) {
        if !(-0x40000000..=0x3FFFFFFF).contains(&offset) {
            return Err(JitError::OperandOutOfRange(
                "Jump distance for Branch Instruction specified too great".to_string(),
            ));
        }

        return encode_jump_relative_4g(x, pc, buf);
    }

    let imm26 = offset & 0x03FFFFFF;
    let instruction = 0b000101 << 26 | imm26;
    *pc += 4;
    buf.push(instruction.to_le());
    Ok(())
}

fn encode_jump_relative_4g(
    x: &JumpRelativeOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    load_pc_rel_address(&x.scratch_register, pc, buf, x.target_address)?;

    let op = BranchRegister::new_br(x.scratch_register.register_number() as u8);
    buf.push(op.0.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters::*;
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
        let operation = JumpRel {
            target_address,
            scratch_register: x0,
        };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 1024 * 1024 * 4096)] // Invalid forward jump
    #[case((1024 * 1024 * 4096) + 1, 0)] // Invalid backward jump
    fn can_encode_jump_relative_out_of_range(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRel {
            target_address,
            scratch_register: x0,
        };

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
    #[case(0, 0x8100000, "0008049000001fd6")] // jump forward, no extra offset
    #[case(0x8100000, 0, "00f8fb9000001fd6")] // jump backward, no extra offset
    #[case(0, 0x8100004, "000804900010009100001fd6")] // jump forward, no small extra offset
    #[case(0x8100004, 0, "00f8fb9000001fd6")] // jump backward, no small extra offset
    fn can_encode_jump_relative_4g(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        let operation = JumpRel {
            target_address,
            scratch_register: x0,
        };

        assert!(encode_jump_relative(&operation, &mut pc, &mut buf).is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
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
