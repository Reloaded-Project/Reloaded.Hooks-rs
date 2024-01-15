extern crate alloc;
use crate::{
    common::{traits::ToZydis, util::get_stolen_instructions::ZydisInstruction},
    instructions::{call_relative::encode_call_relative, jump_relative::encode_jump_relative},
};
use alloc::vec::Vec;
use core::fmt::Debug;
use reloaded_hooks_portable::api::{
    jit::{
        call_relative_operation::CallRelativeOperation,
        jump_relative_operation::JumpRelativeOperation,
    },
    rewriter::code_rewriter::CodeRewriterError,
};
use zydis::EncoderRequest;
use zydis::Mnemonic::*;

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#relative-branches)
#[cfg(feature = "x64")]
pub(crate) fn patch_relative_branch_64<TRegister: Debug + ToZydis + Default>(
    instruction: &ZydisInstruction,
    _instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: Option<TRegister>,
    buf: &mut Vec<u8>,
    is_call: bool,
) -> Result<(), CodeRewriterError> {
    // If the branch offset is within 2GiB, do no action
    // because Iced will handle it for us on re-encode.
    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    *source_address += instruction.length as usize;
    let delta = (target.wrapping_sub(*dest_address as u64)) as i64;
    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        // Hot path, we can re-encode because it's in jump range.
        make_relative_branch::<TRegister>(is_call, target, dest_address, buf);
        return Ok(());
    }

    // Cold path. We need to re-encode as an absolute jump.
    // This can only ever be true for x64
    let scratch_reg =
        scratch_register.ok_or_else(|| CodeRewriterError::NoScratchRegister(Default::default()))?;

    *dest_address += EncoderRequest::new64(MOV)
        .add_operand(scratch_reg.to_zydis())
        .add_operand(target)
        .encode_extend(buf)
        .unwrap();

    let opc = if !is_call { JMP } else { CALL };

    *dest_address += EncoderRequest::new64(opc)
        .add_operand(scratch_reg.to_zydis())
        .encode_extend(buf)
        .unwrap();
    Ok(())
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#relative-branches)
#[cfg(feature = "x86")]
pub(crate) fn patch_relative_branch_32<TRegister: Debug + ToZydis + Default>(
    instruction: &ZydisInstruction,
    _instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    _scratch_register: Option<TRegister>,
    buf: &mut Vec<u8>,
    is_call: bool,
) -> Result<(), CodeRewriterError> {
    // If the branch offset is within 2GiB, do no action
    // because Iced will handle it for us on re-encode.
    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    *source_address += instruction.length as usize;

    // This is x86 so we can branch from anywhere to anywhere.
    make_relative_branch::<TRegister>(is_call, target, dest_address, buf);
    Ok(())
}

#[inline(always)]
fn make_relative_branch<TRegister: Debug + ToZydis + Default>(
    is_call: bool,
    target: u64,
    dest_address: &mut usize,
    buf: &mut Vec<u8>,
) {
    if is_call {
        encode_call_relative::<TRegister>(
            &CallRelativeOperation {
                target_address: target as usize,
            },
            dest_address,
            buf,
        )
        .unwrap();
    } else {
        encode_jump_relative::<TRegister>(
            &JumpRelativeOperation {
                target_address: target as usize,
                scratch_register: Default::default(), // not used
            },
            dest_address,
            buf,
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::util::test_utilities::test_relocate_instruction, x64, x86};
    use rstest::rstest;

    #[rstest]
    #[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_abs_jmp_i8("eb02", 0x80000000, 0, "48b80400008000000000ffe0")] // jmp +2 -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_jmp_i32("e9fb0f0000", 0x80000000, 0, "48b80010008000000000ffe0")] // jmp +4091 -> mov rax, 0x80001000 + jmp rax
    // Some tests when in upper bytes
    #[case::simple_branch_upper64("eb02", 0x8000000000001000, 0x8000000000000000, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_abs_jmp_8b_upper64(
        "eb02",
        0x8000000080000000,
        0x8000000000000000,
        "48b80400008000000080ffe0"
    )] // jmp +2 -> mov rax, 0x8000000000000004 + jmp rax
    #[case::to_abs_jmp_i32_upper64(
        "e9fb0f0000",
        0x8000000080000000,
        0x8000000000000000,
        "48b80010008000000080ffe0"
    )] // jmp +2 -> mov rax, 0x8000000080001000 + jmp rax
    fn relocate_64b_jmp(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_relocate_instruction(
            instructions,
            old_address,
            new_address,
            expected,
            Some(x64::Register::rax),
            true,
            patch_relative_branch_64_jmp, // the function being tested
        );
    }

    #[rstest]
    #[case::to_abs_call_i32("e8fb0f0000", 0x80000000, 0, "48b80010008000000000ffd0")] // call +4091 -> mov rax, 0x80001000 + call rax
    #[case::to_absolute_call("e8ffffffff", 0x80000000, 0, "48b80400008000000000ffd0")] // call -1 -> call rax, 0x80000004 + call rax
    #[case::to_absolute_call_i32_upper64(
        "e8fb0f0000",
        0x8000000080000000,
        0x8000000000000000,
        "48b80010008000000080ffd0"
    )] // call +2 -> mov rax, 0x8000000080001000 + call rax
    fn relocate_64b_call(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_relocate_instruction(
            instructions,
            old_address,
            new_address,
            expected,
            Some(x64::Register::rax),
            true,
            patch_relative_branch_64_call, // the function being tested
        );
    }

    #[rstest]
    #[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::simple_branch_with_underflow("ebf4", 0, 4096, "e9f1efffff")] // jmp -12 -> jmp -4098
    // Some tests when in upper bytes
    #[case::simple_branch_upper32("eb02", 0x80001000, 0x80000000, "e9ff0f0000")] // jmp +2 -> jmp +4098
    fn relocate_32b_jmp(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_relocate_instruction(
            instructions,
            old_address,
            new_address,
            expected,
            Some(x86::Register::eax),
            false,
            patch_relative_branch_32_jmp, // the function being tested
        );
    }

    #[rstest]
    #[case::simple_branch("e8ffffffff", 4096, 0, "e8ff0f0000")] // call +2 -> call +4098
    // Some tests when in upper bytes
    #[case::simple_branch_upper32("e8ffffffff", 0x80001000, 0x80000000, "e8ff0f0000")] // call +2 -> call +4098
    fn relocate_32b_call(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_relocate_instruction(
            instructions,
            old_address,
            new_address,
            expected,
            Some(x86::Register::eax),
            false,
            patch_relative_branch_32_call, // the function being tested
        );
    }

    fn patch_relative_branch_64_jmp(
        instruction: &ZydisInstruction,
        instruction_bytes: &[u8],
        dest_address: &mut usize,
        source_address: &mut usize,
        scratch_register: Option<x64::Register>,
        buf: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        patch_relative_branch_64(
            instruction,
            instruction_bytes,
            dest_address,
            source_address,
            scratch_register,
            buf,
            false,
        )
    }

    fn patch_relative_branch_64_call(
        instruction: &ZydisInstruction,
        instruction_bytes: &[u8],
        dest_address: &mut usize,
        source_address: &mut usize,
        scratch_register: Option<x64::Register>,
        buf: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        patch_relative_branch_64(
            instruction,
            instruction_bytes,
            dest_address,
            source_address,
            scratch_register,
            buf,
            true,
        )
    }

    fn patch_relative_branch_32_jmp(
        instruction: &ZydisInstruction,
        instruction_bytes: &[u8],
        dest_address: &mut usize,
        source_address: &mut usize,
        scratch_register: Option<x86::Register>,
        buf: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        patch_relative_branch_32(
            instruction,
            instruction_bytes,
            dest_address,
            source_address,
            scratch_register,
            buf,
            false,
        )
    }

    fn patch_relative_branch_32_call(
        instruction: &ZydisInstruction,
        instruction_bytes: &[u8],
        dest_address: &mut usize,
        source_address: &mut usize,
        scratch_register: Option<x86::Register>,
        buf: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        patch_relative_branch_32(
            instruction,
            instruction_bytes,
            dest_address,
            source_address,
            scratch_register,
            buf,
            true,
        )
    }
}
