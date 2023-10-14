extern crate alloc;
use alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};

use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::MovFromStack};

use crate::{
    all_registers::AllRegisters,
    jit_common::{convert_error, ARCH_NOT_SUPPORTED},
};

pub(crate) fn encode_mov_from_stack(
    a: &mut CodeAssembler,
    x: &MovFromStack<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    let base_ptr = if a.bitness() == 64 {
        qword_ptr(iced_x86::Register::RSP) + x.stack_offset
    } else {
        dword_ptr(iced_x86::Register::ESP) + x.stack_offset
    };

    if x.target.is_32() {
        a.mov(x.target.as_iced_32()?, base_ptr)
    } else if x.target.is_64() {
        a.mov(x.target.as_iced_64()?, base_ptr)
    } else if x.target.is_xmm() {
        a.movups(x.target.as_iced_xmm()?, base_ptr)
    } else if x.target.is_ymm() {
        a.vmovups(x.target.as_iced_ymm()?, base_ptr)
    } else if x.target.is_zmm() {
        a.vmovups(x.target.as_iced_zmm()?, base_ptr)
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        x64::{self, jit::JitX64},
        x86::{self, jit::JitX86},
    };
    use reloaded_hooks_portable::api::jit::{
        compiler::Jit, mov_from_stack_operation::MovFromStackOperation, operation_aliases::*,
    };
    use rstest::rstest;

    #[rstest]
    #[case(x86::Register::eax, "8b442404")]
    #[case(x86::Register::xmm0, "0f10442404")]
    #[case(x86::Register::ymm0, "c5fc10442404")]
    #[case(x86::Register::zmm0, "62f17c4810842404000000")]
    fn mov_from_stack_x86(#[case] target: x86::Register, #[case] expected_encoded: &str) {
        let mut jit = JitX86 {};
        let operations = vec![Op::MovFromStack(MovFromStackOperation {
            stack_offset: 4,
            target,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x64::Register::rax, "488b442404")]
    #[case(x64::Register::xmm0, "0f10442404")]
    #[case(x64::Register::ymm0, "c5fc10442404")]
    #[case(x64::Register::zmm0, "62f17c4810842404000000")]
    fn mov_from_stack_x64(#[case] target: x64::Register, #[case] expected_encoded: &str) {
        let mut jit = JitX64 {};
        let operations = vec![Op::MovFromStack(MovFromStack::new(4, target))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.as_ref().unwrap()));
    }
}