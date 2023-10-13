use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::CallRel};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_call_relative(
    a: &mut CodeAssembler,
    x: &CallRel,
) -> Result<(), JitError<AllRegisters>> {
    a.call(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}
