use reloaded_hooks_portable::api::jit::compiler::JitError;
extern crate alloc;
use crate::{all_registers::AllRegisters, instructions::stp_immediate::StpImmediate};
use alloc::vec::Vec;

/// Encoded as STP
/// Pops 2 registers from the stack. Registers should be of the same type (vector, or full size GPR).
pub fn encode_push_two(
    reg_1: &AllRegisters,
    reg_2: &AllRegisters,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = reg_1.size();
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
    use crate::jit_instructions::push_two::encode_push_two;
    use crate::test_helpers::instruction_buffer_as_hex;
    use rstest::rstest;

    #[rstest]
    #[case(x0, x1, 4, "e007bfa9", false)]
    #[case(w0, w1, 4, "e007bf29", false)]
    #[case(v0, v1, 4, "e007bfad", false)]
    fn test_encode_push_two(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] expected_size: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_push_two(&reg_1, &reg_2, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_push_two(&reg_1, &reg_2, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_size, pc);
    }
}
