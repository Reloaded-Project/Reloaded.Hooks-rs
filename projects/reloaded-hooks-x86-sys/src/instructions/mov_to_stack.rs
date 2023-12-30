extern crate alloc;
use crate::all_registers::AllRegisters;
use crate::common::jit_common::X86jitError;
use crate::mov_item_to_stack;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::compiler::JitError;
use reloaded_hooks_portable::api::jit::mov_to_stack_operation::MovToStackOperation;

pub(crate) fn encode_mov_to_stack(
    a: &mut CodeAssembler,
    x: &MovToStackOperation<AllRegisters>,
) -> Result<(), X86jitError<AllRegisters>> {
    if x.register.is_32() {
        mov_item_to_stack!(a, x.register, x.stack_offset, as_iced_32, mov);
    } else if x.register.is_64() && cfg!(feature = "x64") {
        #[cfg(feature = "x64")]
        mov_item_to_stack!(a, x.register, x.stack_offset, as_iced_64, mov);
    } else if x.register.is_xmm() {
        mov_item_to_stack!(a, x.register, x.stack_offset, as_iced_xmm, movdqu);
    } else if x.register.is_ymm() {
        mov_item_to_stack!(a, x.register, x.stack_offset, as_iced_ymm, vmovdqu);
    } else if x.register.is_zmm() {
        mov_item_to_stack!(a, x.register, x.stack_offset, as_iced_zmm, vmovdqu8);
    } else {
        return Err(JitError::InvalidRegister(x.register).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x64::{self, jit::JitX64};
    use reloaded_hooks_portable::api::jit::compiler::Jit;
    use reloaded_hooks_portable::api::jit::operation::Operation;
    use rstest::*;

    #[rstest]
    #[case::x64_64bit(x64::Register::rax, 16, "4889442410")]
    #[case::x64_xmm(x64::Register::xmm0, 16, "f30f7f442410")]
    fn test_encode_mov_to_stack_x64(
        #[case] register: x64::Register,
        #[case] offset: i32,
        #[case] expected: &str,
    ) {
        let a = JitX64::compile(
            0,
            &[Operation::MovToStack(MovToStackOperation::new(
                offset, register,
            ))],
        );

        let result = hex::encode(a.unwrap());
        assert_eq!(expected, result);
    }
}
