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
    #[cfg(target_pointer_width = "64")]
    use crate::x64::jit::JitX64;
    use crate::x86::jit::JitX86;
    use reloaded_hooks_portable::api::jit::{compiler::Jit, operation_aliases::*};
    use rstest::rstest;

    #[rstest]
    // Regular relative jump
    #[case(0x7FFFFFFF, "e9faffff7f", 0)]
    // Jump into low memory, by overflow
    #[case(1, "e9fc0f0000", 0xFFFFF000)]
    // Jump into high memory by underflow
    #[case((u32::MAX - 0xF) as isize, "e9ebffffff", 0)]
    // TODO: ^ bug in Iced, this should be 'ebee', but still valid.
    // Jump into high memory by underflow, max offset
    #[case((u32::MAX - 0x7FFFFFFF) as isize, "e9fbffff7f", 0)]
    fn jmp_relative_x86(#[case] offset: isize, #[case] expected_encoded: &str, #[case] pc: usize) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset as usize))];
        let result = JitX86::compile(pc, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }

    #[rstest]
    // Regular relative jump
    #[case(0x7FFFFFFF, "e9faffff7f", 0)]
    // Jump into low memory, by overflow
    #[case(1, "e9fc0f0000", usize::MAX - 0xFFF)]
    // Jump into high memory by underflow
    #[case((usize::MAX - 0xF) as isize, "ebee", 0)]
    // Jump into high memory by underflow, max offset
    #[case((usize::MAX - 0x7FFFFFFA) as isize, "e900000080", 0)]
    // TODO: ^ bug in Iced, does not encode correctly when `0x7FFFFFFF`, off by 5 error.
    #[cfg(target_pointer_width = "64")]
    fn jmp_relative_x64(#[case] offset: isize, #[case] expected_encoded: &str, #[case] pc: usize) {
        let operations = vec![Op::JumpRelative(JumpRel::new(offset as usize))];
        let result = JitX64::compile(pc, &operations);
        assert_eq!(expected_encoded, hex::encode(result.unwrap()));
    }
}
