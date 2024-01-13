extern crate alloc;
use core::ptr::write_unaligned;

use crate::common::jit_common::X86jitError;
use crate::{x64::Register as x64Register, x86::Register as x86Register};
use alloc::string::ToString;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::XChg;
use zydis::{EncoderRequest, Mnemonic::*, Register::*};

// Note: For x86 we use manual implementation to save on code size, as that allows us to skip zydis'
// encoder entirely. For x64 we use zydis' encoder in the rewriter, so it's better to use it here
// to save code space in favour of performance.

// For x86 architecture
#[cfg(feature = "x86")]
pub(crate) fn encode_xchg_x86(
    xchg: &XChg<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_x86_register;

    if xchg.register1.is_32() && xchg.register2.is_32() {
        let reg1_offset = opcode_offset_for_x86_register(xchg.register1);
        let reg2_offset = opcode_offset_for_x86_register(xchg.register2);

        // Construct the ModR/M byte (0b11reg1reg2)
        let modrm = 0b1100_0000 | (reg2_offset << 3) | reg1_offset;

        // Construct the opcode for XCHG r32, r32
        let opcode = 0x8700_u16.to_le() | (modrm as u16);
        buf.extend(&opcode.to_be_bytes()); // how tf does Rust get optimal codegen from this ?? It just... does... wtf
        *pc += 2;
    } else if xchg.register1.is_xmm() && xchg.register2.is_xmm() {
        encode_xchg_xmm(xchg, pc, buf)?;
    } else if xchg.register1.is_ymm() && xchg.register2.is_ymm() {
        encode_xchg_ymm(xchg, pc, buf)?;
    } else if xchg.register1.is_zmm() && xchg.register2.is_zmm() {
        encode_xchg_zmm(xchg, pc, buf)?;
    } else {
        return Err(JitError::InvalidRegisterCombination(xchg.register1, xchg.register2).into());
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_xchg_xmm(
    xchg: &XChg<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;

    let scratch = get_scratch(xchg.scratch)?;
    let scratch_index = opcode_offset_for_xmm_register_x86(scratch);
    let reg1_index = opcode_offset_for_xmm_register_x86(xchg.register1);
    let reg2_index = opcode_offset_for_xmm_register_x86(xchg.register2);

    unsafe {
        buf.reserve(9); // Reserve space for 9 bytes (3 instructions, 3 bytes each)
        let old_len = buf.len();
        let ptr = buf.as_mut_ptr().add(old_len);

        // MOVAPS scratch, reg1
        write_unaligned(ptr as *mut u16, 0x280F_u16.to_le());
        ptr.add(2).write(0xC0 | (scratch_index << 3) | reg1_index);

        // MOVAPS reg1, reg2
        write_unaligned(ptr.add(3) as *mut u16, 0x280F_u16.to_le());
        ptr.add(5).write(0xC0 | (reg1_index << 3) | reg2_index);

        // MOVAPS reg2, scratch
        write_unaligned(ptr.add(6) as *mut u16, 0x280F_u16.to_le());
        ptr.add(8).write(0xC0 | (reg2_index << 3) | scratch_index);

        buf.set_len(old_len + 9);
    }

    *pc += 9;
    Ok(())
}

#[cold]
fn encode_xchg_ymm(
    xchg: &XChg<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;

    let scratch = get_scratch(xchg.scratch)?;
    let scratch_index = opcode_offset_for_ymm_register_x86(scratch);
    let reg1_index = opcode_offset_for_ymm_register_x86(xchg.register1);
    let reg2_index = opcode_offset_for_ymm_register_x86(xchg.register2);

    // Prepare the buffer to write the instructions
    let old_len = buf.len();
    buf.reserve(12); // 3 instructions, each 4 bytes
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // VMOVAPS scratch, reg1
        write_unaligned(ptr as *mut u32, 0x0028FCC5_u32.to_le());
        ptr.add(3).write(0xC0 | (scratch_index << 3) | reg1_index);

        // VMOVAPS reg1, reg2
        write_unaligned(ptr.add(4) as *mut u32, 0x0028FCC5_u32.to_le());
        ptr.add(7).write(0xC0 | (reg1_index << 3) | reg2_index);

        // VMOVAPS reg2, scratch
        write_unaligned(ptr.add(8) as *mut u32, 0x0028FCC5_u32.to_le());
        ptr.add(11).write(0xC0 | (reg2_index << 3) | scratch_index);

        buf.set_len(old_len + 12);
    }

    *pc += 12; // Each instruction is 4 bytes, total 12 bytes
    Ok(())
}

#[cold]
fn encode_xchg_zmm(
    xchg: &XChg<x86Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_zmm_register_x86;

    let scratch = get_scratch(xchg.scratch)?;
    let scratch_index = opcode_offset_for_zmm_register_x86(scratch);
    let reg1_index = opcode_offset_for_zmm_register_x86(xchg.register1);
    let reg2_index = opcode_offset_for_zmm_register_x86(xchg.register2);

    // Prepare the buffer to write the instructions
    let old_len = buf.len();
    buf.reserve(18); // 3 instructions, each 6 bytes
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // VMOVAPS scratch, reg1
        write_unaligned(ptr as *mut u32, 0x487CF162_u32.to_le());
        write_unaligned(
            ptr.add(4) as *mut u16,
            0x0028_u16.to_le() | ((0xC0 | (scratch_index << 3) | reg1_index) as u16) << 8,
        );

        // VMOVAPS reg1, reg2
        write_unaligned(ptr.add(6) as *mut u32, 0x487CF162_u32.to_le());
        write_unaligned(
            ptr.add(10) as *mut u16,
            0x0028_u16.to_le() | ((0xC0 | (reg1_index << 3) | reg2_index) as u16) << 8,
        );

        // VMOVAPS reg2, scratch
        write_unaligned(ptr.add(12) as *mut u32, 0x487CF162_u32.to_le());
        write_unaligned(
            ptr.add(16) as *mut u16,
            0x0028_u16.to_le() | ((0xC0 | (reg2_index << 3) | scratch_index) as u16) << 8,
        );

        buf.set_len(old_len + 18);
    }

    *pc += 18; // Each instruction is 6 bytes, total 18 bytes
    Ok(())
}

// For x64 architecture
#[cfg(feature = "x64")]
pub(crate) fn encode_xchg_x64(
    xchg: &XChg<x64Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64Register>> {
    if xchg.register1.is_64() && xchg.register2.is_64() {
        *pc += EncoderRequest::new64(XCHG)
            .add_operand(xchg.register1.to_zydis())
            .add_operand(xchg.register2.to_zydis())
            .encode_extend(buf)?;
    } else if xchg.register1.is_xmm() && xchg.register2.is_xmm() {
        let scratch = get_scratch(xchg.scratch)?;
        *pc += EncoderRequest::new64(MOVAPS)
            .add_operand(scratch.to_zydis())
            .add_operand(xchg.register1.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new64(MOVAPS)
            .add_operand(xchg.register1.to_zydis())
            .add_operand(xchg.register2.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new64(MOVAPS)
            .add_operand(xchg.register2.to_zydis())
            .add_operand(scratch.to_zydis())
            .encode_extend(buf)?;
    } else if xchg.register1.is_ymm() && xchg.register2.is_ymm() {
        let scratch = get_scratch(xchg.scratch)?;
        *pc += EncoderRequest::new64(VMOVAPS)
            .add_operand(scratch.to_zydis())
            .add_operand(xchg.register1.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new64(VMOVAPS)
            .add_operand(xchg.register1.to_zydis())
            .add_operand(xchg.register2.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new64(VMOVAPS)
            .add_operand(xchg.register2.to_zydis())
            .add_operand(scratch.to_zydis())
            .encode_extend(buf)?;
    } else if xchg.register1.is_zmm() && xchg.register2.is_zmm() {
        let scratch = get_scratch(xchg.scratch)?;
        *pc += EncoderRequest::new32(VMOVAPS)
            .add_operand(scratch.to_zydis())
            .add_operand(K0)
            .add_operand(xchg.register1.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new32(VMOVAPS)
            .add_operand(xchg.register1.to_zydis())
            .add_operand(K0)
            .add_operand(xchg.register2.to_zydis())
            .encode_extend(buf)?;
        *pc += EncoderRequest::new32(VMOVAPS)
            .add_operand(xchg.register2.to_zydis())
            .add_operand(K0)
            .add_operand(scratch.to_zydis())
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegisterCombination(xchg.register1, xchg.register2).into());
    }

    Ok(())
}

fn get_scratch<TRegister>(scratch: Option<TRegister>) -> Result<TRegister, X86jitError<TRegister>> {
    match scratch {
        Some(s) => Ok(s),
        None => Err(JitError::NoScratchRegister("Needed for XChgOperation.".to_string()).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_xchg_x64, encode_xchg_x86};
    use crate::common::util::test_utilities::assert_encode;
    use crate::{x64::Register as x64Register, x86::Register as x86Register};
    use reloaded_hooks_portable::api::jit::operation_aliases::XChg;
    use rstest::rstest;

    #[rstest]
    #[case(x86Register::eax, x86Register::ebx, None, "87d8")]
    #[case(x86Register::ecx, x86Register::esi, None, "87f1")]
    #[case(
        x86Register::xmm0,
        x86Register::xmm1,
        Some(x86Register::xmm2),
        "0f28d00f28c10f28ca"
    )]
    #[case(
        x86Register::xmm6,
        x86Register::xmm7,
        Some(x86Register::xmm2),
        "0f28d60f28f70f28fa"
    )]
    #[case(
        x86Register::ymm0,
        x86Register::ymm1,
        Some(x86Register::ymm2),
        "c5fc28d0c5fc28c1c5fc28ca"
    )]
    #[case(
        x86Register::zmm0,
        x86Register::zmm1,
        Some(x86Register::zmm2),
        "62f17c4828d062f17c4828c162f17c4828ca"
    )]
    fn test_encode_xchg_x86(
        #[case] register1: x86Register,
        #[case] register2: x86Register,
        #[case] scratch: Option<x86Register>,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let xchg = XChg::new(register1, register2, scratch);
        encode_xchg_x86(&xchg, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(x64Register::rax, x64Register::rbx, None, "4887d8")]
    #[case(
        x64Register::xmm0,
        x64Register::xmm1,
        Some(x64Register::xmm2),
        "0f28d00f28c10f28ca"
    )]
    #[case(
        x64Register::ymm0,
        x64Register::ymm1,
        Some(x64Register::ymm2),
        "c5fc28d0c5fc28c1c5fc28ca"
    )]
    #[case(
        x64Register::zmm0,
        x64Register::zmm1,
        Some(x64Register::zmm2),
        "62f17c4828d062f17c4828c162f17c4828ca"
    )]
    fn test_encode_xchg_x64(
        #[case] register1: x64Register,
        #[case] register2: x64Register,
        #[case] scratch: Option<x64Register>,
        #[case] expected_encoded: &str,
    ) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let xchg = XChg::new(register1, register2, scratch);
        encode_xchg_x64(&xchg, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
