extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::Push;

use crate::jit_common::ARCH_NOT_SUPPORTED;
use crate::{all_registers::AllRegisters, jit_common::convert_error};
use iced_x86::code_asm::dword_ptr;
use iced_x86::code_asm::qword_ptr;
use iced_x86::code_asm::registers as iced_regs;

macro_rules! encode_xmm_push {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 {
            $a.sub(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
            $a.$op(dword_ptr(iced_regs::esp), $reg.$reg_type()?)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 {
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
    } else if push.register.is_64() {
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
