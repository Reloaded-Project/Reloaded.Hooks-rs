use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Return};

pub(crate) fn encode_return(
    a: &mut CodeAssembler,
    x: &Return,
) -> Result<(), JitError<AllRegisters>> {
    if x.offset == 0 {
        a.ret().map_err(convert_error)
    } else {
        a.ret_1(x.offset as i32).map_err(convert_error)
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
        let mut jit = JitX64 {};

        let operations = vec![Op::Return(Return::new(offset))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0, "c3")]
    #[case(4, "c20400")]
    fn ret_x86(#[case] offset: usize, #[case] expected: &str) {
        let mut jit = JitX86 {};

        let operations = vec![Op::Return(Return::new(offset))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected, hex::encode(result.unwrap()));
    }
}
