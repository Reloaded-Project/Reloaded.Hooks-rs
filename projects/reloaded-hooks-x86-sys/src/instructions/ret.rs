use crate::{all_registers::AllRegisters, common::jit_common::X86jitError};
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::operation_aliases::Return;

pub(crate) fn encode_return(
    a: &mut CodeAssembler,
    x: &Return,
) -> Result<(), X86jitError<AllRegisters>> {
    if x.offset == 0 {
        a.ret()?;
        Ok(())
    } else {
        a.ret_1(x.offset as i32)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::x64::jit::JitX64;
    use crate::x86::jit::JitX86;
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(0, "c3")]
    #[case(4, "c20400")]
    fn ret_x64(#[case] offset: usize, #[case] expected: &str) {
        let operations = vec![Op::Return(Return::new(offset))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0, "c3")]
    #[case(4, "c20400")]
    fn ret_x86(#[case] offset: usize, #[case] expected: &str) {
        let operations = vec![Op::Return(Return::new(offset))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected, hex::encode(result.unwrap()));
    }
}
