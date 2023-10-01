use reloaded_hooks_portable::api::jit::{compiler::JitError, push_operation::PushOperation};
extern crate alloc;
use crate::{
    all_registers::AllRegisters, instructions::str_immediate_pre_indexed::StrImmediatePreIndexed,
};
use alloc::vec::Vec;

/// Encoded as STR
pub fn encode_push(
    x: &PushOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = x.register.size();
    let op = if size == 8 {
        StrImmediatePreIndexed::new_push_register(true, x.register as u8, -8)?.0
    } else if size == 4 {
        StrImmediatePreIndexed::new_push_register(false, x.register as u8, -4)?.0
    } else if size == 16 {
        StrImmediatePreIndexed::new_push_vector(x.register as u8, -16)?.0
    } else {
        return Err(JitError::InvalidRegister(x.register));
    };

    buf.push(op.to_le() as i32);
    *pc += 4;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::jit_instructions::push::encode_push;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(x0, 4, "e08f1ff8", false)]
    #[case(w0, 4, "e0cf1fb8", false)]
    // Vector cases
    #[case(v0, 4, "e00f9f3c", false)]
    fn test_encode_push(
        #[case] register: AllRegisters,
        #[case] expected_pc: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Push { register };

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_push(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_push(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_pc, pc);
    }
}
