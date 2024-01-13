extern crate alloc;

use crate::{
    common::jit_instructions::helpers::{
        opcode_offset_for_x64_register, opcode_offset_for_x86_register,
    },
    x64, x86,
};
use alloc::vec::Vec;
use core::ptr::write_unaligned;
use reloaded_hooks_portable::api::jit::call_absolute_operation::CallAbsoluteOperation;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// x86 encoding
pub fn encode_absolute_call_x86(
    x: &CallAbsoluteOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<x86::Register>> {
    let opcode_offset = opcode_offset_for_x86_register(x.scratch_register);

    unsafe {
        buf.reserve(7); // Reserve space for the instruction
        let ptr = buf.as_mut_ptr().add(buf.len());

        // MOV instruction (opcode + immediate)
        ptr.write(0xB8 + opcode_offset); // MOV reg, imm32
        write_unaligned(ptr.add(1) as *mut u32, x.target_address as u32); // 32-bit immediate

        // CALL reg (opcode + ModRM)
        write_unaligned(
            ptr.add(5) as *mut u16,
            0xD0FF_u16.to_le() + ((opcode_offset as u16) << 8),
        ); // Opcode for CALL r/m32 + ModRM byte

        buf.set_len(buf.len() + 7);
        *pc += 7;
    }

    Ok(())
}

// x64 encoding
pub fn encode_absolute_call_x64(
    x: &CallAbsoluteOperation<x64::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<x64::Register>> {
    let opcode_offset = opcode_offset_for_x64_register(x.scratch_register);

    unsafe {
        buf.reserve(12); // Reserve space for the instruction
        let ptr = buf.as_mut_ptr().add(buf.len());

        // MOV instruction (REX prefix + opcode + immediate)
        ptr.write(0x48 + ((opcode_offset >= 8) as u8)); // REX prefix for 64-bit operand size
        ptr.add(1).write(0xB8 + (opcode_offset % 8)); // MOV reg, imm64
        write_unaligned(ptr.add(2) as *mut u64, x.target_address as u64); // 64-bit immediate value

        // CALL reg (REX prefix + opcode + ModRM)
        let jmp_start = if opcode_offset >= 8 { 11 } else { 10 };
        if opcode_offset >= 8 {
            ptr.add(10).write(0x41); // REX Prefix for R8-R15
        }
        write_unaligned(
            ptr.add(jmp_start) as *mut u16,
            0xD0FF_u16.to_le() + ((opcode_offset as u16 % 8) << 8),
        ); // Opcode for CALL r/m64

        let len = jmp_start + 2;
        buf.set_len(buf.len() + len);
        *pc += len;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    #[cfg(target_pointer_width = "64")]
    use crate::x64::Register as X64Register;
    use crate::x86::Register as X86Register;
    use rstest::rstest;

    // Test cases for x64 architecture
    #[cfg(target_pointer_width = "64")]
    #[rstest]
    #[case(X64Register::rax, "48b8efbeaddeefbeaddeffd0")]
    #[case(X64Register::rcx, "48b9efbeaddeefbeaddeffd1")]
    #[case(X64Register::rdx, "48baefbeaddeefbeaddeffd2")]
    #[case(X64Register::rbx, "48bbefbeaddeefbeaddeffd3")]
    #[case(X64Register::rsp, "48bcefbeaddeefbeaddeffd4")]
    #[case(X64Register::rbp, "48bdefbeaddeefbeaddeffd5")]
    #[case(X64Register::rsi, "48beefbeaddeefbeaddeffd6")]
    #[case(X64Register::rdi, "48bfefbeaddeefbeaddeffd7")]
    #[case(X64Register::r8, "49b8efbeaddeefbeadde41ffd0")]
    #[case(X64Register::r9, "49b9efbeaddeefbeadde41ffd1")]
    #[case(X64Register::r10, "49baefbeaddeefbeadde41ffd2")]
    #[case(X64Register::r11, "49bbefbeaddeefbeadde41ffd3")]
    #[case(X64Register::r12, "49bcefbeaddeefbeadde41ffd4")]
    #[case(X64Register::r13, "49bdefbeaddeefbeadde41ffd5")]
    #[case(X64Register::r14, "49beefbeaddeefbeadde41ffd6")]
    #[case(X64Register::r15, "49bfefbeaddeefbeadde41ffd7")]
    // Add other cases for different x64 registers here
    fn test_encode_absolute_call_x64(
        #[case] scratch_register: X64Register,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let mut pc = 0;
        let operation = CallAbsoluteOperation {
            scratch_register,
            target_address: 0xDEADBEEFDEADBEEF,
        };

        encode_absolute_call_x64(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    // Test cases for x86 architecture
    #[rstest]
    #[case(X86Register::eax, "b8efbeaddeffd0")]
    #[case(X86Register::ecx, "b9efbeaddeffd1")]
    #[case(X86Register::edx, "baefbeaddeffd2")]
    #[case(X86Register::ebx, "bbefbeaddeffd3")]
    #[case(X86Register::esp, "bcefbeaddeffd4")]
    #[case(X86Register::ebp, "bdefbeaddeffd5")]
    #[case(X86Register::esi, "beefbeaddeffd6")]
    #[case(X86Register::edi, "bfefbeaddeffd7")]
    // Add other cases for different x86 registers here
    fn test_encode_absolute_call_x86(
        #[case] scratch_register: X86Register,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let mut pc = 0;
        let operation = CallAbsoluteOperation {
            scratch_register,
            target_address: 0xDEADBEEF,
        };

        encode_absolute_call_x86(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
