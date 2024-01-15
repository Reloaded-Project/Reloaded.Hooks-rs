extern crate alloc;

use crate::common::jit_common::X86jitError;
use crate::x64::Register as x64Register;
use crate::x86::Register as x86Register;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::operation_aliases::Mov;
use zydis::Register::K0;
use zydis::{EncoderRequest, Mnemonic::*};

// x86 implementation
#[cfg(feature = "x86")]
pub(crate) fn encode_mov_x86(
    mov: &Mov<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_x86_register;
    use reloaded_hooks_portable::api::jit::compiler::JitError;

    if mov.target.is_32() && mov.source.is_32() {
        // MOV r32, r32 opcode
        let opcode = 0x89C0_u16.to_le()
            | (opcode_offset_for_x86_register(mov.source) << 3) as u16
            | opcode_offset_for_x86_register(mov.target) as u16;

        buf.extend(opcode.to_be_bytes());
        *pc += 2;
    } else if mov.target.is_xmm() && mov.source.is_xmm() {
        encode_mov_xmm_x86(mov, pc, buf)?;
    } else if mov.target.is_ymm() && mov.source.is_ymm() {
        encode_mov_ymm_x86(mov, pc, buf)?;
    } else if mov.target.is_zmm() && mov.source.is_zmm() {
        encode_mov_zmm_x86(mov, pc, buf)?;
    } else {
        return Err(JitError::InvalidRegisterCombination(mov.source, mov.target).into());
    }
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_xmm_x86(
    mov: &Mov<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    // MOVAPS xmm, xmm (opcode: 0x0F 0x28 + ModRM)
    buf.reserve(3);

    let old_len = buf.len();
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);
        write_unaligned(ptr as *mut u16, 0x280F_u16.to_le());
        ptr.add(2).write(
            0xC0 | (opcode_offset_for_xmm_register_x86(mov.target) << 3)
                | opcode_offset_for_xmm_register_x86(mov.source),
        );
        buf.set_len(old_len + 3);
    }
    *pc += 3;
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_ymm_x86(
    mov: &Mov<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;
    use core::ptr::write_unaligned;

    // VMOVAPS ymm, ymm (VEX prefix + opcode: 0xC5 0xFC 0x28 + ModRM)
    buf.reserve(4);

    let old_len = buf.len();
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);
        write_unaligned(ptr as *mut u32, 0x0028FCC5_u32.to_le());
        ptr.add(3).write(
            0xC0 | (opcode_offset_for_ymm_register_x86(mov.target) << 3)
                | opcode_offset_for_ymm_register_x86(mov.source),
        );
        buf.set_len(old_len + 4);
    }
    *pc += 4;
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_mov_zmm_x86(
    mov: &Mov<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_zmm_register_x86;
    use core::ptr::write_unaligned;

    // VMOVAPS ymm, ymm (VEX prefix + opcode: 0xC5 0xFC 0x28 + ModRM)
    buf.reserve(6);

    let old_len = buf.len();
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);
        write_unaligned(ptr as *mut u32, 0x487CF162_u32.to_le());
        ptr.add(4).write(0x28);
        ptr.add(5).write(
            0xC0 | (opcode_offset_for_zmm_register_x86(mov.target) << 3)
                | opcode_offset_for_zmm_register_x86(mov.source),
        );
        buf.set_len(old_len + 6);
    }
    *pc += 6;
    Ok(())
}

// x64 implementation
#[cfg(feature = "x64")]
pub(crate) fn encode_mov_x64(
    mov: &Mov<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    use reloaded_hooks_portable::api::jit::compiler::JitError;

    use crate::common::traits::ToZydis;

    if mov.target.is_64() && mov.source.is_64() {
        *pc += EncoderRequest::new64(MOV)
            .add_operand(mov.target.to_zydis())
            .add_operand(mov.source.to_zydis())
            .encode_extend(buf)?;
    } else if mov.target.is_xmm() && mov.source.is_xmm() {
        *pc += EncoderRequest::new64(MOVAPS)
            .add_operand(mov.target.to_zydis())
            .add_operand(mov.source.to_zydis())
            .encode_extend(buf)?;
    } else if mov.target.is_ymm() && mov.source.is_ymm() {
        *pc += EncoderRequest::new64(VMOVAPS)
            .add_operand(mov.target.to_zydis())
            .add_operand(mov.source.to_zydis())
            .encode_extend(buf)?;
    } else if mov.target.is_zmm() && mov.source.is_zmm() {
        *pc += EncoderRequest::new64(VMOVAPS)
            .add_operand(mov.target.to_zydis())
            .add_operand(K0)
            .add_operand(mov.source.to_zydis())
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegisterCombination(mov.source, mov.target).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::assert_encode;
    use crate::{x64::Register as x64Register, x86::Register as x86Register};
    use reloaded_hooks_portable::api::jit::operation_aliases::Mov;
    use rstest::rstest;

    #[rstest]
    #[case(x86Register::eax, x86Register::ecx, "89c1")]
    #[case(x86Register::eax, x86Register::edx, "89c2")]
    #[case(x86Register::eax, x86Register::ebx, "89c3")]
    #[case(x86Register::eax, x86Register::esp, "89c4")]
    #[case(x86Register::eax, x86Register::ebp, "89c5")]
    #[case(x86Register::eax, x86Register::esi, "89c6")]
    #[case(x86Register::eax, x86Register::edi, "89c7")]
    #[case(x86Register::ecx, x86Register::eax, "89c8")]
    #[case(x86Register::ecx, x86Register::edx, "89ca")]
    #[case(x86Register::ecx, x86Register::ebx, "89cb")]
    #[case(x86Register::ecx, x86Register::esp, "89cc")]
    #[case(x86Register::ecx, x86Register::ebp, "89cd")]
    #[case(x86Register::ecx, x86Register::esi, "89ce")]
    #[case(x86Register::ecx, x86Register::edi, "89cf")]
    #[case(x86Register::xmm0, x86Register::xmm1, "0f28c8")]
    #[case(x86Register::xmm0, x86Register::xmm2, "0f28d0")]
    #[case(x86Register::xmm0, x86Register::xmm3, "0f28d8")]
    #[case(x86Register::xmm0, x86Register::xmm4, "0f28e0")]
    #[case(x86Register::xmm0, x86Register::xmm5, "0f28e8")]
    #[case(x86Register::xmm0, x86Register::xmm6, "0f28f0")]
    #[case(x86Register::xmm0, x86Register::xmm7, "0f28f8")]
    #[case(x86Register::xmm1, x86Register::xmm0, "0f28c1")]
    #[case(x86Register::xmm1, x86Register::xmm2, "0f28d1")]
    #[case(x86Register::xmm1, x86Register::xmm3, "0f28d9")]
    #[case(x86Register::xmm1, x86Register::xmm4, "0f28e1")]
    #[case(x86Register::xmm1, x86Register::xmm5, "0f28e9")]
    #[case(x86Register::xmm1, x86Register::xmm6, "0f28f1")]
    #[case(x86Register::xmm1, x86Register::xmm7, "0f28f9")]
    #[case(x86Register::ymm0, x86Register::ymm1, "c5fc28c8")]
    #[case(x86Register::ymm0, x86Register::ymm2, "c5fc28d0")]
    #[case(x86Register::ymm0, x86Register::ymm3, "c5fc28d8")]
    #[case(x86Register::ymm0, x86Register::ymm4, "c5fc28e0")]
    #[case(x86Register::ymm0, x86Register::ymm5, "c5fc28e8")]
    #[case(x86Register::ymm0, x86Register::ymm6, "c5fc28f0")]
    #[case(x86Register::ymm0, x86Register::ymm7, "c5fc28f8")]
    #[case(x86Register::ymm1, x86Register::ymm0, "c5fc28c1")]
    #[case(x86Register::ymm1, x86Register::ymm2, "c5fc28d1")]
    #[case(x86Register::ymm1, x86Register::ymm3, "c5fc28d9")]
    #[case(x86Register::ymm1, x86Register::ymm4, "c5fc28e1")]
    #[case(x86Register::ymm1, x86Register::ymm5, "c5fc28e9")]
    #[case(x86Register::ymm1, x86Register::ymm6, "c5fc28f1")]
    #[case(x86Register::ymm1, x86Register::ymm7, "c5fc28f9")]
    #[case(x86Register::zmm0, x86Register::zmm1, "62f17c4828c8")]
    #[case(x86Register::zmm0, x86Register::zmm2, "62f17c4828d0")]
    #[case(x86Register::zmm0, x86Register::zmm3, "62f17c4828d8")]
    #[case(x86Register::zmm0, x86Register::zmm4, "62f17c4828e0")]
    #[case(x86Register::zmm0, x86Register::zmm5, "62f17c4828e8")]
    #[case(x86Register::zmm0, x86Register::zmm6, "62f17c4828f0")]
    #[case(x86Register::zmm0, x86Register::zmm7, "62f17c4828f8")]
    #[case(x86Register::zmm1, x86Register::zmm0, "62f17c4828c1")]
    #[case(x86Register::zmm1, x86Register::zmm2, "62f17c4828d1")]
    #[case(x86Register::zmm1, x86Register::zmm3, "62f17c4828d9")]
    #[case(x86Register::zmm1, x86Register::zmm4, "62f17c4828e1")]
    #[case(x86Register::zmm1, x86Register::zmm5, "62f17c4828e9")]
    #[case(x86Register::zmm1, x86Register::zmm6, "62f17c4828f1")]
    #[case(x86Register::zmm1, x86Register::zmm7, "62f17c4828f9")]
    fn test_encode_mov_x86(
        #[case] source: x86Register,
        #[case] target: x86Register,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mov = Mov::new(source, target);
        encode_mov_x86(&mov, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[cfg(target_pointer_width = "64")]
    #[case(x64Register::rax, x64Register::rbx, "4889c3")]
    #[case(x64Register::xmm0, x64Register::xmm1, "0f28c8")]
    #[case(x64Register::ymm0, x64Register::ymm1, "c5fc28c8")]
    #[case(x64Register::zmm0, x64Register::zmm1, "62f17c4828c8")]
    fn test_encode_mov_x64(
        #[case] source: x64Register,
        #[case] target: x64Register,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let mov = Mov::new(source, target);
        encode_mov_x64(&mov, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
