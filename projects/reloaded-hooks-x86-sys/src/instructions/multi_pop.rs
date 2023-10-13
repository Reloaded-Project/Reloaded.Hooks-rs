extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Pop};

use crate::jit_common::ARCH_NOT_SUPPORTED;
use crate::{all_registers::AllRegisters, jit_common::convert_error};
use iced_x86::code_asm::dword_ptr;
use iced_x86::code_asm::qword_ptr;
use iced_x86::code_asm::registers as iced_regs;

macro_rules! multi_pop_item {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        match $a.bitness() {
            32 => {
                $a.$op($reg.$convert_method()?, dword_ptr(iced_regs::esp) + $offset)
                    .map_err(convert_error)?;
            }
            64 => {
                $a.$op($reg.$convert_method()?, qword_ptr(iced_regs::rsp) + $offset)
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

pub(crate) fn encode_multi_pop(
    a: &mut CodeAssembler,
    ops: &[Pop<AllRegisters>],
) -> Result<(), JitError<AllRegisters>> {
    // Note: It is important that we do MOV in ascending address order, to help CPU caching :wink:

    // Start from the top of the reserved space.
    let mut current_offset = 0;
    for x in ops {
        if x.register.is_32() {
            multi_pop_item!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() {
            multi_pop_item!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    // Release the space.
    let total_space = ops.iter().map(|x| x.register.size()).sum::<usize>();
    a.add(iced_regs::esp, total_space as i32)
        .map_err(convert_error)?;

    Ok(())
}
