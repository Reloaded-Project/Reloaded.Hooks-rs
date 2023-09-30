use reloaded_hooks_portable::api::jit::{compiler::JitError, mov_operation::MovOperation};
extern crate alloc;
use crate::{
    all_registers::AllRegisters,
    instructions::{orr::Orr, orr_vector::OrrVector},
};
use alloc::vec::Vec;

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/MOV--register---Move--register---an-alias-of-ORR--shifted-register--
pub fn encode_mov(
    x: &MovOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let source_size = x.source.size();
    let target_size = x.target.size();

    // TODO: Handle Vector Registers
    // a.k.a. `sf` flag
    let is_64bit = if source_size == 8 && target_size == 8 {
        true
    } else if source_size == 16 && target_size == 16 {
        return encode_mov_vector(x, pc, buf);
    } else if source_size == 4 && target_size == 4 {
        false
    } else {
        return Err(JitError::InvalidRegisterCombination(x.source, x.target));
    };

    let rm = x.source.register_number();
    let rd = x.target.register_number();
    let orr = Orr::new_mov(is_64bit, rd as u8, rm as u8);

    *pc += 4;
    buf.push(orr.0.to_le() as i32);

    Ok(())
}

/// # Remarks
///
/// Part of encode_mov, assumes validation already done.
fn encode_mov_vector(
    x: &MovOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Note: Validation was already done
    let rm = x.source.register_number();
    let rd = x.target.register_number();
    let orr = OrrVector::new_mov(rd as u8, rm as u8);

    *pc += 4;
    buf.push(orr.0.to_le() as i32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::mov::encode_mov;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(w0, w1, 4, 4, "e003012a")]
    #[case(x0, x1, 8, 8, "e00301aa")]
    #[case(w0, x1, 4, 8, "fail")] // should fail

    // Vector operations
    #[case(v0, v1, 16, 16, "201ca14e")]
    fn test_encode_mov(
        #[case] target: AllRegisters,
        #[case] source: AllRegisters,
        #[case] source_size: usize,
        #[case] target_size: usize,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Mov { source, target };

        // If source and target size don't match, expect an error
        if source_size != target_size {
            assert!(encode_mov(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        assert!(encode_mov(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(4, pc);
    }
}
