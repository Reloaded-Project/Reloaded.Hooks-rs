extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::jit_common::{convert_error, ARCH_NOT_SUPPORTED};
use alloc::string::ToString;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpAbs};

pub(crate) fn encode_jump_absolute(
    a: &mut CodeAssembler,
    x: &JumpAbs<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 64 {
        let target_reg = x.scratch_register.as_iced_64()?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = x.scratch_register.as_iced_32()?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
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

    #[rstest]
    #[case(x64::Register::rax, 0x12345678, "48b87856341200000000ffe0")]
    fn jump_absolute_x64(
        #[case] scratch_register: x64::Register,
        #[case] target_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX64 {};
        let operations = vec![Op::JumpAbsolute(JumpAbs {
            scratch_register,
            target_address,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(x86::Register::eax, 0x12345678, "b878563412ffe0")]
    fn jump_absolute_x86(
        #[case] scratch_register: x86::Register,
        #[case] target_address: usize,
        #[case] expected_encoded: &str,
    ) {
        let mut jit = JitX86 {};
        let operations = vec![Op::JumpAbsolute(JumpAbs {
            scratch_register,
            target_address,
        })];
        let result = jit.compile(0, &operations);
        assert!(result.is_ok());
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[test]
    #[should_panic]
    fn out_of_range_x86() {
        let mut jit = JitX86 {};

        let operations = vec![Op::JumpRelative(JumpRel::new(usize::MAX))];
        let result = jit.compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn is_relative_to_eip() {
        let mut jit = JitX86 {};

        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Op::JumpRelative(JumpRel::new(0x80000005))];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        assert_eq!("e9fbffff7f", hex::encode(result.unwrap()));
    }

    #[test]
    #[should_panic]
    fn out_of_range_x64() {
        let mut jit = JitX64 {};

        let operations = vec![Op::JumpRelative(JumpRel::new(usize::MAX))];
        let result = jit.compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn is_relative_to_rip() {
        let mut jit = JitX64 {};

        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Op::JumpRelative(JumpRel::new(0x80000005))];
        let result = jit.compile(5, &operations);
        assert!(result.is_ok());
        assert_eq!("e9fbffff7f", hex::encode(result.as_ref().unwrap()));
    }
}
