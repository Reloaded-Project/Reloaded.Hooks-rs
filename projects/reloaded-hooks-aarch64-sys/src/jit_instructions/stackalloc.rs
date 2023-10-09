use reloaded_hooks_portable::api::jit::{
    compiler::JitError, stack_alloc_operation::StackAllocOperation,
};
extern crate alloc;
use crate::{
    all_registers::AllRegisters,
    instructions::{add_immediate::AddImmediate, sub_immediate::SubImmediate},
};
use alloc::vec::Vec;

/// Encoded as SUB, SP, SP, #operand
/// or ADD, SP, SP, #operand
pub fn encode_stackalloc(
    x: &StackAllocOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    if x.operand >= 0 {
        let sub = SubImmediate::new_stackalloc(true, x.operand as u16)?;

        *pc += 4;
        buf.push(sub.0.to_le() as i32);
        Ok(())
    } else {
        let add = AddImmediate::new_stackalloc(true, -x.operand as u16)?;

        *pc += 4;
        buf.push(add.0.to_le() as i32);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::assert_error;
    use crate::jit_instructions::stackalloc::encode_stackalloc;
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::compiler::JitError;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(-4096)] // Below Min Range
    #[case(4096)] // Above Max Range
    fn error_on_out_of_range(#[case] stack_size: i32) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = StackAlloc::new(stack_size);

        let result = encode_stackalloc(&operation, &mut pc, &mut buf);
        assert_error!(result, JitError::OperandOutOfRange(_), pc, buf);
    }

    #[rstest]
    #[case(4, "ff1300d1")]
    #[case(-4, "ff130091")]
    #[case(0, "ff0300d1")]
    #[case(2048, "ff0320d1")]
    #[case(-2048, "ff032091")]
    // On edge of range
    #[case(4095, "ffff3fd1")]
    #[case(-4095, "ffff3f91")]
    fn standard_cases(#[case] stack_size: i32, #[case] expected_hex: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = StackAlloc::new(stack_size);

        assert!(encode_stackalloc(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
