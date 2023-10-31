extern crate alloc;

use crate::{
    all_registers::AllRegisters,
    instructions::ldr_immediate_unsigned_offset::LdrImmediateUnsignedOffset,
};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::MovFromStack};

/// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
pub fn encode_mov_from_stack(
    x: &MovFromStack<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let target_size = x.target.size();

    let rd = x.target.register_number();
    let ldr = if target_size == 8 {
        LdrImmediateUnsignedOffset::new_mov_from_stack(true, rd as u8, x.stack_offset)?.0
    } else if target_size == 4 {
        LdrImmediateUnsignedOffset::new_mov_from_stack(false, rd as u8, x.stack_offset)?.0
    } else if target_size == 16 {
        LdrImmediateUnsignedOffset::new_mov_from_stack_vector(rd as u8, x.stack_offset)?.0
    } else {
        return Err(JitError::InvalidRegister(x.target));
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
    use crate::jit_instructions::mov_from_stack::encode_mov_from_stack;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(w0, 4, "e00740b9")]
    #[case(x0, 8, "e00740f9")]
    #[case(w0, 8, "e00b40b9")]
    #[case(v0, 16, "e007c03d")]
    // Larger Regiser Number
    #[case(w29, 4, "fd0740b9")]
    #[case(x29, 8, "fd0740f9")]
    #[case(w29, 8, "fd0b40b9")]
    #[case(v29, 16, "fd07c03d")]
    // Max Range
    #[case(w0, 16380, "e0ff7fb9")]
    #[case(x0, 32760, "e0ff7ff9")]
    #[case(v0, 65520, "e0ffff3d")]
    fn standard_cases(
        #[case] target: AllRegisters,
        #[case] stack_offset: i32,
        #[case] expected_hex: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovFromStack {
            stack_offset,
            target,
        };

        assert!(encode_mov_from_stack(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }

    #[rstest]
    // Below Min Range
    #[case(w0, -4)]
    #[case(x0, -8)]
    #[case(v0, -16)]
    // Above Max Range
    #[case(w0, 16384)]
    #[case(x0, 32768)]
    #[case(v0, 65536)]
    fn error_on_out_of_range(#[case] target: AllRegisters, #[case] stack_offset: i32) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovFromStack {
            stack_offset,
            target,
        };

        let result = encode_mov_from_stack(&operation, &mut pc, &mut buf);
        assert_error!(result, JitError::OperandOutOfRange(_), pc, buf);
    }

    #[rstest]
    // Not Aligned to 4
    #[case(w0, 1)]
    #[case(w0, 2)]
    #[case(w0, 3)]
    // Not Aligned to 8
    #[case(x0, 1)]
    #[case(x0, 2)]
    #[case(x0, 3)]
    #[case(x0, 4)]
    #[case(x0, 5)]
    #[case(x0, 6)]
    #[case(x0, 7)]
    // Not Aligned to 15
    #[case(v0, 1)]
    #[case(v0, 2)]
    #[case(v0, 3)]
    #[case(v0, 4)]
    #[case(v0, 5)]
    #[case(v0, 6)]
    #[case(v0, 7)]
    #[case(v0, 8)]
    #[case(v0, 9)]
    #[case(v0, 10)]
    #[case(v0, 12)]
    #[case(v0, 13)]
    #[case(v0, 14)]
    #[case(v0, 15)]
    fn error_on_wrong_stack_alignment(#[case] target: AllRegisters, #[case] stack_offset: i32) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mut operation = MovFromStack {
            stack_offset,
            target,
        };

        let result = encode_mov_from_stack(&operation, &mut pc, &mut buf);
        assert_error!(result, JitError::InvalidOffset(_), pc, buf);

        // Add register size to ensure it works on values above register size
        operation.stack_offset += target.size() as i32;
        let result = encode_mov_from_stack(&operation, &mut pc, &mut buf);
        assert_error!(result, JitError::InvalidOffset(_), pc, buf);
    }
}
