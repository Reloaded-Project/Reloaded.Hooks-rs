extern crate alloc;

use crate::{all_registers::AllRegisters, common::jit_common::X86jitError};
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpAbsInd};

pub(crate) fn encode_jump_absolute_indirect(
    a: &mut CodeAssembler,
    x: &JumpAbsInd<AllRegisters>,
) -> Result<(), X86jitError<AllRegisters>> {
    let mem_op = if a.bitness() == 64 && cfg!(feature = "x64") {
        qword_ptr(x.pointer_address)
    } else if cfg!(feature = "x86") {
        dword_ptr(x.pointer_address)
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            "Please use 'x86' or 'x64' library feature".to_string(),
        )
        .into());
    };

    a.jmp(mem_op)?;
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
        let jump_op = JumpAbsInd::new(pointer_address);

        let operations = vec![Op::JumpAbsoluteIndirect(jump_op)];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x12345678, "ff2578563412")]
    fn encode_jump_absolute_indirect_x86(
        #[case] pointer_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let jump_op = JumpAbsInd::new(pointer_address);

        let operations = vec![Op::JumpAbsoluteIndirect(jump_op)];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
