extern crate alloc;

use crate::{
    common::jit_common::X86jitError, x64::Register as x64Register, x86::Register as x86Register,
};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::mov_to_stack_operation::MovToStackOperation;
use zydis::Register::K0;
use zydis::{mem, EncoderRequest, Mnemonic::*};

// For x86 architecture
#[cfg(feature = "x86")]
pub(crate) fn encode_mov_to_stack_x86(
    x: &MovToStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    match x.register {
        r if r.is_32() => encode_mov_to_stack_32_x86(x, pc, buf),
        r if r.is_xmm() => encode_mov_to_stack_xmm_x86(x, pc, buf),
        r if r.is_ymm() => encode_mov_to_stack_ymm_x86(x, pc, buf),
        r if r.is_zmm() => encode_mov_to_stack_zmm_x86(x, pc, buf),
        _ => Err(JitError::InvalidRegister(x.register).into()),
    }
}

#[inline]
#[cfg(feature = "x86")]
fn encode_mov_to_stack_32_x86(
    x: &MovToStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 4;
    const INS_SIZE_U32: usize = 7;

    let old_len = buf.len();
    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            let opcode = 0x00_24_44_89_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.register)) << 3) as u32) << 8;
            let offset = (x.stack_offset as u32 & 0xFF) << 24;
            let instruction = opcode | modrm | offset; // Insert offset into the instruction

            // Write the instruction
            write_unaligned(ptr as *mut u32, instruction.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            let opcode = 0x00_24_84_89_u32.to_le(); // MOV [ESP], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.register)) << 3) as u32) << 8;
            let instruction = opcode | modrm;

            write_unaligned(ptr as *mut u32, instruction.to_le()); // Write the instruction
            write_unaligned(ptr.add(3) as *mut u32, (x.stack_offset as u32).to_le()); // offset

            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_to_stack_xmm_x86(
    x: &MovToStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 6;
    const INS_SIZE_U32: usize = 9;

    let old_len = buf.len();
    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            let opcode = 0x44_7F_0F_F3_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.register)) << 3) as u32) << 24;
            let part1 = opcode | modrm;
            write_unaligned(ptr as *mut u32, part1.to_le());

            let offset = (x.stack_offset as u32 & 0xFF) << 8;
            let part2 = 0x0024_u16 | offset as u16;
            write_unaligned(ptr.add(4) as *mut u16, part2.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            let opcode = 0x84_7F_0F_F3_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.register)) << 3) as u32) << 24;
            let part1 = opcode | modrm;
            write_unaligned(ptr as *mut u32, part1.to_le());
            write_unaligned(ptr.add(4), 0x24);
            write_unaligned(ptr.add(5) as *mut u32, (x.stack_offset as u32).to_le());
            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_to_stack_ymm_x86(
    x: &MovToStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 6;
    const INS_SIZE_U32: usize = 9;

    let old_len = buf.len();
    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            let opcode = 0x44_7F_FE_C5_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_ymm_register_x86(x.register)) << 3) as u32) << 24;
            let part1 = opcode | modrm;
            write_unaligned(ptr as *mut u32, part1.to_le());

            let offset = (x.stack_offset as u32 & 0xFF) << 8;
            let part2 = 0x0024_u16 | offset as u16;
            write_unaligned(ptr.add(4) as *mut u16, part2.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            let opcode = 0x84_7F_FE_C5_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_ymm_register_x86(x.register)) << 3) as u32) << 24;
            let part1 = opcode | modrm;
            write_unaligned(ptr as *mut u32, part1.to_le());
            write_unaligned(ptr.add(4), 0x24);
            write_unaligned(ptr.add(5) as *mut u32, (x.stack_offset as u32).to_le());
            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_to_stack_zmm_x86(
    x: &MovToStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_zmm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 8;
    const INS_SIZE_U32: usize = 11;

    // Note: There is also another encoding for offset == 0, which takes 1 less byte, however
    // our call convention wrapper code shouldn't emit this.

    let old_len = buf.len();
    let is_divisible = x.stack_offset % 0x40 == 0;

    if is_divisible && x.stack_offset <= 0x7F * 0x40 && x.stack_offset >= -0x80 * 0x40 {
        buf.reserve(INS_SIZE_U8);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // First part of opcode
            write_unaligned(ptr as *mut u32, 0x48_7E_F1_62_u32.to_le());

            // Construct the rest of instruction
            let opcode = 0x0024_447F_u32.to_le();
            let modrm = (((opcode_offset_for_zmm_register_x86(x.register)) << 3) as u32) << 8;
            let offset = ((x.stack_offset / 0x40) as u32 & 0xFF) << 24;
            let ins = opcode | modrm | offset;
            write_unaligned(ptr.add(4) as *mut u32, ins.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // Construct the instruction
            write_unaligned(ptr as *mut u32, 0x48_7E_F1_62_u32.to_le());

            let opcode = 0x24_84_7F_48_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_zmm_register_x86(x.register)) << 3) as u32) << 16;
            write_unaligned(ptr.add(3) as *mut u32, (opcode | modrm).to_le());
            write_unaligned(ptr.add(7) as *mut u32, (x.stack_offset as u32).to_le());
            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

// For x64 architecture
#[cfg(feature = "x64")]
pub(crate) fn encode_mov_to_stack_x64(
    x: &MovToStackOperation<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    use crate::common::traits::ToZydis;

    let offset = x.stack_offset as i64;
    if x.register.is_64() {
        *pc += EncoderRequest::new64(MOV)
            .add_operand(mem!(qword ptr [RSP + (offset)]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_xmm() {
        *pc += EncoderRequest::new64(MOVDQU)
            .add_operand(mem!(xmmword ptr [RSP + (offset)]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_ymm() {
        *pc += EncoderRequest::new64(VMOVDQU)
            .add_operand(mem!(ymmword ptr [RSP + (offset)]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_zmm() {
        *pc += EncoderRequest::new64(VMOVDQU64)
            .add_operand(mem!(zmmword ptr [RSP + (offset)]))
            .add_operand(K0)
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use crate::{x64::Register as x64Register, x86::Register as x86Register};
    use reloaded_hooks_portable::api::jit::mov_to_stack_operation::MovToStackOperation;
    use rstest::rstest;

    // Test cases for x64 architecture
    #[rstest]
    #[case(x64Register::rax, 16, "4889442410")]
    #[case(x64Register::xmm0, 16, "f30f7f442410")]
    #[case(x64Register::ymm0, 16, "c5fe7f442410")]
    #[case(x64Register::zmm0, 16, "62f1fe487f842410000000")]
    fn test_encode_mov_to_stack_x64(
        #[case] register: x64Register,
        #[case] offset: i32,
        #[case] expected: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovToStackOperation::new(offset, register);
        encode_mov_to_stack_x64(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected, &buf, pc);
    }

    // Test cases for x86 architecture
    #[rstest]
    #[case(x86Register::eax, 16, "89442410")]
    #[case(x86Register::ecx, 16, "894c2410")]
    #[case(x86Register::edx, 16, "89542410")]
    #[case(x86Register::ebx, 16, "895c2410")]
    #[case(x86Register::esp, 16, "89642410")]
    #[case(x86Register::ebp, 16, "896c2410")]
    #[case(x86Register::esi, 16, "89742410")]
    #[case(x86Register::edi, 16, "897c2410")]
    #[case(x86Register::eax, 0x100, "89842400010000")]
    #[case(x86Register::ecx, 0x100, "898c2400010000")]
    #[case(x86Register::edx, 0x100, "89942400010000")]
    #[case(x86Register::ebx, 0x100, "899c2400010000")]
    #[case(x86Register::esp, 0x100, "89a42400010000")]
    #[case(x86Register::ebp, 0x100, "89ac2400010000")]
    #[case(x86Register::esi, 0x100, "89b42400010000")]
    #[case(x86Register::edi, 0x100, "89bc2400010000")]
    #[case(x86Register::xmm0, 16, "f30f7f442410")]
    #[case(x86Register::xmm1, 16, "f30f7f4c2410")]
    #[case(x86Register::xmm2, 16, "f30f7f542410")]
    #[case(x86Register::xmm3, 16, "f30f7f5c2410")]
    #[case(x86Register::xmm4, 16, "f30f7f642410")]
    #[case(x86Register::xmm5, 16, "f30f7f6c2410")]
    #[case(x86Register::xmm6, 16, "f30f7f742410")]
    #[case(x86Register::xmm7, 16, "f30f7f7c2410")]
    #[case(x86Register::xmm0, 0x100, "f30f7f842400010000")]
    #[case(x86Register::xmm1, 0x100, "f30f7f8c2400010000")]
    #[case(x86Register::xmm2, 0x100, "f30f7f942400010000")]
    #[case(x86Register::xmm3, 0x100, "f30f7f9c2400010000")]
    #[case(x86Register::xmm4, 0x100, "f30f7fa42400010000")]
    #[case(x86Register::xmm5, 0x100, "f30f7fac2400010000")]
    #[case(x86Register::xmm6, 0x100, "f30f7fb42400010000")]
    #[case(x86Register::xmm7, 0x100, "f30f7fbc2400010000")]
    #[case(x86Register::ymm0, 16, "c5fe7f442410")]
    #[case(x86Register::ymm1, 16, "c5fe7f4c2410")]
    #[case(x86Register::ymm2, 16, "c5fe7f542410")]
    #[case(x86Register::ymm3, 16, "c5fe7f5c2410")]
    #[case(x86Register::ymm4, 16, "c5fe7f642410")]
    #[case(x86Register::ymm5, 16, "c5fe7f6c2410")]
    #[case(x86Register::ymm6, 16, "c5fe7f742410")]
    #[case(x86Register::ymm7, 16, "c5fe7f7c2410")]
    #[case(x86Register::ymm0, 0x100, "c5fe7f842400010000")]
    #[case(x86Register::ymm1, 0x100, "c5fe7f8c2400010000")]
    #[case(x86Register::ymm2, 0x100, "c5fe7f942400010000")]
    #[case(x86Register::ymm3, 0x100, "c5fe7f9c2400010000")]
    #[case(x86Register::ymm4, 0x100, "c5fe7fa42400010000")]
    #[case(x86Register::ymm5, 0x100, "c5fe7fac2400010000")]
    #[case(x86Register::ymm6, 0x100, "c5fe7fb42400010000")]
    #[case(x86Register::ymm7, 0x100, "c5fe7fbc2400010000")]
    #[case(x86Register::zmm0, 0x40, "62f17e487f442401")]
    #[case(x86Register::zmm1, 0x40, "62f17e487f4c2401")]
    #[case(x86Register::zmm2, 0x40, "62f17e487f542401")]
    #[case(x86Register::zmm3, 0x40, "62f17e487f5c2401")]
    #[case(x86Register::zmm4, 0x40, "62f17e487f642401")]
    #[case(x86Register::zmm5, 0x40, "62f17e487f6c2401")]
    #[case(x86Register::zmm6, 0x40, "62f17e487f742401")]
    #[case(x86Register::zmm7, 0x40, "62f17e487f7c2401")]
    #[case(x86Register::zmm0, 0x80, "62f17e487f442402")]
    #[case(x86Register::zmm1, 0x80, "62f17e487f4c2402")]
    #[case(x86Register::zmm2, 0x80, "62f17e487f542402")]
    #[case(x86Register::zmm3, 0x80, "62f17e487f5c2402")]
    #[case(x86Register::zmm4, 0x80, "62f17e487f642402")]
    #[case(x86Register::zmm5, 0x80, "62f17e487f6c2402")]
    #[case(x86Register::zmm6, 0x80, "62f17e487f742402")]
    #[case(x86Register::zmm7, 0x80, "62f17e487f7c2402")]
    #[case(x86Register::zmm0, 0x10, "62f17e487f842410000000")]
    #[case(x86Register::zmm1, 0x10, "62f17e487f8c2410000000")]
    #[case(x86Register::zmm2, 0x10, "62f17e487f942410000000")]
    #[case(x86Register::zmm3, 0x10, "62f17e487f9c2410000000")]
    #[case(x86Register::zmm4, 0x10, "62f17e487fa42410000000")]
    #[case(x86Register::zmm5, 0x10, "62f17e487fac2410000000")]
    #[case(x86Register::zmm6, 0x10, "62f17e487fb42410000000")]
    #[case(x86Register::zmm7, 0x10, "62f17e487fbc2410000000")]
    #[case(x86Register::zmm0, 0x20, "62f17e487f842420000000")]
    #[case(x86Register::zmm1, 0x20, "62f17e487f8c2420000000")]
    #[case(x86Register::zmm2, 0x20, "62f17e487f942420000000")]
    #[case(x86Register::zmm3, 0x20, "62f17e487f9c2420000000")]
    #[case(x86Register::zmm4, 0x20, "62f17e487fa42420000000")]
    #[case(x86Register::zmm5, 0x20, "62f17e487fac2420000000")]
    #[case(x86Register::zmm6, 0x20, "62f17e487fb42420000000")]
    #[case(x86Register::zmm7, 0x20, "62f17e487fbc2420000000")]
    fn test_encode_mov_to_stack_x86(
        #[case] register: x86Register,
        #[case] offset: i32,
        #[case] expected: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = MovToStackOperation::new(offset, register);
        encode_mov_to_stack_x86(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected, &buf, pc);
    }
}
