extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpAbsInd};

pub(crate) fn encode_jump_absolute_indirect(
    a: &mut CodeAssembler,
    x: &JumpAbsInd<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    let mem_op = if a.bitness() == 64 {
        qword_ptr(x.pointer_address)
    } else {
        dword_ptr(x.pointer_address)
    };

    a.jmp(mem_op).map_err(convert_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{x64::jit::JitX64, x86::jit::JitX86};
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(0x12345678, "ff242578563412")]
    fn encode_jump_absolute_indirect_x64(
        #[case] pointer_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX64 {};
        let jump_op = JumpAbsInd::new(pointer_address);

        let operations = vec![Op::JumpAbsoluteIndirect(jump_op)];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x12345678, "ff2578563412")]
    fn encode_jump_absolute_indirect_x86(
        #[case] pointer_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX86 {};
        let jump_op = JumpAbsInd::new(pointer_address);

        let operations = vec![Op::JumpAbsoluteIndirect(jump_op)];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
