use reloaded_hooks_portable::api::jit::{compiler::JitError, pop_operation::PopOperation};
extern crate alloc;
use crate::{
    all_registers::AllRegisters, instructions::ldr_immediate_post_indexed::LdrImmediatePostIndexed,
};
use alloc::vec::Vec;

/// Encoded as LDR
pub fn encode_pop(
    x: &PopOperation<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    let size = x.register.size();
    let op = if size == 8 {
        LdrImmediatePostIndexed::new_pop_register(true, x.register as u8, 8)?.0
    } else if size == 4 {
        LdrImmediatePostIndexed::new_pop_register(false, x.register as u8, 4)?.0
    } else if size == 16 {
        LdrImmediatePostIndexed::new_pop_vector(x.register as u8, 16)?.0
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
    use crate::jit_instructions::pop::encode_pop;
    use crate::test_helpers::instruction_buffer_as_hex;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(v0, 4, "e007c13c", false)]
    #[case(x0, 4, "e08740f8", false)]
    #[case(w0, 4, "e04740b8", false)]
    // #[case(v0, 16, "expected_hex_value_for_vector", false)] // if you implement this
    fn test_encode_pop(
        #[case] register: AllRegisters,
        #[case] expected_size: usize,
        #[case] expected_hex: &str,
        #[case] is_err: bool,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Pop { register };

        // Expect an error for invalid register sizes
        if is_err {
            assert!(encode_pop(&operation, &mut pc, &mut buf).is_err());
            return;
        }

        // If the encoding is successful, compare with the expected hex value
        assert!(encode_pop(&operation, &mut pc, &mut buf).is_ok());
        assert_eq!(expected_hex, instruction_buffer_as_hex(&buf));
        assert_eq!(expected_size, pc);
    }
}
