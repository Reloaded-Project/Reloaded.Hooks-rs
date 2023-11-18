extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{convert_error, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, registers as iced_regs, CodeAssembler};
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::Pop};

macro_rules! multi_pop_item {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        match $a.bitness() {
            32 => {
                $a.$op($reg.$convert_method()?, dword_ptr(iced_regs::esp) + $offset)
                    .map_err(convert_error)?;
            }
            64 => {
                $a.$op($reg.$convert_method()?, qword_ptr(iced_regs::rsp) + $offset)
                    .map_err(convert_error)?;
            }
            _ => {
                return Err(JitError::ThirdPartyAssemblerError(
                    ARCH_NOT_SUPPORTED.to_string(),
                ));
            }
        }
    };
}

pub(crate) fn encode_multi_pop(
    a: &mut CodeAssembler,
    ops: &[Pop<AllRegisters>],
) -> Result<(), JitError<AllRegisters>> {
    // Note: It is important that we do MOV in ascending address order, to help CPU caching :wink:

    // Start from the top of the reserved space.
    let mut current_offset = 0;
    for x in ops {
        if x.register.is_32() {
            multi_pop_item!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() {
            multi_pop_item!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    // Release the space.
    let total_space = ops.iter().map(|x| x.register.size()).sum::<usize>();
    a.add(iced_regs::esp, total_space as i32)
        .map_err(convert_error)?;

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
    #[case::compile_multi_push_basic_regs_x64(JitX64 {}, vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::rax),
    Push::new(x64::Register::rbx),
    Push::new(x64::Register::rcx),
])], "83ec1848890c2448895c24084889442410")]
    // XMM register push for x64
    #[case::compile_multi_push_xmm_x64(JitX64 {}, vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::xmm0),
    Push::new(x64::Register::xmm1),
    Push::new(x64::Register::xmm2),
])], "83ec30f30f7f1424f30f7f4c2410f30f7f442420")]
    // YMM register push for x64
    #[case::compile_multi_push_ymm_x64(JitX64 {}, vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::ymm0),
    Push::new(x64::Register::ymm1),
    Push::new(x64::Register::ymm2),
])], "83ec60c5fe7f1424c5fe7f4c2420c5fe7f442440")]
    // ZMM register push for x64
    #[case::compile_multi_push_zmm_x64(JitX64 {}, vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::zmm0),
    Push::new(x64::Register::zmm1),
    Push::new(x64::Register::zmm2),
])], "81ecc000000062f17f487f142462f17f487f4c240162f17f487f442402")]
    // Mixed register push for x64
    #[case::compile_multi_push_mixed_x64(JitX64 {}, vec![Op::MultiPush(smallvec![
    Push::new(x64::Register::rax),
    Push::new(x64::Register::xmm0),
    Push::new(x64::Register::ymm1),
])], "83ec38c5fe7f0c24f30f7f4424204889442430")]
    fn test_compile_x64(
        #[case] mut jit: JitX64,
        #[case] operations: Vec<Op<x64::Register>>,
        #[case] expected_hex: &str,
    ) {
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_hex, hex::encode(result.as_ref().unwrap()));
    }

    #[rstest]
    // Basic register push for x86
    #[case::compile_multi_push_basic_regs_x86(JitX86 {}, vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::eax),
    Push::new(x86::Register::ebx),
    Push::new(x86::Register::ecx),
])], "83ec0c890c24895c240489442408")]
    // XMM register push for x86
    #[case::compile_multi_push_xmm_x86(JitX86 {}, vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::xmm0),
    Push::new(x86::Register::xmm1),
    Push::new(x86::Register::xmm2),
])], "83ec30f30f7f1424f30f7f4c2410f30f7f442420")]
    // YMM register push for x86
    #[case::compile_multi_push_ymm_x86(JitX86 {}, vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::ymm0),
    Push::new(x86::Register::ymm1),
    Push::new(x86::Register::ymm2),
])], "83ec60c5fe7f1424c5fe7f4c2420c5fe7f442440")]
    // ZMM register push for x86
    #[case::compile_multi_push_zmm_x86(JitX86 {}, vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::zmm0),
    Push::new(x86::Register::zmm1),
    Push::new(x86::Register::zmm2),
])], "81ecc000000062f17f487f142462f17f487f4c240162f17f487f442402")]
    // Mixed register push for x86
    #[case::compile_multi_push_mixed_x86(JitX86 {}, vec![Op::MultiPush(smallvec![
    Push::new(x86::Register::eax),
    Push::new(x86::Register::xmm0),
    Push::new(x86::Register::ymm1),
])], "83ec34c5fe7f0c24f30f7f44242089442430")]
    fn test_compile_x86(
        #[case] mut jit: JitX86,
        #[case] operations: Vec<Op<x86::Register>>,
        #[case] expected_hex: &str,
    ) {
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_hex, hex::encode(result.as_ref().unwrap()));
    }
}
