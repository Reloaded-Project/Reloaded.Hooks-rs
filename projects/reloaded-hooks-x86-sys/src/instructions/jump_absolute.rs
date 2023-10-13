extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpAbs};

use crate::{
    all_registers::AllRegisters,
    jit_common::{convert_error, ARCH_NOT_SUPPORTED},
};

pub(crate) fn encode_jump_absolute(
    a: &mut CodeAssembler,
    x: &JumpAbs<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 64 {
        let target_reg = x.scratch_register.as_iced_64()?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = x.scratch_register.as_iced_32()?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }

    Ok(())
}
