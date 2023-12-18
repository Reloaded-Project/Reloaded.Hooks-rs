use crate::all_registers::AllRegisters;
use crate::common::jit_common::convert_error;
use iced_x86::code_asm::CodeAssembler;
use reloaded_hooks_portable::api::jit::{compiler::JitError, operation_aliases::JumpRel};

pub(crate) fn encode_jump_relative(
    a: &mut CodeAssembler,
    x: &JumpRel<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    a.jmp(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{x64::jit::JitX64, x86::jit::JitX86};
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    #[case(-0x80000000, "e9fbffff7f")] // underflow
    fn jmp_relative_underflow_x86(#[case] offset: isize, #[case] expected_encoded: &str) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset as usize))];
        let result = JitX86::compile(0, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(-0x10, "e9ebffffff")] // underflow
    fn jmp_relative_underflow_x86_into_highmem(
        #[case] offset: isize,
        #[case] expected_encoded: &str,
    ) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset as usize))];
        let result = JitX86::compile(0, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(1, "e9fc0f0000")] // overflow
    fn jmp_relative_overflow_x86(#[case] offset: isize, #[case] expected_encoded: &str) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset as usize))];
        let result = JitX86::compile(0xFFFFF000, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x7FFFFFFF, "e9faffff7f")]
    fn jmp_relative_x86(#[case] offset: usize, #[case] expected_encoded: &str) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset))];
        let result = JitX86::compile(0, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    #[case(0x7FFFFFFF, "e9faffff7f")]
    fn jmp_relative_x64(#[case] offset: usize, #[case] expected_encoded: &str) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset))];
        let result = JitX64::compile(0, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[test]
    #[should_panic]
    fn out_of_range_x86() {
        // Note: This fails inside Iced :/
        let operations = vec![Op::CallRelative(CallRel::new(usize::MAX))];
        let result = JitX86::compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn relative_to_eip_x86() {
        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Op::CallRelative(CallRel::new(0x80000005))];
        let result = JitX86::compile(5, &operations);
        assert_eq!("e8fbffff7f", hex::encode(result.unwrap()));
    }

    #[test]
    #[should_panic]
    fn out_of_range_x64() {
        // Note: This fails inside Iced :/
        let operations = vec![Op::CallRelative(CallRel::new(usize::MAX))];
        let result = JitX64::compile(0, &operations);
        assert!(result.is_err());
    }

    #[test]
    fn is_relative_to_rip_x64() {
        // Verifies that the JIT compiles a relative call that branches towards target_address
        // This is verified by branching to an address outside of the 2GB range and setting
        // Instruction Pointer of assembled code to make it within range.
        let operations = vec![Op::CallRelative(CallRel::new(0x80000005))];
        let result = JitX64::compile(5, &operations);
        assert_eq!("e8fbffff7f", hex::encode(result.unwrap()));
    }
}
