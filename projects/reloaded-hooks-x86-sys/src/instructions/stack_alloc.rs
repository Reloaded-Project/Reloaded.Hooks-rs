extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::{x64::Register as x64Register, x86::Register as x86Register};
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::operation_aliases::StackAlloc;

#[cfg(feature = "x86")]
pub(crate) fn encode_stack_alloc_32(
    x: &StackAlloc,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    unsafe {
        let len = if x.operand >= -128 && x.operand <= 127 {
            3
        } else {
            6
        };

        let old_len = buf.len();
        buf.reserve(len);
        let ptr = buf.as_mut_ptr().add(old_len);

        if len == 3 {
            // 8-bit immediate
            *(ptr as *mut u16) = 0xEC83_u16.to_le(); // Opcode for SUB ESP, imm8 with ModR/M byte for ESP
            ptr.add(2).write(x.operand as u8); // 8-bit immediate value
        } else {
            // 32-bit immediate
            *(ptr as *mut u16) = 0xEC81_u16.to_le(); // Opcode for SUB ESP, imm32 with ModR/M byte for ESP
            write_unaligned(ptr.add(2) as *mut i32, x.operand.to_le()); // 32-bit immediate value
        }

        buf.set_len(old_len + len);
        *pc += len;
    }
    Ok(())
}

#[cfg(feature = "x64")]
pub(crate) fn encode_stack_alloc_64(
    x: &StackAlloc,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    unsafe {
        let len = if x.operand >= -128 && x.operand <= 127 {
            4
        } else {
            7
        };
        let old_len = buf.len();
        buf.reserve(len);
        let ptr = buf.as_mut_ptr().add(old_len);

        if len == 4 {
            // 8-bit immediate
            ptr.write(0x48); // REX prefix for 64-bit operand size
            write_unaligned(ptr.add(1) as *mut u16, 0xEC83_u16.to_le()); // Opcode for SUB RSP, imm8 with ModR/M byte for RSP
            ptr.add(3).write(x.operand as u8); // 8-bit immediate value
        } else {
            // 32-bit immediate
            ptr.write(0x48); // REX prefix for 64-bit operand size
            write_unaligned(ptr.add(1) as *mut u16, 0xEC81_u16.to_le()); // Opcode for SUB RSP, imm8 with ModR/M byte for RSP
            write_unaligned(ptr.add(3) as *mut i32, x.operand.to_le()); // 32-bit immediate value
        }
        buf.set_len(old_len + len);
        *pc += len;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use rstest::rstest;

    #[rstest]
    #[case(StackAlloc::new(10), "83ec0a", false)] // 32-bit with 8-bit operand
    #[case(StackAlloc::new(0x12345678), "81ec78563412", false)] // 32-bit with 32-bit operand
    #[case(StackAlloc::new(10), "4883ec0a", true)] // 64-bit with 8-bit operand
    #[case(StackAlloc::new(0x12345678), "4881ec78563412", true)] // 64-bit with 32-bit operand
    fn test_encode_stack_alloc(
        #[case] operation: StackAlloc,
        #[case] expected_encoded: &str,
        #[case] is_64bit: bool,
    ) {
        let mut buf = Vec::new();
        let mut pc = 0;
        if is_64bit {
            encode_stack_alloc_64(&operation, &mut pc, &mut buf).unwrap();
        } else {
            encode_stack_alloc_32(&operation, &mut pc, &mut buf).unwrap();
        }
        assert_encode(expected_encoded, &buf, pc);
    }
}
