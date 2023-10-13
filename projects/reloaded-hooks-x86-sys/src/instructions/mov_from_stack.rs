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
    if a.bitness() == 32 {
        let ptr = dword_ptr(iced_x86::Register::ESP) + x.stack_offset;
        a.mov(x.target.as_iced_32()?, ptr)
    } else if a.bitness() == 64 {
        let ptr = qword_ptr(iced_x86::Register::RSP) + x.stack_offset;
        a.mov(x.target.as_iced_64()?, ptr)
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

    #[test]
    fn mov_from_stack_x86() {
        let mut jit = JitX86 {};

        let operations = vec![Op::MovFromStack(MovFromStackOperation {
            stack_offset: 4,
            target: x86::Register::eax,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("8b442404", hex::encode(result.unwrap()));
    }

    #[test]
    fn mov_from_stack_x64() {
        let mut jit = JitX64 {};

        let operations = vec![Op::MovFromStack(MovFromStack::new(4, x64::Register::rax))];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!("488b442404", hex::encode(result.as_ref().unwrap()));
    }
}
