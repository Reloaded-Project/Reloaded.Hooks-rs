extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{X86jitError, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::PushStack};

macro_rules! encode_push_stack_impl {
    ($a:expr, $push:expr, $reg:expr, $size:expr, $ptr_type:ident, $error_msg:expr) => {
        if $push.item_size != $size {
            // Need to do some custom shenanigans to re-push larger values.
            if $push.item_size % $size != 0 {
                return Err(JitError::ThirdPartyAssemblerError($error_msg.to_string()).into());
            } else {
                let num_operations = $push.item_size / $size;
                for op_idx in 0..num_operations {
                    let ptr = $ptr_type($reg) + $push.offset as i32 + (op_idx * $size * 2);
                    $a.push(ptr)?;
                }
            }
        } else {
            let ptr = $ptr_type($reg) + $push.offset as i32;
            $a.push(ptr)?;
        }
    };
}

pub(crate) fn encode_push_stack(
    a: &mut CodeAssembler,
    push: &PushStack<AllRegisters>,
) -> Result<(), X86jitError<AllRegisters>> {
    match a.bitness() {
        #[cfg(feature = "x86")]
        32 => {
            // This could be faster for 32-bit; using SSE registers to re-push 4 params at once
            // Only problem is, there is no common callee saved register for SSE on 32-bit,
            let error_msg =
                "Stack parameter must be a multiple of 4 if not a single register size.";
            encode_push_stack_impl!(a, push, iced_x86::Register::ESP, 4, dword_ptr, error_msg);
        }
        #[cfg(feature = "x64")]
        64 => {
            let error_msg =
                "Stack parameter must be a multiple of 8 if not a single register size.";
            encode_push_stack_impl!(a, push, iced_x86::Register::RSP, 8, qword_ptr, error_msg);
        }
        _ => {
            return Err(JitError::ThirdPartyAssemblerError(ARCH_NOT_SUPPORTED.to_string()).into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{x64::jit::JitX64, x86::jit::JitX86};
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(4, 8, "ff742404")]
    #[case(32, 16, "ff742420ff742430")]
    fn push_from_stack_x64(#[case] offset: i32, #[case] size: u32, #[case] expected_encoded: &str) {
        let operations = vec![Op::PushStack(PushStack::with_offset_and_size(offset, size))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(4, 4, "ff742404")]
    #[case(32, 16, "ff742420ff742428ff742430ff742438")]
    fn push_from_stack_x86(#[case] offset: i32, #[case] size: u32, #[case] expected_encoded: &str) {
        let operations = vec![Op::PushStack(PushStack::with_offset_and_size(offset, size))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
