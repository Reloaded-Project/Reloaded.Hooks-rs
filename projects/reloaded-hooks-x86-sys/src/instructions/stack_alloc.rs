extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::StackAlloc;

use crate::jit_common::ARCH_NOT_SUPPORTED;
use crate::{all_registers::AllRegisters, jit_common::convert_error};
use iced_x86::code_asm::registers as iced_regs;

pub(crate) fn encode_stack_alloc(
    a: &mut CodeAssembler,
    sub: &StackAlloc,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        a.sub(iced_regs::esp, sub.operand).map_err(convert_error)?;
    } else if a.bitness() == 64 {
        a.sub(iced_regs::rsp, sub.operand).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }

    Ok(())
}
