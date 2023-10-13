extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;

use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::PushConst};

use crate::{
    all_registers::AllRegisters,
    jit_common::{convert_error, ARCH_NOT_SUPPORTED},
};

pub(crate) fn encode_push_constant(
    a: &mut CodeAssembler,
    x: &PushConst<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        a.push(x.value as i32).map_err(convert_error)
    } else if a.bitness() == 64 {
        a.push(((x.value >> 32) & 0xFFFFFFFF) as i32)
            .map_err(convert_error)?;
        a.push((x.value & 0xFFFFFFFF) as i32).map_err(convert_error)
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }
}
