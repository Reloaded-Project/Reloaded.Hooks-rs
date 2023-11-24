extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{convert_error, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, registers as iced_regs, CodeAssembler};
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::Push;

macro_rules! encode_xmm_push {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 && cfg!(feature = "x86") {
            $a.sub(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
            $a.$op(dword_ptr(iced_regs::esp), $reg.$reg_type()?)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 && cfg!(feature = "x64") {
            $a.sub(iced_regs::rsp, $reg.size() as i32)
                .map_err(convert_error)?;
            $a.$op(qword_ptr(iced_regs::rsp), $reg.$reg_type()?)
                .map_err(convert_error)?;
        } else {
            return Err(JitError::ThirdPartyAssemblerError(
                ARCH_NOT_SUPPORTED.to_string(),
            ));
        }
    };
}

pub(crate) fn encode_push(
    a: &mut CodeAssembler,
    push: &Push<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if push.register.is_32() {
        a.push(push.register.as_iced_32()?).map_err(convert_error)?;
    } else if push.register.is_64() && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        a.push(push.register.as_iced_64()?).map_err(convert_error)?;
    } else if push.register.is_xmm() {
        encode_xmm_push!(a, push.register, as_iced_xmm, movdqu);
    } else if push.register.is_ymm() {
        encode_xmm_push!(a, push.register, as_iced_ymm, vmovdqu);
    } else if push.register.is_zmm() {
        encode_xmm_push!(a, push.register, as_iced_zmm, vmovdqu8);
    } else {
        return Err(JitError::InvalidRegister(push.register));
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
    #[case(x64::Register::rax, "50")]
    #[case(x64::Register::xmm0, "4883ec10f30f7f0424")]
    #[case(x64::Register::ymm0, "4883ec20c5fe7f0424")]
    #[case(x64::Register::zmm0, "4883ec4062f17f487f0424")]
    fn push_x64(#[case] register: x64::Register, #[case] expected_encoded: &str) {
        let operations = vec![Op::Push(Push::new(register))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x86::Register::eax, "50")]
    #[case(x86::Register::xmm0, "83ec10f30f7f0424")]
    #[case(x86::Register::ymm0, "83ec20c5fe7f0424")]
    #[case(x86::Register::zmm0, "83ec4062f17f487f0424")]
    fn push_x86(#[case] register: x86::Register, #[case] expected_encoded: &str) {
        let operations = vec![Op::Push(Push::new(register))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
