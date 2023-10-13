use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::XChg};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_xchg(
    a: &mut CodeAssembler,
    xchg: &XChg<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if xchg.register1.is_32() && xchg.register2.is_32() {
        a.xchg(xchg.register1.as_iced_32()?, xchg.register2.as_iced_32()?)
    } else if xchg.register1.is_64() && xchg.register2.is_64() {
        a.xchg(xchg.register1.as_iced_64()?, xchg.register2.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(
            xchg.register1,
            xchg.register2,
        ));
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
    #[case(x86::Register::eax, x86::Register::ebx, "87d8")]
    fn test_compile_xchg_x86(
        #[case] register1: x86::Register,
        #[case] register2: x86::Register,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX86 {};
        let operations = vec![Op::Xchg(XChg {
            register1,
            register2,
            scratch: None,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x64::Register::rax, x64::Register::rbx, "4887d8")]
    fn test_compile_xchg_x64(
        #[case] register1: x64::Register,
        #[case] register2: x64::Register,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX64 {};
        let operations = vec![Op::Xchg(XChg {
            register1,
            register2,
            scratch: None,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
