use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Return};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_return(
    a: &mut CodeAssembler,
    x: &Return,
) -> Result<(), JitError<AllRegisters>> {
    if x.offset == 0 {
        a.ret().map_err(convert_error)
    } else {
        a.ret_1(x.offset as i32).map_err(convert_error)
    }
}
