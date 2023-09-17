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

    use crate::jit_instructions::stackalloc::encode_stackalloc;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(4, "ff1300d1", false)]
    #[case(-4, "ff130091", false)]
    #[case(0, "ff0300d1", false)]
    #[case(2048, "ff0320d1", false)]
    #[case(-2048, "ff032091", false)]
    fn test_encode_stackalloc(
        #[case] operand: i32,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = StackAlloc { operand };

        // Check for errors if applicable
        if is_err {
            assert!(encode_stackalloc(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_stackalloc(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));

        // Assert that the program counter has been incremented by 4
        assert_eq!(4, pc);
    }
}
