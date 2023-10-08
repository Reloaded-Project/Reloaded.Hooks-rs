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

    let rm = x.source.register_number();
    let rd = x.target.register_number();

    let orr = if source_size == 8 && target_size == 8 {
        Orr::new_mov(true, rd as u8, rm as u8).0
    } else if source_size == 16 && target_size == 16 {
        OrrVector::new_mov(rd as u8, rm as u8).0
    } else if source_size == 4 && target_size == 4 {
        Orr::new_mov(false, rd as u8, rm as u8).0
    } else {
        return Err(JitError::InvalidRegisterCombination(x.source, x.target));
    };

    *pc += 4;
    buf.push(orr.to_le() as i32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::assert_error;
    use crate::jit_instructions::mov::encode_mov;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(w0, w1, "e003012a")]
    #[case(x0, x1, "e00301aa")]
    #[case(v0, v1, "201ca14e")]
    // High Reg Number
    #[case(w28, w29, "fc031d2a")]
    #[case(x28, x29, "fc031daa")]
    #[case(v28, v29, "bc1fbd4e")]
    fn standard_cases(
        #[case] target: AllRegisters,
        #[case] source: AllRegisters,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Mov { source, target };

        assert!(encode_mov(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }

    #[rstest]
    #[case(w0, x1)]
    #[case(w0, v1)]
    fn error_on_different_register_sizes(
        #[case] target: AllRegisters,
        #[case] source: AllRegisters,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Mov { source, target };

        // If source and target size don't match, expect an error
        let result = encode_mov(&operation, &mut pc, &mut buf);
        assert_error!(result, JitError::InvalidRegisterCombination(_, _), pc, buf);
    }
}
