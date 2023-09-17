use reloaded_hooks_portable::api::jit::{
    compiler::JitError, push_stack_operation::PushStackOperation,
};
extern crate alloc;
use crate::all_registers::AllRegisters;
use alloc::string::ToString;
use alloc::vec::Vec;

pub fn encode_push_stack(
    x: &PushStackOperation,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    // Validate remaining size is usable.
    if x.item_size % 4 != 0 {
        return Err(JitError::ThirdPartyAssemblerError(
            "ARM64 PushStack Must use Multiple of 4 Sizes".to_string(),
        ));
    }

    let mut remaining_bytes = x.item_size;

    while remaining_bytes > 0 {
        if remaining_bytes >= 16 {
            // Push Multiple Register
            remaining_bytes -= 16;
        } else if remaining_bytes >= 8 {
            // Push Single Register
            remaining_bytes -= 8;
        } else {
            // Push Remaining Multiple of 4
            remaining_bytes -= 4;
        }
    }

    Ok(())
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
