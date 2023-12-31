extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{convert_error, ARCH_NOT_SUPPORTED};
use crate::mov_item_to_stack;
use alloc::string::ToString;
use iced_x86::code_asm::{registers as iced_regs, CodeAssembler};
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::operation_aliases::Push;

pub(crate) fn encode_multi_push(
    a: &mut CodeAssembler,
    ops: &[Push<AllRegisters>],
) -> Result<(), X86jitError<AllRegisters>> {
    // Calculate space to reserve.
    let mut space_needed = 0;

    for x in ops {
        space_needed += x.register.size();
    }

    // Reserve the space.
    if a.bitness() == 64 && cfg!(feature = "x64") {
        a.sub(iced_regs::rsp, space_needed as i32)?;
    } else if a.bitness() == 32 && cfg!(feature = "x86") {
        a.sub(iced_regs::esp, space_needed as i32)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(ARCH_NOT_SUPPORTED.to_string()).into());
    }

    // Push the items.
    let mut current_offset = 0;
    for x in ops.iter().rev() {
        // Loop through the operations in reverse
        if x.register.is_32() {
            mov_item_to_stack!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() && cfg!(feature = "x64") {
            #[cfg(feature = "x64")]
            mov_item_to_stack!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            mov_item_to_stack!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            mov_item_to_stack!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            mov_item_to_stack!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        x64::{self, jit::JitX64},
        x86::{self, jit::JitX86},
    };
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;
    use smallvec::smallvec;

    #[rstest]
    // Basic register push for x64
    #[case::compile_multi_push_basic_regs_x64(vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::rax),
    Push::new(x64::Register::rbx),
    Push::new(x64::Register::rcx),
    ])], "4883ec1848890c2448895c24084889442410")]
    // XMM register push for x64
    #[case::compile_multi_push_xmm_x64(vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::xmm0),
    Push::new(x64::Register::xmm1),
    Push::new(x64::Register::xmm2),
    ])], "4883ec30f30f7f1424f30f7f4c2410f30f7f442420")]
    // YMM register push for x64
    #[case::compile_multi_push_ymm_x64(vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::ymm0),
    Push::new(x64::Register::ymm1),
    Push::new(x64::Register::ymm2),
    ])], "4883ec60c5fe7f1424c5fe7f4c2420c5fe7f442440")]
    // ZMM register push for x64
    #[case::compile_multi_push_zmm_x64(vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::zmm0),
    Push::new(x64::Register::zmm1),
    Push::new(x64::Register::zmm2),
    ])], "4881ecc000000062f17f487f142462f17f487f4c240162f17f487f442402")]
    // Mixed register push for x64
    #[case::compile_multi_push_mixed_x64(vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::rax),
    Push::new(x64::Register::xmm0),
    Push::new(x64::Register::ymm1),
    ])], "4883ec38c5fe7f0c24f30f7f4424204889442430")]
    fn multi_push_x64(#[case] operations: Vec<Op<x64::Register>>, #[case] expected_hex: &str) {
        let result = JitX64::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_hex, hex::encode(result.as_ref().unwrap()));
    }

    #[rstest]
    // Basic register push for x86
    #[case::compile_multi_push_basic_regs_x86(vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::eax),
    Push::new(x86::Register::ebx),
    Push::new(x86::Register::ecx),
    ])], "83ec0c890c24895c240489442408")]
    // XMM register push for x86
    #[case::compile_multi_push_xmm_x86(vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::xmm0),
    Push::new(x86::Register::xmm1),
    Push::new(x86::Register::xmm2),
    ])], "83ec30f30f7f1424f30f7f4c2410f30f7f442420")]
    // YMM register push for x86
    #[case::compile_multi_push_ymm_x86(vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::ymm0),
    Push::new(x86::Register::ymm1),
    Push::new(x86::Register::ymm2),
    ])], "83ec60c5fe7f1424c5fe7f4c2420c5fe7f442440")]
    // ZMM register push for x86
    #[case::compile_multi_push_zmm_x86(vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::zmm0),
    Push::new(x86::Register::zmm1),
    Push::new(x86::Register::zmm2),
    ])], "81ecc000000062f17f487f142462f17f487f4c240162f17f487f442402")]
    // Mixed register push for x86
    #[case::compile_multi_push_mixed_x86(vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::eax),
    Push::new(x86::Register::xmm0),
    Push::new(x86::Register::ymm1),
    ])], "83ec34c5fe7f0c24f30f7f44242089442430")]
    fn multi_push_x86(#[case] operations: Vec<Op<x86::Register>>, #[case] expected_hex: &str) {
        let result = JitX86::compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_hex, hex::encode(result.as_ref().unwrap()));
    }
}
