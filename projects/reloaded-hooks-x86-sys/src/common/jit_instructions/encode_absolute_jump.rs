// File: encode_absolute_jump.rs

extern crate alloc;
use super::helpers::{opcode_offset_for_x64_register, opcode_offset_for_x86_register};
use crate::{x64, x86};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, jump_absolute_operation::JumpAbsoluteOperation,
};

// x86 encoding
pub fn encode_absolute_jump_x86(
    x: &JumpAbsoluteOperation<x86::Register>,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<x86::Register>> {
    let opcode_offset = opcode_offset_for_x86_register(x.scratch_register);

    // MOV instruction (opcode + immediate)
    buf.push(0xB8 + opcode_offset); // MOV reg, imm32
    buf.extend_from_slice(&(x.target_address as u32).to_le_bytes()); // 32-bit immediate

    // JMP reg (opcode + ModRM)
    buf.push(0xFF); // Opcode for JMP r/m32
    buf.push(0xE0 + opcode_offset); // ModRM byte for reg

    Ok(())
}

// x64 encoding
pub fn encode_absolute_jump_x64(
    x: &JumpAbsoluteOperation<x64::Register>,
    buf: &mut Vec<u8>,
) -> Result<(), JitError<x64::Register>> {
    let opcode_offset = opcode_offset_for_x64_register(x.scratch_register);

    // MOV instruction (REX prefix + opcode + immediate)
    buf.push(0x48 + ((opcode_offset >= 8) as u8)); // REX prefix for 64-bit operand size
    buf.push(0xB8 + (opcode_offset % 8)); // MOV reg, imm64
    buf.extend_from_slice(&(x.target_address as u64).to_le_bytes()); // 64-bit immediate value for the target address

    // JMP reg (REX prefix + opcode + ModRM)
    if opcode_offset >= 8 {
        buf.push(0x41); // REX Prefix for R8-R15
    }

    buf.push(0xFF); // Opcode for JMP r/m64
    buf.push(0xE0 + (opcode_offset % 8)); // ModRM byte for reg
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_pointer_width = "64")]
    use crate::x64::Register as X64Register;
    use crate::x86::Register as X86Register;
    use rstest::rstest;

    // Test cases for x64 architecture
    #[cfg(target_pointer_width = "64")]
    #[rstest]
    #[case(X64Register::rax, "48b8efbeaddeefbeaddeffe0")]
    #[case(X64Register::rcx, "48b9efbeaddeefbeaddeffe1")]
    #[case(X64Register::rdx, "48baefbeaddeefbeaddeffe2")]
    #[case(X64Register::rbx, "48bbefbeaddeefbeaddeffe3")]
    #[case(X64Register::rsp, "48bcefbeaddeefbeaddeffe4")]
    #[case(X64Register::rbp, "48bdefbeaddeefbeaddeffe5")]
    #[case(X64Register::rsi, "48beefbeaddeefbeaddeffe6")]
    #[case(X64Register::rdi, "48bfefbeaddeefbeaddeffe7")]
    #[case(X64Register::r8, "49b8efbeaddeefbeadde41ffe0")]
    #[case(X64Register::r9, "49b9efbeaddeefbeadde41ffe1")]
    #[case(X64Register::r10, "49baefbeaddeefbeadde41ffe2")]
    #[case(X64Register::r11, "49bbefbeaddeefbeadde41ffe3")]
    #[case(X64Register::r12, "49bcefbeaddeefbeadde41ffe4")]
    #[case(X64Register::r13, "49bdefbeaddeefbeadde41ffe5")]
    #[case(X64Register::r14, "49beefbeaddeefbeadde41ffe6")]
    #[case(X64Register::r15, "49bfefbeaddeefbeadde41ffe7")]
    // Add other cases for different x64 registers here
    fn test_encode_absolute_jump_x64(
        #[case] scratch_register: X64Register,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let operation = JumpAbsoluteOperation {
            scratch_register,
            target_address: 0xDEADBEEFDEADBEEF,
        };

        encode_absolute_jump_x64(&operation, &mut buf).unwrap();
        assert_eq!(expected_encoded, hex::encode(buf));
    }

    // Test cases for x86 architecture
    #[rstest]
    #[case(X86Register::eax, "b8efbeaddeffe0")]
    #[case(X86Register::ecx, "b9efbeaddeffe1")]
    #[case(X86Register::edx, "baefbeaddeffe2")]
    #[case(X86Register::ebx, "bbefbeaddeffe3")]
    #[case(X86Register::esp, "bcefbeaddeffe4")]
    #[case(X86Register::ebp, "bdefbeaddeffe5")]
    #[case(X86Register::esi, "beefbeaddeffe6")]
    #[case(X86Register::edi, "bfefbeaddeffe7")]
    // Add other cases for different x86 registers here
    fn test_encode_absolute_jump_x86(
        #[case] scratch_register: X86Register,
        #[case] expected_encoded: &str,
    ) {
        let mut buf = Vec::new();
        let operation = JumpAbsoluteOperation {
            scratch_register,
            target_address: 0xDEADBEEF,
        };

        encode_absolute_jump_x86(&operation, &mut buf).unwrap();
        assert_eq!(expected_encoded, hex::encode(buf));
    }
}
