use reloaded_hooks_portable::api::jit::compiler::JitError;
extern crate alloc;
use crate::{all_registers::AllRegisters, instructions::ldp_immediate::LdpImmediate};
use alloc::vec::Vec;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
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
    use crate::jit_instructions::mov_two_from_stack::encode_mov_two_from_stack;
    use crate::test_helpers::instruction_buffer_as_hex;
    use rstest::rstest;

    #[rstest]
    #[case(w0, w1, 8, "e0074129", false)]
    #[case(x0, x1, 16, "e00741a9", false)]
    #[case(v0, v1, 32, "e00741ad", false)]
    fn test_encode_mov_two_from_stack(
        #[case] reg_1: AllRegisters,
        #[case] reg_2: AllRegisters,
        #[case] stack_offset: i32,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        // Check the size, expect an error if the size is not 4 or 8
        if is_err {
            assert!(
                encode_mov_two_from_stack(&reg_1, &reg_2, stack_offset, &mut pc, &mut buf).is_err()
            );
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_mov_two_from_stack(&reg_1, &reg_2, stack_offset, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(4, pc);
    }
}
