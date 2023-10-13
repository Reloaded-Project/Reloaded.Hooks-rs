use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::XChg};

use crate::{all_registers::AllRegisters, jit_common::convert_error};

pub(crate) fn encode_xchg(
    a: &mut CodeAssembler,
    xchg: &XChg<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if xchg.register1.is_32() && xchg.register2.is_32() {
        a.xchg(xchg.register1.as_iced_32()?, xchg.register2.as_iced_32()?)
    } else if xchg.register1.is_64() && xchg.register2.is_64() {
        a.xchg(xchg.register1.as_iced_64()?, xchg.register2.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(
            xchg.register1,
            xchg.register2,
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}
