extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Pop};

use crate::{
    all_registers::AllRegisters,
    jit_common::{convert_error, ARCH_NOT_SUPPORTED},
};

use iced_x86::code_asm::registers as iced_regs;

macro_rules! encode_xmm_pop {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 {
            $a.$op($reg.$reg_type()?, dword_ptr(iced_regs::esp))
                .map_err(convert_error)?;
            $a.add(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 {
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
    } else if pop.register.is_64() {
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
