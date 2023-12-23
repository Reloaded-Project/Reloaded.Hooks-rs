use crate::{x64, x86};
use core::hint::unreachable_unchecked;

// Lookup table for x86 registers to opcode offset
pub(crate) fn opcode_offset_for_x86_register(register: x86::Register) -> u8 {
    match register {
        x86::Register::eax => 0,
        x86::Register::ecx => 1,
        x86::Register::edx => 2,
        x86::Register::ebx => 3,
        x86::Register::esp => 4,
        x86::Register::ebp => 5,
        x86::Register::esi => 6,
        x86::Register::edi => 7,
        _ => unsafe { unreachable_unchecked() },
    }
}

// Lookup table for x64 registers to opcode offset
pub(crate) fn opcode_offset_for_x64_register(register: x64::Register) -> u8 {
    match register {
        x64::Register::rax => 0,
        x64::Register::rcx => 1,
        x64::Register::rdx => 2,
        x64::Register::rbx => 3,
        x64::Register::rsp => 4,
        x64::Register::rbp => 5,
        x64::Register::rsi => 6,
        x64::Register::rdi => 7,
        // Additional registers for x64
        x64::Register::r8 => 8,
        x64::Register::r9 => 9,
        x64::Register::r10 => 10,
        x64::Register::r11 => 11,
        x64::Register::r12 => 12,
        x64::Register::r13 => 13,
        x64::Register::r14 => 14,
        x64::Register::r15 => 15,
        // Add other registers if needed
        // TODO: Support for Intel APX registers
        _ => unsafe { unreachable_unchecked() },
    }
}
