extern crate alloc;

use crate::{
    all_registers::AllRegisters,
    instructions::{errors::invalid_register_combination, stp_immediate::StpImmediate},
};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;

/// Encoded as STP
/// Pops 2 registers from the stack. Registers should be of the same type (vector, or full size GPR).
pub fn encode_push_two(
    reg_1: &AllRegisters,
    reg_2: &AllRegisters,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = reg_1.size();
    if size != reg_2.size() {
        return Err(invalid_register_combination(*reg_1, *reg_2));
    }

    let ldr = if size == 8 {
        StpImmediate::new_push_registers(true, *reg_1 as u8, *reg_2 as u8, -16)?.0
    } else if size == 4 {
        StpImmediate::new_push_registers(false, *reg_1 as u8, *reg_2 as u8, -8)?.0
    } else if size == 16 {
        StpImmediate::new_push_registers_vector(*reg_1 as u8, *reg_2 as u8, -32)?.0
    } else {
        return Err(JitError::InvalidRegister(*reg_1));
    };

    buf.push(ldr.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::assert_error;
    use crate::jit_instructions::push_two::encode_push_two;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use rstest::rstest;

    #[rstest]
    #[case(x0, x1, "e007bfa9")]
    #[case(w0, w1, "e007bf29")]
    #[case(v0, v1, "e007bfad")]
    fn standard_cases(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        assert!(encode_push_two(&reg_1, &reg_2, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }

    #[rstest]
    #[case(w0, x1)]
    #[case(w0, v1)]
    fn error_on_mismatching_register_sizes(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let result = encode_push_two(&reg_1, &reg_2, &mut pc, &mut buf);
        assert_error!(result, JitError::InvalidRegisterCombination(_, _), pc, buf);
    }
}
