use crate::{x64, x86};

// Lookup table for x86 registers to opcode offset
#[inline]
pub(crate) fn opcode_offset_for_x86_register(register: x86::Register) -> u8 {
    (register as u16).wrapping_sub(x86::Register::eax as u16) as u8
}

// Lookup table for x64 registers to opcode offset
#[inline]
pub(crate) fn opcode_offset_for_x64_register(register: x64::Register) -> u8 {
    (register as u16).wrapping_sub(x64::Register::rax as u16) as u8
}

// Lookup table for x86 registers to opcode offset
#[inline]
pub(crate) fn opcode_offset_for_xmm_register_x86(register: x86::Register) -> u8 {
    (register as u16).wrapping_sub(x86::Register::xmm0 as u16) as u8
}

// Lookup table for x86 registers to opcode offset
#[inline]
pub(crate) fn opcode_offset_for_ymm_register_x86(register: x86::Register) -> u8 {
    (register as u16).wrapping_sub(x86::Register::ymm0 as u16) as u8
}

// Lookup table for x86 registers to opcode offset
#[inline]
pub(crate) fn opcode_offset_for_zmm_register_x86(register: x86::Register) -> u8 {
    (register as u16).wrapping_sub(x86::Register::zmm0 as u16) as u8
}
