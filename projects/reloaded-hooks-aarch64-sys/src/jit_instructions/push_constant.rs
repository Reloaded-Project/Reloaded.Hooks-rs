extern crate alloc;

use super::push::encode_push;
use crate::{all_registers::AllRegisters, instructions::mov_immediate::MovImmediate};
use alloc::{string::ToString, vec::Vec};
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, operation_aliases::PushConst, push_operation::PushOperation,
};

/// Encoded as MOVK/MOVZ + STR
pub fn encode_push_constant(
    x: &PushConst<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    if x.scratch.is_none() {
        return Err(JitError::NoScratchRegister("for PushConstant.".to_string()));
    }

    unsafe {
        let scratch = x.scratch.unwrap_unchecked();
        encode_mov_constant_to_reg(x.value, scratch.register_number() as u8, pc, buf)?;
        encode_push(&PushOperation::new(scratch), pc, buf)?;
    }
    Ok(())
}

/// Encoded as MOVK/MOVZ + STR
pub fn encode_mov_constant_to_reg(
    value: usize,
    destination: u8,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Determine leading zeroes using native lzcnt instruction
    let leading_zeros = value.leading_zeros();
    let used_bits = usize::BITS - leading_zeros;

    match used_bits {
        0..=16 => {
            let op = MovImmediate::new_movz(true, destination, value as u16, 0)?;
            buf.push(op.0.to_le() as i32);
            *pc += 4;
        }
        17..=32 => {
            let op = MovImmediate::new_movz(true, destination, value as u16, 0)?;
            let op2 = MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)?;
            buf.push(op.0.to_le() as i32);
            buf.push(op2.0.to_le() as i32);
            *pc += 8;
        }
        33..=48 => {
            let op = MovImmediate::new_movz(true, destination, value as u16, 0)?;
            let op2 = MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)?;
            let op3 = MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)?;
            buf.push(op.0.to_le() as i32);
            buf.push(op2.0.to_le() as i32);
            buf.push(op3.0.to_le() as i32);
            *pc += 12;
        }
        49..=64 => {
            let op = MovImmediate::new_movz(true, destination, value as u16, 0)?;
            let op2 = MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)?;
            let op3 = MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)?;
            let op4 = MovImmediate::new_movk(true, destination, (value >> 48) as u16, 48)?;
            buf.push(op.0.to_le() as i32);
            buf.push(op2.0.to_le() as i32);
            buf.push(op3.0.to_le() as i32);
            buf.push(op4.0.to_le() as i32);
            *pc += 16;
        }
        _ => unreachable!(), // This case should never be reached unless platform is >64 bits
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::push_constant::encode_push_constant;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(x0, 0x1234, "804682d2e08f1ff8")]
    #[case(x0, 0x12345678, "00cf8ad28046a2f2e08f1ff8")]
    #[case(x0, 0x1234567890AB, "601592d200cfaaf28046c2f2e08f1ff8")]
    #[case(x0, 0x1234567890ABCDEF, "e0bd99d26015b2f200cfcaf28046e2f2e08f1ff8")]
    fn standard_cases(
        #[case] register: AllRegisters,
        #[case] constant_to_push: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = PushConst::new(constant_to_push, Some(register));

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_push_constant(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
