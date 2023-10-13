extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::Push;

use crate::jit_common::ARCH_NOT_SUPPORTED;
use crate::{all_registers::AllRegisters, jit_common::convert_error};
use iced_x86::code_asm::dword_ptr;
use iced_x86::code_asm::qword_ptr;
use iced_x86::code_asm::registers as iced_regs;

macro_rules! multi_push_item {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        match $a.bitness() {
            32 => {
                $a.$op(dword_ptr(iced_regs::esp) + $offset, $reg.$convert_method()?)
                    .map_err(convert_error)?;
            }
            64 => {
                $a.$op(qword_ptr(iced_regs::rsp) + $offset, $reg.$convert_method()?)
                    .map_err(convert_error)?;
            }
            _ => {
                return Err(JitError::ThirdPartyAssemblerError(
                    ARCH_NOT_SUPPORTED.to_string(),
                ));
            }
        }
    };
}

pub(crate) fn encode_multi_push(
    a: &mut CodeAssembler,
    ops: &[Push<AllRegisters>],
) -> Result<(), JitError<AllRegisters>> {
    // Calculate space to reserve.
    let mut space_needed = 0;

    for x in ops {
        space_needed += x.register.size();
    }

    // Reserve the space.
    a.sub(iced_regs::esp, space_needed as i32)
        .map_err(convert_error)?;

    // Push the items.
    let mut current_offset = 0;
    for x in ops.iter().rev() {
        // Loop through the operations in reverse
        if x.register.is_32() {
            multi_push_item!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() {
            multi_push_item!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            multi_push_item!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            multi_push_item!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            multi_push_item!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    Ok(())
}
