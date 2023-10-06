use reloaded_hooks_portable::api::jit::call_absolute_operation::CallAbsoluteOperation;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::instructions::branch_register::BranchRegister;

use alloc::vec::Vec;

use super::push_constant::encode_mov_constant_to_reg;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/BR--Branch-to-Register-?lang=en
pub fn encode_jump_absolute(
    x: &JumpAbsoluteOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let register_number = x.scratch_register.register_number() as u8;
    encode_mov_constant_to_reg(x.target_address, register_number, pc, buf)?;

    let op = BranchRegister::new_br(register_number);
    buf.push(op.0.to_le() as i32);
    *pc += 4;
    Ok(())
}

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/BLR--Branch-with-Link-to-Register-?lang=en
pub fn encode_call_absolute(
    x: &CallAbsoluteOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let register_number = x.scratch_register.register_number() as u8;
    encode_mov_constant_to_reg(x.target_address, register_number, pc, buf)?;

    let op = BranchRegister::new_blr(register_number);
    buf.push(op.0.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::branch_absolute::encode_call_absolute;
    use crate::jit_instructions::branch_absolute::encode_jump_absolute;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(x0, 8, 0x1234, "804682d200003fd6")] // 16-bit address
    #[case(x0, 12, 0x12345678, "00cf8ad28046a2f200003fd6")] // 32-bit address
    #[case(x0, 16, 0x123456789ABC, "805793d200cfaaf28046c2f200003fd6")] // 48-bit address
    #[case(x0, 20, 0x123456789ABCDEF0, "00de9bd28057b3f200cfcaf28046e2f200003fd6")] // 64-bit address
    fn test_encode_call_absolute(
        #[case] scratch_register: AllRegisters,
        #[case] expected_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = CallAbs {
            target_address,
            scratch_register,
        };

        assert!(encode_call_absolute(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }

    #[rstest]
    #[case(x0, 8, 0x1234, "804682d200001fd6")] // 16-bit address
    #[case(x0, 12, 0x12345678, "00cf8ad28046a2f200001fd6")] // 32-bit address
    #[case(x0, 16, 0x123456789ABC, "805793d200cfaaf28046c2f200001fd6")] // 48-bit address
    #[case(x0, 20, 0x123456789ABCDEF0, "00de9bd28057b3f200cfcaf28046e2f200001fd6")] // 64-bit address
    fn test_encode_jump_absolute(
        #[case] scratch_register: AllRegisters,
        #[case] expected_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = JumpAbs {
            target_address,
            scratch_register,
        };

        assert!(encode_jump_absolute(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }
}
