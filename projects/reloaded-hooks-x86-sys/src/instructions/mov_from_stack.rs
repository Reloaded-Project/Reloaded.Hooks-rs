extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::{x64::Register as x64Register, x86::Register as x86Register};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::mov_from_stack_operation::MovFromStackOperation;
use zydis::mem;
use zydis::{EncoderRequest, Mnemonic::*, Register::*};

#[cfg(feature = "x86")]
pub(crate) fn encode_mov_from_stack_x86(
    x: &MovFromStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    match x.target {
        r if r.is_32() => encode_mov_from_stack_32_x86(x, pc, buf),
        r if r.is_xmm() => encode_mov_from_stack_xmm_x86(x, pc, buf),
        r if r.is_ymm() => encode_mov_from_stack_ymm_x86(x, pc, buf),
        r if r.is_zmm() => encode_mov_from_stack_zmm_x86(x, pc, buf),
        _ => Err(JitError::InvalidRegister(x.target).into()),
    }
}

#[inline]
#[cfg(feature = "x86")]
fn encode_mov_from_stack_32_x86(
    x: &MovFromStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_x86_register;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 4; // For short offset (-128 to 127)
    const INS_SIZE_U32: usize = 7; // For long offset (beyond -128 to 127)

    let old_len = buf.len();

    // Check if the offset is within the 8-bit range
    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOV reg, [ESP + offset]
            let opcode = 0x04_24_44_8B_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_x86_register(x.target)) << 3) as u32) << 8;
            let offset = (x.stack_offset as u32 & 0xFF) << 24;
            let instruction = opcode | modrm | offset;

            // Write the instruction
            write_unaligned(ptr as *mut u32, instruction.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);

        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOV reg, [ESP + offset]
            let opcode = 0x04_24_84_8B_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_x86_register(x.target)) << 3) as u32) << 8;

            // Write the instruction
            write_unaligned(ptr as *mut u32, (opcode | modrm).to_le());
            write_unaligned(ptr.add(3) as *mut u32, (x.stack_offset as u32).to_le());

            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_from_stack_xmm_x86(
    x: &MovFromStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    // Implement the manual encoding for XMM registers
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 5; // For short offset (-128 to 127)
    const INS_SIZE_U32: usize = 8; // For long offset (beyond -128 to 127)

    let old_len = buf.len();

    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);
        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOVUPS xmm, [ESP + offset]
            let opcode = 0x24_44_10_0F_u32.to_le(); // MOVUPS [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.target)) << 3) as u32) << 16;

            // Write the instruction
            write_unaligned(ptr as *mut u32, (opcode | modrm).to_le());
            write_unaligned(ptr.add(4), x.stack_offset as u8);

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);
        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOVUPS xmm, [ESP + offset]
            let opcode = 0x24_84_10_0F_u32.to_le(); // MOVUPS [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_xmm_register_x86(x.target)) << 3) as u32) << 16;

            // Write the instruction
            write_unaligned(ptr as *mut u32, (opcode | modrm).to_le());
            write_unaligned(ptr.add(4) as *mut i32, x.stack_offset.to_le());

            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_from_stack_ymm_x86(
    x: &MovFromStackOperation<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    // Implement the manual encoding for XMM registers
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;
    use core::ptr::write_unaligned;

    const INS_SIZE_U8: usize = 6; // For short offset (-128 to 127)
    const INS_SIZE_U32: usize = 9; // For long offset (beyond -128 to 127)

    let old_len = buf.len();

    if x.stack_offset <= 0x7F && x.stack_offset >= -0x80 {
        buf.reserve(INS_SIZE_U8);
        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOVUPS xmm, [ESP + offset]
            let opcode = 0x44_10_FC_C5_u32.to_le(); // MOVUPS [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_ymm_register_x86(x.target)) << 3) as u32) << 24;

            // Write the instruction
            write_unaligned(ptr as *mut u32, (opcode | modrm).to_le());

            let ofs = 0x0024_u16.to_le() | (x.stack_offset as u16) << 8;
            write_unaligned(ptr.add(4) as *mut u16, ofs.to_le());

            buf.set_len(old_len + INS_SIZE_U8);
            *pc += INS_SIZE_U8;
        }
    } else {
        buf.reserve(INS_SIZE_U32);
        unsafe {
            let ptr = buf.as_mut_ptr().add(old_len);

            // MOVUPS xmm, [ESP + offset]
            let opcode = 0x84_10_FC_C5_u32.to_le(); // MOVUPS [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_ymm_register_x86(x.target)) << 3) as u32) << 24;

            // Write the instruction
            write_unaligned(ptr as *mut u32, (opcode | modrm).to_le());
            write_unaligned(ptr.add(4), 0x24);
            write_unaligned(ptr.add(5) as *mut i32, x.stack_offset.to_le());

            buf.set_len(old_len + INS_SIZE_U32);
            *pc += INS_SIZE_U32;
        }
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_from_stack_zmm_x86(
    x: &MovFromStackOperation<x86Register>,
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
            write_unaligned(ptr as *mut u32, 0x48_7C_F1_62_u32.to_le());

            // Construct the rest of instruction
            let opcode = 0x0024_4410_u32.to_le();
            let modrm = (((opcode_offset_for_zmm_register_x86(x.target)) << 3) as u32) << 8;
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
            write_unaligned(ptr as *mut u32, 0x48_7C_F1_62_u32.to_le());

            let opcode = 0x24_84_10_48_u32.to_le(); // MOV [ESP + offset], with placeholder for offset
            let modrm = (((opcode_offset_for_zmm_register_x86(x.target)) << 3) as u32) << 16;
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
pub(crate) fn encode_mov_from_stack_x64(
    x: &MovFromStackOperation<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    use crate::common::traits::ToZydis;

    let offset = x.stack_offset as i64;

    if x.target.is_64() {
        *pc += EncoderRequest::new64(MOV)
            .add_operand(x.target.to_zydis())
            .add_operand(mem!(qword ptr [RSP + (offset)]))
            .encode_extend(buf)?;
    } else if x.target.is_xmm() {
        *pc += EncoderRequest::new64(MOVUPS)
            .add_operand(x.target.to_zydis())
            .add_operand(mem!(xmmword ptr [RSP + (offset)]))
            .encode_extend(buf)?;
    } else if x.target.is_ymm() {
        *pc += EncoderRequest::new64(VMOVUPS)
            .add_operand(x.target.to_zydis())
            .add_operand(mem!(ymmword ptr [RSP + (offset)]))
            .encode_extend(buf)?;
    } else if x.target.is_zmm() {
        *pc += EncoderRequest::new64(VMOVUPS)
            .add_operand(x.target.to_zydis())
            .add_operand(K0)
            .add_operand(mem!(zmmword ptr [RSP + (offset)]))
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.target).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use crate::{x64::Register as x64Register, x86::Register as x86Register};
    use reloaded_hooks_portable::api::jit::mov_from_stack_operation::MovFromStackOperation;
    use rstest::rstest;

    #[rstest]
    /*
        #[case(x86Register::eax, 4, "8b442404")]
        #[case(x86Register::ecx, 4, "8b4c2404")]
        #[case(x86Register::edx, 4, "8b542404")]
        #[case(x86Register::ebx, 4, "8b5c2404")]
        #[case(x86Register::esp, 4, "8b642404")]
        #[case(x86Register::ebp, 4, "8b6c2404")]
        #[case(x86Register::esi, 4, "8b742404")]
        #[case(x86Register::edi, 4, "8b7c2404")]
        #[case(x86Register::eax, 0x100, "8b842400010000")]
        #[case(x86Register::ecx, 0x100, "8b8c2400010000")]
        #[case(x86Register::edx, 0x100, "8b942400010000")]
        #[case(x86Register::ebx, 0x100, "8b9c2400010000")]
        #[case(x86Register::esp, 0x100, "8ba42400010000")]
        #[case(x86Register::ebp, 0x100, "8bac2400010000")]
        #[case(x86Register::esi, 0x100, "8bb42400010000")]
        #[case(x86Register::edi, 0x100, "8bbc2400010000")]

        #[case(x86Register::xmm0, 4, "0f10442404")]
        #[case(x86Register::xmm1, 4, "0f104c2404")]
        #[case(x86Register::xmm2, 4, "0f10542404")]
        #[case(x86Register::xmm3, 4, "0f105c2404")]
        #[case(x86Register::xmm4, 4, "0f10642404")]
        #[case(x86Register::xmm5, 4, "0f106c2404")]
        #[case(x86Register::xmm6, 4, "0f10742404")]
        #[case(x86Register::xmm7, 4, "0f107c2404")]
        #[case(x86Register::xmm0, 0x100, "0f10842400010000")]
        #[case(x86Register::xmm1, 0x100, "0f108c2400010000")]
        #[case(x86Register::xmm2, 0x100, "0f10942400010000")]
        #[case(x86Register::xmm3, 0x100, "0f109c2400010000")]
        #[case(x86Register::xmm4, 0x100, "0f10a42400010000")]
        #[case(x86Register::xmm5, 0x100, "0f10ac2400010000")]
        #[case(x86Register::xmm6, 0x100, "0f10b42400010000")]
        #[case(x86Register::xmm7, 0x100, "0f10bc2400010000")]
            #[case(x86Register::ymm0, 4, "c5fc10442404")]
    #[case(x86Register::ymm1, 4, "c5fc104c2404")]
    #[case(x86Register::ymm2, 4, "c5fc10542404")]
    #[case(x86Register::ymm3, 4, "c5fc105c2404")]
    #[case(x86Register::ymm4, 4, "c5fc10642404")]
    #[case(x86Register::ymm5, 4, "c5fc106c2404")]
    #[case(x86Register::ymm6, 4, "c5fc10742404")]
    #[case(x86Register::ymm7, 4, "c5fc107c2404")]
    #[case(x86Register::ymm0, 0x100, "c5fc10842400010000")]
    #[case(x86Register::ymm1, 0x100, "c5fc108c2400010000")]
    #[case(x86Register::ymm2, 0x100, "c5fc10942400010000")]
    #[case(x86Register::ymm3, 0x100, "c5fc109c2400010000")]
    #[case(x86Register::ymm4, 0x100, "c5fc10a42400010000")]
    #[case(x86Register::ymm5, 0x100, "c5fc10ac2400010000")]
    #[case(x86Register::ymm6, 0x100, "c5fc10b42400010000")]
    #[case(x86Register::ymm7, 0x100, "c5fc10bc2400010000")]
        #[case(x86Register::zmm0, 0x80, "62f17c4810442402")]
    #[case(x86Register::zmm1, 0x80, "62f17c48104c2402")]
    #[case(x86Register::zmm2, 0x80, "62f17c4810542402")]
    #[case(x86Register::zmm3, 0x80, "62f17c48105c2402")]
    #[case(x86Register::zmm4, 0x80, "62f17c4810642402")]
    #[case(x86Register::zmm5, 0x80, "62f17c48106c2402")]
    #[case(x86Register::zmm6, 0x80, "62f17c4810742402")]
    #[case(x86Register::zmm7, 0x80, "62f17c48107c2402")]
    */
    #[case(x86Register::zmm0, 0x10, "62f17c4810842410000000")]
    #[case(x86Register::zmm1, 0x10, "62f17c48108c2410000000")]
    #[case(x86Register::zmm2, 0x10, "62f17c4810942410000000")]
    #[case(x86Register::zmm3, 0x10, "62f17c48109c2410000000")]
    #[case(x86Register::zmm4, 0x10, "62f17c4810a42410000000")]
    #[case(x86Register::zmm5, 0x10, "62f17c4810ac2410000000")]
    #[case(x86Register::zmm6, 0x10, "62f17c4810b42410000000")]
    #[case(x86Register::zmm7, 0x10, "62f17c4810bc2410000000")]
    #[case(x86Register::zmm0, 0x20, "62f17c4810842420000000")]
    #[case(x86Register::zmm1, 0x20, "62f17c48108c2420000000")]
    #[case(x86Register::zmm2, 0x20, "62f17c4810942420000000")]
    #[case(x86Register::zmm3, 0x20, "62f17c48109c2420000000")]
    #[case(x86Register::zmm4, 0x20, "62f17c4810a42420000000")]
    #[case(x86Register::zmm5, 0x20, "62f17c4810ac2420000000")]
    #[case(x86Register::zmm6, 0x20, "62f17c4810b42420000000")]
    #[case(x86Register::zmm7, 0x20, "62f17c4810bc2420000000")]

    /*
     */
    fn test_encode_mov_from_stack_x86(
        #[case] target: x86Register,
        #[case] stack_offset: i32,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mov = MovFromStackOperation {
            stack_offset,
            target,
        };
        encode_mov_from_stack_x86(&mov, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(x64Register::rax, "488b442404")]
    #[case(x64Register::xmm0, "0f10442404")]
    #[case(x64Register::ymm0, "c5fc10442404")]
    #[case(x64Register::zmm0, "62f17c4810842404000000")]
    fn test_encode_mov_from_stack_x64(#[case] target: x64Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mov = MovFromStackOperation {
            stack_offset: 4,
            target,
        };
        encode_mov_from_stack_x64(&mov, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
