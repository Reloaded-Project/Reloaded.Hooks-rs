use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::{CallIpRel, JumpIpRel};
extern crate alloc;
use crate::instructions::add_immediate::AddImmediate;
use crate::instructions::adr::Adr;
use crate::{all_registers::AllRegisters, instructions::branch_register::BranchRegister};
use alloc::vec::Vec;

use super::load_pc_relative_value::{self, load_pc_rel_value};

pub fn encode_call_ip_relative(
    x: &CallIpRel<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    load_pc_rel_value(x.scratch, pc, buf, x.target_address)?;

    let op = BranchRegister::new_blr(x.scratch.register_number() as u8);
    buf.push(op.0.to_le() as i32);
    *pc += 4;
    Ok(())
}

pub fn encode_jump_ip_relative(
    x: &JumpIpRel<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    load_pc_rel_value(x.scratch, pc, buf, x.target_address)?;

    let op = BranchRegister::new_br(x.scratch.register_number() as u8);
    buf.push(op.0.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::branch_ip_relative::encode_call_ip_relative;
    use crate::jit_instructions::branch_ip_relative::encode_jump_ip_relative;
    use crate::test_helpers::assert_encode_with_initial_pc;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 4, "20000010000040f900001fd6")] // next instruction
    #[case(4, 0, "e0ffff10000040f900001fd6")] // last instruction
    #[case(0, 1048575, "e0ff7f70000040f900001fd6")] // max value
    #[case(1048576, 0, "00008010000040f900001fd6")] // min value
    fn can_encode_jump_ip_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(encode_jump_ip_relative(
            &JumpIpRel {
                scratch: x0,
                target_address
            },
            &mut pc,
            &mut buf
        )
        .is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    #[rstest]
    #[case(0, 4, "20000010000040f900003fd6")] // next instruction
    #[case(4, 0, "e0ffff10000040f900003fd6")] // last instruction
    #[case(0, 1048575, "e0ff7f70000040f900003fd6")] // max value
    #[case(1048576, 0, "00008010000040f900003fd6")] // min value
    fn can_encode_call_ip_relative(
        #[case] initial_pc: usize,
        #[case] target_address: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = initial_pc;
        let mut buf = Vec::new();
        assert!(encode_call_ip_relative(
            &CallIpRel {
                scratch: x0,
                target_address
            },
            &mut pc,
            &mut buf
        )
        .is_ok());
        assert_encode_with_initial_pc(expected_hex, &buf, initial_pc, pc);
    }

    // Note: Remaining cases covered by other tests.
}
