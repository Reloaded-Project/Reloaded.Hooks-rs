use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpRel};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_jump_relative(
    a: &mut CodeAssembler,
    x: &JumpRel<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    a.jmp(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}
