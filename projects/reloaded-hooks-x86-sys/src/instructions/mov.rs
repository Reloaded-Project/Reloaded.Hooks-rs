use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Mov};

pub(crate) fn encode_mov(
    a: &mut CodeAssembler,
    mov: &Mov<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if mov.target.is_32() && mov.source.is_32() {
        a.mov(mov.target.as_iced_32()?, mov.source.as_iced_32()?)
    } else if mov.target.is_64() && mov.source.is_64() && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        {
            a.mov(mov.target.as_iced_64()?, mov.source.as_iced_64()?)
        }
        #[cfg(not(feature = "x64"))]
        {
            Ok(())
        }
    } else if mov.target.is_xmm() && mov.source.is_xmm() {
        a.movaps(mov.target.as_iced_xmm()?, mov.source.as_iced_xmm()?)
    } else if mov.target.is_ymm() && mov.source.is_ymm() {
        a.vmovaps(mov.target.as_iced_ymm()?, mov.source.as_iced_ymm()?)
    } else if mov.target.is_zmm() && mov.source.is_zmm() {
        a.vmovaps(mov.target.as_iced_zmm()?, mov.source.as_iced_zmm()?)
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
    #[case(x64::Register::xmm0, x64::Register::xmm1, "0f28c8")]
    #[case(x64::Register::ymm0, x64::Register::ymm1, "c5fc28c8")]
    #[case(x64::Register::zmm0, x64::Register::zmm1, "62f17c4828c8")]
    fn mov_x64(
        #[case] source: x64::Register,
        #[case] target: x64::Register,
        #[case] expected_encoded: &str,
    ) {
        let operations = vec![Op::Mov(Mov { source, target })];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x86::Register::eax, x86::Register::ebx, "89c3")]
    #[case(x86::Register::xmm0, x86::Register::xmm1, "0f28c8")]
    #[case(x86::Register::ymm0, x86::Register::ymm1, "c5fc28c8")] // Note: Check if AVX is supported for 32-bit in your environment
    #[case(x86::Register::zmm0, x86::Register::zmm1, "62f17c4828c8")]
    fn mov_x86(
        #[case] source: x86::Register,
        #[case] target: x86::Register,
        #[case] expected_encoded: &str,
    ) {
        let operations = vec![Op::Mov(Mov { source, target })];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
