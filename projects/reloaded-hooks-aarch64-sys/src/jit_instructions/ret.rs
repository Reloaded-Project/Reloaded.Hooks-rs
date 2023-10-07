use reloaded_hooks_portable::api::jit::{
    compiler::JitError, push_operation::PushOperation, return_operation::ReturnOperation,
    stack_alloc_operation::StackAllocOperation,
};
extern crate alloc;
use crate::{
    all_registers::AllRegisters,
    instructions::{
        branch_register::BranchRegister, str_immediate_pre_indexed::StrImmediatePreIndexed,
    },
};
use alloc::vec::Vec;

use super::stackalloc::encode_stackalloc;

/// Encoded as ADD (if needed) + RET
pub fn encode_return(
    x: &ReturnOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    if x.offset > 0 {
        encode_stackalloc(&StackAllocOperation::new(-(x.offset as i32)), pc, buf)?;
    }

    let op = BranchRegister::new_ret().0;
    buf.push(op.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::jit_instructions::ret::encode_return;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 4, "c0035fd6", false)]
    #[case(4, 8, "ff130091c0035fd6", false)]
    #[case(8, 8, "ff230091c0035fd6", false)]
    fn test_encode_ret(
        #[case] amount: usize,
        #[case] expected_pc: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Return { offset: amount };

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_return(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_return(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }
}
