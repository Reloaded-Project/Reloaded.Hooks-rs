extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{X86jitError, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::PushConst};

pub(crate) fn encode_push_constant(
    a: &mut CodeAssembler,
    x: &PushConst<AllRegisters>,
) -> Result<(), X86jitError<AllRegisters>> {
    if a.bitness() == 32 && cfg!(feature = "x86") {
        a.push(x.value as i32)?;
        Ok(())
    } else if a.bitness() == 64 && cfg!(feature = "x64") {
        a.push(((x.value as u64 >> 32) & 0xFFFFFFFF) as i32)?;
        a.push((x.value & 0xFFFFFFFF) as i32)?;
        Ok(())
    } else {
        return Err(JitError::ThirdPartyAssemblerError(ARCH_NOT_SUPPORTED.to_string()).into());
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_pointer_width = "64")]
    use crate::x64::jit::JitX64;
    use crate::x86::jit::JitX86;
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[cfg(target_pointer_width = "64")]
    #[case(0x11111111EFEFEFEF, "681111111168efefefef")]
    fn push_constant_x64(#[case] constant: usize, #[case] expected_encoded: &str) {
        let operations = vec![Op::PushConst(PushConst::new(constant, None))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x87654321, "6821436587")]
    fn push_constant_x86(#[case] constant: usize, #[case] expected_encoded: &str) {
        let operations = vec![Op::PushConst(PushConst::new(constant, None))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
