extern crate alloc;
use crate::common::jit_common::X86jitError;
use crate::x64;
use crate::x86;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::pop_operation::PopOperation;
use zydis::{mem, EncoderRequest, Mnemonic::*, Register::*};

#[cfg(feature = "x86")]
pub(crate) fn encode_pop_32(
    x: &PopOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_x86_register;

    if x.register.is_32() {
        buf.push(0x58 + opcode_offset_for_x86_register(x.register));
        *pc += 1;
    } else if x.register.is_xmm() {
        encode_pop_xmm_32(x, pc, buf)?;
    } else if x.register.is_ymm() {
        encode_pop_ymm_32(x, pc, buf)?;
    } else if x.register.is_zmm() {
        encode_pop_zmm_32(x, pc, buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_pop_xmm_32(
    x: &PopOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_xmm_register_x86(x.register);

    let old_len = buf.len();
    buf.reserve(8);
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // MOVDQU reg, [ESP]
        // ADD ESP, 16
        let instruction: u64 =
            0x10C4_8324_046F_0FF3_u64.to_le() | (((reg_offset << 3) as u64) << 24);
        write_unaligned(ptr as *mut u64, instruction.to_le());
        buf.set_len(old_len + 8);
    }

    *pc += 8;
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_pop_ymm_32(
    x: &PopOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_ymm_register_x86(x.register);

    let old_len = buf.len();
    buf.reserve(8);
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // VMOVDQU reg, [ESP]
        // ADD ESP, 32
        let instruction: u64 =
            0x20C4_8324_046F_FEC5_u64.to_le() | (((reg_offset << 3) as u64) << 24);
        write_unaligned(ptr as *mut u64, instruction.to_le());
        buf.set_len(old_len + 8);
    }

    *pc += 8;
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_pop_zmm_32(
    x: &PopOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_zmm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_zmm_register_x86(x.register);

    let old_len = buf.len();
    buf.reserve(10);
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // VMOVDQU64 reg, [ESP]
        // ADD ESP, 64
        write_unaligned(
            ptr as *mut u64,
            0x8324_046F_487E_F162_u64.to_le() | (((reg_offset << 3) as u64) << 40),
        );
        write_unaligned(ptr.add(8) as *mut u16, 0x40C4_u16.to_le());
        buf.set_len(old_len + 10);
    }

    *pc += 10;
    Ok(())
}

#[cfg(feature = "x64")]
pub(crate) fn encode_pop_64(
    x: &PopOperation<x64::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64::Register>> {
    use crate::common::traits::ToZydis;

    if x.register.is_64() {
        *pc += EncoderRequest::new64(POP)
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_xmm() {
        *pc += EncoderRequest::new64(MOVDQU)
            .add_operand(x.register.to_zydis())
            .add_operand(mem!(xmmword ptr [RSP]))
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(ADD)
            .add_operand(RSP)
            .add_operand(16)
            .encode_extend(buf)?;
    } else if x.register.is_ymm() {
        *pc += EncoderRequest::new64(VMOVDQU)
            .add_operand(x.register.to_zydis())
            .add_operand(mem!(ymmword ptr [RSP]))
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(ADD)
            .add_operand(RSP)
            .add_operand(32)
            .encode_extend(buf)?;
    } else if x.register.is_zmm() {
        *pc += EncoderRequest::new64(VMOVDQU64)
            .add_operand(x.register.to_zydis())
            .add_operand(K0)
            .add_operand(mem!(zmmword ptr [RSP]))
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(ADD)
            .add_operand(RSP)
            .add_operand(64)
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; // Assuming you have the necessary imports at the beginning of pop.rs
    use crate::{
        common::util::test_utilities::assert_encode, x64::Register as X64Register,
        x86::Register as X86Register,
    };
    use reloaded_hooks_portable::api::jit::pop_operation::PopOperation;
    use rstest::rstest;

    #[rstest]
    #[case(X86Register::eax, "58")]
    #[case(X86Register::ebx, "5b")]
    #[case(X86Register::ecx, "59")]
    #[case(X86Register::edx, "5a")]
    #[case(X86Register::esi, "5e")]
    #[case(X86Register::edi, "5f")]
    #[case(X86Register::ebp, "5d")]
    #[case(X86Register::esp, "5c")]
    #[case(X86Register::xmm0, "f30f6f042483c410")]
    #[case(X86Register::xmm1, "f30f6f0c2483c410")]
    #[case(X86Register::xmm2, "f30f6f142483c410")]
    #[case(X86Register::xmm3, "f30f6f1c2483c410")]
    #[case(X86Register::xmm4, "f30f6f242483c410")]
    #[case(X86Register::xmm5, "f30f6f2c2483c410")]
    #[case(X86Register::xmm6, "f30f6f342483c410")]
    #[case(X86Register::xmm7, "f30f6f3c2483c410")]
    #[case(X86Register::ymm0, "c5fe6f042483c420")]
    #[case(X86Register::ymm1, "c5fe6f0c2483c420")]
    #[case(X86Register::ymm2, "c5fe6f142483c420")]
    #[case(X86Register::ymm3, "c5fe6f1c2483c420")]
    #[case(X86Register::ymm4, "c5fe6f242483c420")]
    #[case(X86Register::ymm5, "c5fe6f2c2483c420")]
    #[case(X86Register::ymm6, "c5fe6f342483c420")]
    #[case(X86Register::ymm7, "c5fe6f3c2483c420")]
    #[case(X86Register::zmm0, "62f17e486f042483c440")]
    #[case(X86Register::zmm1, "62f17e486f0c2483c440")]
    #[case(X86Register::zmm2, "62f17e486f142483c440")]
    #[case(X86Register::zmm3, "62f17e486f1c2483c440")]
    #[case(X86Register::zmm4, "62f17e486f242483c440")]
    #[case(X86Register::zmm5, "62f17e486f2c2483c440")]
    #[case(X86Register::zmm6, "62f17e486f342483c440")]
    #[case(X86Register::zmm7, "62f17e486f3c2483c440")]
    fn pop_x86(#[case] register: X86Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = PopOperation { register };

        encode_pop_32(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(X64Register::rax, "58")]
    #[case(X64Register::rbx, "5b")]
    #[case(X64Register::rcx, "59")]
    #[case(X64Register::rdx, "5a")]
    #[case(X64Register::rsi, "5e")]
    #[case(X64Register::rdi, "5f")]
    #[case(X64Register::rbp, "5d")]
    #[case(X64Register::rsp, "5c")]
    #[case(X64Register::xmm0, "f30f6f04244883c410")]
    #[case(X64Register::ymm0, "c5fe6f04244883c420")]
    #[case(X64Register::zmm0, "62f1fe486f04244883c440")]
    fn pop_x64(#[case] register: X64Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = PopOperation { register };

        encode_pop_64(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
