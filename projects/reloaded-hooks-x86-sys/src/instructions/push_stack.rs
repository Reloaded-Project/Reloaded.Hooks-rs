extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::{x64::Register as x64Register, x86::Register as x86Register};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::operation_aliases::PushStack;
use zydis::mem;
use zydis::{EncoderRequest, Mnemonic::PUSH};

#[cfg(feature = "x86")]
pub(crate) fn encode_push_stack_x86(
    push: &PushStack<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use core::ptr::write_unaligned;

    const REG_SIZE: u32 = 4;
    const INS_SIZE_U8: usize = 4;
    const INS_SIZE_U32: usize = 7;

    let old_len = buf.len();
    let num_operations = push.item_size / REG_SIZE;

    // Reserve space in the buffer
    if push.offset <= 0x7F && push.offset >= -0x80 {
        let num_bytes = INS_SIZE_U8 * num_operations as usize;
        buf.reserve(num_bytes);

        unsafe {
            let mut ptr = buf.as_mut_ptr().add(old_len);
            for _ in 0..num_operations {
                // Construct the instruction
                let opcode = 0x00_24_74_FF_u32.to_le(); // PUSH [ESP + offset], with placeholder for offset
                let instruction = opcode | ((push.offset as u32 & 0xFF) << 24); // Insert offset into the instruction

                // Write the instruction
                write_unaligned(ptr as *mut u32, instruction.to_le());
                ptr = ptr.wrapping_add(INS_SIZE_U8);
            }

            buf.set_len(old_len + num_bytes);
            *pc += num_bytes;
        }
    } else {
        let num_bytes = INS_SIZE_U32 * num_operations as usize;
        buf.reserve(num_bytes);

        unsafe {
            let mut ptr = buf.as_mut_ptr().add(old_len);
            for _ in 0..num_operations {
                write_unaligned(ptr as *mut u32, 0x00_24_B4_FF_u32.to_le()); // PUSH [ESP], with placeholder for offset
                write_unaligned(ptr.add(3) as *mut u32, (push.offset as u32).to_le()); // offset
                ptr = ptr.add(INS_SIZE_U32);
            }

            buf.set_len(old_len + num_bytes);
            *pc += num_bytes;
        }
    }

    Ok(())
}

// x64 implementation
#[cfg(feature = "x64")]
pub(crate) fn encode_push_stack_x64(
    push: &PushStack<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    const REG_SIZE: u32 = 8;
    let num_operations = push.item_size / REG_SIZE;

    for _ in 0..num_operations {
        *pc += EncoderRequest::new64(PUSH)
            .add_operand(mem!(qword ptr [RSP + (push.offset as i64)]))
            .encode_extend(buf)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::util::test_utilities::assert_encode, x64, x86};
    use rstest::rstest;

    #[rstest]
    #[case(4, 8, "ff742404")]
    #[case(32, 16, "ff742420ff742420")]
    fn push_from_stack_x64(#[case] offset: i32, #[case] size: u32, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let push_stack_operation = PushStack::with_offset_and_size(offset, size);

        encode_push_stack_x64(&push_stack_operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(4, 4, "ff742404")]
    #[case(32, 16, "ff742420ff742420ff742420ff742420")]
    #[case(0x200, 4, "ffb42400020000")]
    #[case(0x200, 8, "ffb42400020000ffb42400020000")]
    fn push_from_stack_x86(#[case] offset: i32, #[case] size: u32, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let push_stack_operation = PushStack::with_offset_and_size(offset, size);

        encode_push_stack_x86(&push_stack_operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
