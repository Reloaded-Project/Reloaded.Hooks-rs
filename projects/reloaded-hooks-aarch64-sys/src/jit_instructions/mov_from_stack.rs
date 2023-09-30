use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::MovFromStack};
extern crate alloc;
use crate::{
    all_registers::AllRegisters,
    instructions::ldr_immediate_unsigned_offset::LdrImmediateUnsignedOffset,
};
use alloc::vec::Vec;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
pub fn encode_mov_from_stack(
    x: &MovFromStack<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let target_size = x.target.size();

    // TODO: Handle Vector Registers
    let is_64bit = if target_size == 8 {
        true
    } else if target_size == 4 {
        false
    } else if target_size == 16 {
        return encode_mov_from_stack_vector(x, pc, buf);
    } else {
        return Err(JitError::InvalidRegister(x.target));
    };

    let rd = x.target.register_number();
    let ldr = LdrImmediateUnsignedOffset::new_mov_from_stack(is_64bit, rd as u8, x.stack_offset)?;

    *pc += 4;
    buf.push(ldr.0.to_le() as i32);

    Ok(())
}

fn encode_mov_from_stack_vector(
    x: &MovFromStack<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let rd = x.target.register_number();
    let ldr = LdrImmediateUnsignedOffset::new_mov_from_stack_vector(rd as u8, x.stack_offset)?;
    *pc += 4;
    buf.push(ldr.0.to_le() as i32);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::mov_from_stack::encode_mov_from_stack;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(w0, 4, "e00740b9", false)]
    #[case(x0, 8, "e00740f9", false)]
    #[case(w0, 8, "e00b40b9", false)]
    // Vector cases
    #[case(v0, 16, "e007c03d", false)]
    #[case(v31, 16, "ff07c03d", false)]
    fn test_encode_mov_from_stack(
        #[case] target: AllRegisters,
        #[case] stack_offset: i32,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovFromStack {
            stack_offset,
            target,
        };

        // Check the size, expect an error if the size is not 4 or 8
        if is_err {
            assert!(encode_mov_from_stack(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_mov_from_stack(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(4, pc);
    }
}
