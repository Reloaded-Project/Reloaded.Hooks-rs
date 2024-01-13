extern crate alloc;
use crate::common::jit_common::X86jitError;
use crate::x64;
use crate::x86;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::push_operation::PushOperation;
use zydis::{mem, EncoderRequest, Mnemonic::*, Register::*};

#[cfg(feature = "x86")]
pub(crate) fn encode_push_32(
    x: &PushOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_x86_register;

    if x.register.is_32() {
        buf.push(0x50 + opcode_offset_for_x86_register(x.register));
        *pc += 1;
    } else if x.register.is_xmm() {
        encode_push_xmm_32(x, pc, buf)?;
    } else if x.register.is_ymm() {
        encode_push_ymm_32(x, pc, buf)?;
    } else if x.register.is_zmm() {
        encode_push_zmm_32(x, pc, buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }

    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_push_xmm_32(
    x: &PushOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_xmm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_xmm_register_x86(x.register);

    // Prepare the buffer for the 8-byte write
    let old_len = buf.len();
    buf.reserve(8); // Reserve space for 8 bytes
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // SUB ESP, 16
        // MOVDQU [ESP], reg
        let instruction: u64 =
            0x2404_7F0F_F310_EC83_u64.to_le() | (((reg_offset << 3) as u64) << 48);
        write_unaligned(ptr as *mut u64, instruction.to_le());
        buf.set_len(old_len + 8);
    }

    *pc += 8; // Update the program counter
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_push_ymm_32(
    x: &PushOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_ymm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_ymm_register_x86(x.register);

    // Prepare the buffer for the 8-byte write
    let old_len = buf.len();
    buf.reserve(8); // Reserve space for 8 bytes
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // SUB ESP, 16
        // MOVDQU [ESP], reg
        let instruction: u64 =
            0x2404_7FFE_C520_EC83_u64.to_le() | (((reg_offset << 3) as u64) << 48);
        write_unaligned(ptr as *mut u64, instruction.to_le());
        buf.set_len(old_len + 8);
    }

    *pc += 8; // Update the program counter
    Ok(())
}

#[cold]
#[inline]
#[cfg(feature = "x86")]
fn encode_push_zmm_32(
    x: &PushOperation<x86::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x86::Register>> {
    use crate::common::jit_instructions::helpers::opcode_offset_for_zmm_register_x86;
    use core::ptr::write_unaligned;

    let reg_offset = opcode_offset_for_zmm_register_x86(x.register);

    // Prepare the buffer for the 8-byte write
    let old_len = buf.len();
    buf.reserve(10); // Reserve space for 8 bytes
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // SUB ESP, 16
        // VMOVDQU64 [ESP], reg
        write_unaligned(ptr as *mut u64, 0x7F48_FEF1_6240_EC83_u64.to_le());
        write_unaligned(
            ptr.add(8) as *mut u16,
            0x2404_u16.to_le() | ((reg_offset << 3) as u16),
        );
        buf.set_len(old_len + 10);
    }

    *pc += 10; // Update the program counter
    Ok(())
}

#[cfg(feature = "x64")]
pub(crate) fn encode_push_64(
    x: &PushOperation<x64::Register>,
    pc: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<(), X86jitError<x64::Register>> {
    if x.register.is_64() {
        *pc += EncoderRequest::new64(PUSH)
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_xmm() {
        *pc += EncoderRequest::new64(SUB)
            .add_operand(RSP)
            .add_operand(16)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(MOVDQU)
            .add_operand(mem!(xmmword ptr [RSP]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_ymm() {
        *pc += EncoderRequest::new64(SUB)
            .add_operand(RSP)
            .add_operand(32)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(VMOVDQU)
            .add_operand(mem!(ymmword ptr [RSP]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_zmm() {
        *pc += EncoderRequest::new64(SUB)
            .add_operand(RSP)
            .add_operand(64)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new64(VMOVDQU64)
            .add_operand(mem!(zmmword ptr [RSP]))
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
    use crate::common::util::test_utilities::assert_encode;
    use crate::instructions::push::encode_push_32;
    use crate::instructions::push::encode_push_64;
    use crate::x64;
    use crate::x86;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(x64::Register::rax, "50")]
    #[case(x64::Register::xmm0, "4883ec10f30f7f0424")]
    #[case(x64::Register::ymm0, "4883ec20c5fe7f0424")]
    #[case(x64::Register::zmm0, "4883ec4062f1fe487f0424")]
    fn push_x64(#[case] register: x64::Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Push { register };

        encode_push_64(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }

    #[rstest]
    #[case(x86::Register::eax, "50")]
    #[case(x86::Register::xmm0, "83ec10f30f7f0424")]
    #[case(x86::Register::xmm1, "83ec10f30f7f0c24")]
    #[case(x86::Register::xmm2, "83ec10f30f7f1424")]
    #[case(x86::Register::xmm3, "83ec10f30f7f1c24")]
    #[case(x86::Register::xmm4, "83ec10f30f7f2424")]
    #[case(x86::Register::xmm5, "83ec10f30f7f2c24")]
    #[case(x86::Register::xmm6, "83ec10f30f7f3424")]
    #[case(x86::Register::xmm7, "83ec10f30f7f3c24")]
    #[case(x86::Register::ymm0, "83ec20c5fe7f0424")]
    #[case(x86::Register::ymm1, "83ec20c5fe7f0c24")]
    #[case(x86::Register::ymm2, "83ec20c5fe7f1424")]
    #[case(x86::Register::ymm3, "83ec20c5fe7f1c24")]
    #[case(x86::Register::ymm4, "83ec20c5fe7f2424")]
    #[case(x86::Register::ymm5, "83ec20c5fe7f2c24")]
    #[case(x86::Register::ymm6, "83ec20c5fe7f3424")]
    #[case(x86::Register::ymm7, "83ec20c5fe7f3c24")]
    #[case(x86::Register::zmm0, "83ec4062f1fe487f0424")]
    #[case(x86::Register::zmm1, "83ec4062f1fe487f0c24")]
    #[case(x86::Register::zmm2, "83ec4062f1fe487f1424")]
    #[case(x86::Register::zmm3, "83ec4062f1fe487f1c24")]
    #[case(x86::Register::zmm4, "83ec4062f1fe487f2424")]
    #[case(x86::Register::zmm5, "83ec4062f1fe487f2c24")]
    #[case(x86::Register::zmm6, "83ec4062f1fe487f3424")]
    #[case(x86::Register::zmm7, "83ec4062f1fe487f3c24")]
    fn push_x86(#[case] register: x86::Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Push { register };

        encode_push_32(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
