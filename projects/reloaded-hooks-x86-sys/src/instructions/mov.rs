use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Mov};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_mov(
    a: &mut CodeAssembler,
    mov: &Mov<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if mov.target.is_32() && mov.source.is_32() {
        a.mov(mov.target.as_iced_32()?, mov.source.as_iced_32()?)
    } else if mov.target.is_64() && mov.source.is_64() {
        a.mov(mov.target.as_iced_64()?, mov.source.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(mov.source, mov.target));
    }
    .map_err(convert_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        x64::{self, jit::JitX64},
        x86::{self, jit::JitX86},
    };
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(x64::Register::rax, x64::Register::rbx, "4889c3")]
    fn mov_x64(
        #[case] source: x64::Register,
        #[case] target: x64::Register,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX64 {};
        let operations = vec![Op::Mov(Mov { source, target })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x86::Register::eax, x86::Register::ebx, "89c3")]
    fn mov_x86(
        #[case] source: x86::Register,
        #[case] target: x86::Register,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX86 {};
        let operations = vec![Op::Mov(Mov { source, target })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
