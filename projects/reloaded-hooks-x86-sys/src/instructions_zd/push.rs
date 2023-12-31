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
    if x.register.is_32() {
        *pc += EncoderRequest::new32(PUSH)
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_xmm() {
        *pc += EncoderRequest::new32(SUB)
            .add_operand(ESP)
            .add_operand(16)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new32(MOVDQU)
            .add_operand(mem!(xmmword ptr [ESP]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_ymm() {
        *pc += EncoderRequest::new32(SUB)
            .add_operand(ESP)
            .add_operand(32)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new32(VMOVDQU)
            .add_operand(mem!(ymmword ptr [ESP]))
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else if x.register.is_zmm() {
        *pc += EncoderRequest::new32(SUB)
            .add_operand(ESP)
            .add_operand(64)
            .encode_extend(buf)?;

        *pc += EncoderRequest::new32(VMOVDQU8)
            .add_operand(mem!(zmmword ptr [ESP]))
            .add_operand(K0)
            .add_operand(x.register.to_zydis())
            .encode_extend(buf)?;
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }

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

        *pc += EncoderRequest::new64(VMOVDQU8)
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
    use crate::instructions_zd::push::encode_push_32;
    use crate::instructions_zd::push::encode_push_64;
    use crate::x64;
    use crate::x86;
    use reloaded_hooks_portable::api::jit::operation_aliases::*;
    use rstest::rstest;

    #[rstest]
    #[case(x64::Register::rax, "50")]
    #[case(x64::Register::xmm0, "4883ec10f30f7f0424")]
    #[case(x64::Register::ymm0, "4883ec20c5fe7f0424")]
    #[case(x64::Register::zmm0, "4883ec4062f17f487f0424")]
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
    #[case(x86::Register::ymm0, "83ec20c5fe7f0424")]
    #[case(x86::Register::zmm0, "83ec4062f17f487f0424")]
    fn push_x86(#[case] register: x86::Register, #[case] expected_encoded: &str) {
        let mut pc = 0;
        let mut buf = Vec::new();
        let operation = Push { register };

        encode_push_32(&operation, &mut pc, &mut buf).unwrap();
        assert_encode(expected_encoded, &buf, pc);
    }
}
