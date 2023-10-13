extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};

use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::MovFromStack};

use crate::{
    all_registers::AllRegisters,
    jit_common::{convert_error, ARCH_NOT_SUPPORTED},
};

pub(crate) fn encode_mov_from_stack(
    a: &mut CodeAssembler,
    x: &MovFromStack<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        let ptr = dword_ptr(iced_x86::Register::ESP) + x.stack_offset;
        a.mov(x.target.as_iced_32()?, ptr)
    } else if a.bitness() == 64 {
        let ptr = qword_ptr(iced_x86::Register::RSP) + x.stack_offset;
        a.mov(x.target.as_iced_64()?, ptr)
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}
