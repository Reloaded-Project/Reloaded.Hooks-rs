extern crate alloc;

use super::stackalloc::encode_stackalloc;
use crate::{all_registers::AllRegisters, instructions::branch_register::BranchRegister};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, return_operation::ReturnOperation,
    stack_alloc_operation::StackAllocOperation,
};

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
    use crate::test_helpers::assert_encode;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, "c0035fd6")] // without offset
    #[case(4, "ff130091c0035fd6")] // with offset
    #[case(8, "ff230091c0035fd6")]
    fn can_encode_ret(#[case] amount: usize, #[case] expected_hex: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Return { offset: amount };

        assert!(encode_return(&operation, &mut pc, &mut buf).is_ok());
        assert_encode(expected_hex, &buf, pc);
    }
}
