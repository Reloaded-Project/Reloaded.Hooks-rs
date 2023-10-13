use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Mov};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_mov(
    a: &mut CodeAssembler,
    mov: &Mov<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if mov.target.is_32() && mov.source.is_32() {
        a.mov(mov.target.as_iced_32()?, mov.source.as_iced_32()?)
    } else if mov.target.is_64() && mov.source.is_64() {
        a.mov(mov.target.as_iced_64()?, mov.source.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(mov.source, mov.target));
    }
    .map_err(convert_error)?;

    Ok(())
}
