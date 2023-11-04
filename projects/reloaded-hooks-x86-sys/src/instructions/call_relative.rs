use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::CallRel};

pub(crate) fn encode_call_relative(
    a: &mut CodeAssembler,
    x: &CallRel,
) -> Result<(), JitError<AllRegisters>> {
    a.call(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{x64::jit::JitX64, x86::jit::JitX86};
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(0x7FFFFFFF, "e8faffff7f")]
    fn call_relative_x64(#[case] offset: usize, #[case] expected_encoded: &str) {
        let mut jit = JitX64 {};
        let operations = vec![Op::CallRelative(CallRel::new(offset))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x7FFFFFFF, "e8faffff7f")]
    fn call_relative_x86(#[case] offset: usize, #[case] expected_encoded: &str) {
        let mut jit = JitX86 {};
        let operations = vec![Op::CallRelative(CallRel::new(offset))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
