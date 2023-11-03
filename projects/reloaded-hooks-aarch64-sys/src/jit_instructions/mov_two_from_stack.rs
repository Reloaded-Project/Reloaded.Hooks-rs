extern crate alloc;

use crate::{all_registers::AllRegisters, instructions::ldp_immediate::LdpImmediate};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDP--Load-Pair-of-Registers-?lang=en
pub fn encode_mov_two_from_stack(
    reg_1: &AllRegisters,
    reg_2: &AllRegisters,
    stack_offset: i32,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let target_size = reg_1.size();

    let rd = reg_1.register_number();
    let rd2 = reg_2.register_number();

    let ldr = if target_size == 8 {
        LdpImmediate::new_mov_from_stack(true, rd as u8, rd2 as u8, stack_offset)?.0
    } else if target_size == 4 {
        LdpImmediate::new_mov_from_stack(false, rd as u8, rd2 as u8, stack_offset)?.0
    } else if target_size == 16 {
        LdpImmediate::new_mov_from_stack_vector(rd as u8, rd2 as u8, stack_offset)?.0
    } else {
        return Err(JitError::InvalidRegister(*reg_1));
    };

    *pc += 4;
    buf.push(ldr.to_le() as i32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::assert_error;
    use crate::jit_instructions::mov_two_from_stack::encode_mov_two_from_stack;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use rstest::rstest;

    #[rstest]
    // Low Reg
    #[case(w0, w1, 8, "e0074129")]
    #[case(x0, x1, 16, "e00741a9")]
    #[case(v0, v1, 32, "e00741ad")]
    // High Reg
    #[case(w28, w29, 8, "fc774129")]
    #[case(x28, x29, 16, "fc7741a9")]
    #[case(v28, v29, 32, "fc7741ad")]
    // Max Range
    #[case(w0, w1, 252, "e0875f29")]
    #[case(x0, x1, 504, "e0875fa9")]
    #[case(v0, v1, 1008, "e0875fad")]
    // Min Range
    #[case(w0, w1, -256, "e0076029")]
    #[case(x0, x1, -512, "e00760a9")]
    #[case(v0, v1, -1024, "e00760ad")]
    fn standard_cases(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] stack_offset: i32,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        assert!(encode_mov_two_from_stack(&reg_1, &reg_2, stack_offset, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }

    #[rstest]
    // Max Offset
    #[case(w0, w1, 256)]
    #[case(x0, x1, 512)]
    #[case(v0, v1, 1024)]
    // Min Offset
    #[case(w28, w29, -260)]
    #[case(x28, x29, -520)]
    #[case(v28, v29, -1040)]
    fn error_on_out_of_range(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] stack_offset: i32,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let result = encode_mov_two_from_stack(&reg_1, &reg_2, stack_offset, &mut pc, &mut buf);
        assert_error!(result, JitError::OperandOutOfRange(_), pc, buf);
    }

    #[rstest]
    // Not Aligned to 4
    #[case(w0, w1, 1)]
    #[case(w0, w1, 2)]
    #[case(w0, w1, 3)]
    // Not Aligned to 8
    #[case(x0, x1, 1)]
    #[case(x0, x1, 2)]
    #[case(x0, x1, 3)]
    #[case(x0, x1, 4)]
    #[case(x0, x1, 5)]
    #[case(x0, x1, 6)]
    #[case(x0, x1, 7)]
    // Not Aligned to 15
    #[case(v0, v1, 1)]
    #[case(v0, v1, 2)]
    #[case(v0, v1, 3)]
    #[case(v0, v1, 4)]
    #[case(v0, v1, 5)]
    #[case(v0, v1, 6)]
    #[case(v0, v1, 7)]
    #[case(v0, v1, 8)]
    #[case(v0, v1, 9)]
    #[case(v0, v1, 10)]
    #[case(v0, v1, 12)]
    #[case(v0, v1, 13)]
    #[case(v0, v1, 14)]
    #[case(v0, v1, 15)]
    fn error_on_stack_misalignment(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] stack_offset: i32,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let result = encode_mov_two_from_stack(&reg_1, &reg_2, stack_offset, &mut pc, &mut buf);
        assert_error!(result, JitError::InvalidOffset(_), pc, buf);

        // Add register size to ensure it works on values above register size
        let result = encode_mov_two_from_stack(
            &reg_1,
            &reg_2,
            stack_offset + reg_1.size() as i32,
            &mut pc,
            &mut buf,
        );
        assert_error!(result, JitError::InvalidOffset(_), pc, buf);
    }
}
