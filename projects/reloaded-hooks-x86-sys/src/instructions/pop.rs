extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{convert_error, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, registers as iced_regs, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Pop};

macro_rules! encode_xmm_pop {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 && cfg!(feature = "x86") {
            $a.$op($reg.$reg_type()?, dword_ptr(iced_regs::esp))
                .map_err(convert_error)?;
            $a.add(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 && cfg!(feature = "x64") {
            $a.$op($reg.$reg_type()?, qword_ptr(iced_regs::rsp))
                .map_err(convert_error)?;
            $a.add(iced_regs::rsp, $reg.size() as i32)
                .map_err(convert_error)?;
        } else {
            return Err(JitError::ThirdPartyAssemblerError(
                ARCH_NOT_SUPPORTED.to_string(),
            ));
        }
    };
}

pub(crate) fn encode_pop(
    a: &mut CodeAssembler,
    pop: &Pop<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if pop.register.is_32() {
        a.pop(pop.register.as_iced_32()?).map_err(convert_error)?;
    } else if pop.register.is_64() && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        a.pop(pop.register.as_iced_64()?).map_err(convert_error)?;
    } else if pop.register.is_xmm() {
        encode_xmm_pop!(a, pop.register, as_iced_xmm, movdqu);
    } else if pop.register.is_ymm() {
        encode_xmm_pop!(a, pop.register, as_iced_ymm, vmovdqu);
    } else if pop.register.is_zmm() {
        encode_xmm_pop!(a, pop.register, as_iced_zmm, vmovdqu8);
    } else {
        return Err(JitError::InvalidRegister(pop.register));
    }

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
    #[case(x86::Register::xmm0, "f30f6f042483c410")]
    #[case(x86::Register::ymm0, "c5fe6f042483c420")]
    #[case(x86::Register::zmm0, "62f17f486f042483c440")]
    fn pop_x86(#[case] register: x86::Register, #[case] expected_encoded: &str) {
        let mut jit = JitX86 {};
        let operations = vec![Op::Pop(Pop::new(register))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x64::Register::xmm0, "f30f6f04244883c410")]
    #[case(x64::Register::ymm0, "c5fe6f04244883c420")]
    #[case(x64::Register::zmm0, "62f17f486f04244883c440")]
    fn pop_x64(#[case] register: x64::Register, #[case] expected_encoded: &str) {
        let mut jit = JitX64 {};
        let operations = vec![Op::Pop(Pop::new(register))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
