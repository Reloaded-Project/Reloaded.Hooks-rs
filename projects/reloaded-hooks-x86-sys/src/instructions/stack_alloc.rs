extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::common::jit_common::X86jitError;
use crate::common::jit_common::ARCH_NOT_SUPPORTED;
use alloc::string::ToString;
use iced_x86::code_asm::registers as iced_regs;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::StackAlloc;

pub(crate) fn encode_stack_alloc(
    a: &mut CodeAssembler,
    sub: &StackAlloc,
) -> Result<(), X86jitError<AllRegisters>> {
    if a.bitness() == 32 && cfg!(feature = "x86") {
        a.sub(iced_regs::esp, sub.operand)?;
    } else if a.bitness() == 64 && cfg!(feature = "x64") {
        a.sub(iced_regs::rsp, sub.operand)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(ARCH_NOT_SUPPORTED.to_string()).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{x64::jit::JitX64, x86::jit::JitX86};
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(10, "4883ec0a")]
    fn stackalloc_x64(#[case] size: i32, #[case] expected_encoded: &str) {
        let operations = vec![Op::StackAlloc(StackAlloc::new(size))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(10, "83ec0a")]
    fn stackalloc_x86(#[case] size: i32, #[case] expected_encoded: &str) {
        let operations = vec![Op::StackAlloc(StackAlloc::new(size))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
