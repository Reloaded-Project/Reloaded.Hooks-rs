extern crate alloc;

use super::{
    aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult},
    b::rewrite_b_4gib,
};
use crate::instructions::{b::B, bcc::Bcc, branch_register::BranchRegister};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the `Bcc` (Branch Conditional) instruction for a new address.
///
/// The `Bcc` instruction in ARM architectures performs a conditional branch based on
/// specific condition flags. This function is designed to modify the `Bcc` instruction's
/// encoding to adjust for a new memory location.
///
/// # Parameters
///
/// * `instruction`: The original `Bcc` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `Bcc` instruction.
/// * `new_address`: The new address of the instruction.
/// * `scratch_register`: Specifies the register to use as a scratch when the target is too far for direct branching.
///
/// # Behaviour
///
/// The Branch Conditional instruction is rewritten as one of the following:
/// - BCC
/// - BCC <skip> + B
/// - BCC <skip> + ADRP + Add + Branch Register
/// - BCC <skip> + MOV to Register + Branch Register
///
/// # Safety
///
/// Ensure that the provided `instruction` is a valid `Bcc` opcode. Supplying invalid opcodes or
/// wrongly assuming that a different type of instruction is a `Bcc` can result in unintended behaviours.
pub(crate) fn rewrite_bcc(
    instruction: u32,
    old_address: usize,
    new_address: usize,
    scratch_register: Option<u8>,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let orig_ins = Bcc(instruction.to_le());
    let orig_target = (old_address as isize).wrapping_add(orig_ins.offset());
    let delta = orig_target.wrapping_sub(new_address as isize);

    // Output as another BCC if within 1MiB range
    if (-0x100000..=0xFFFFF).contains(&delta) {
        return Ok(InstructionRewriteResult::Bcc(
            Bcc::assemble_bcc(orig_ins.condition(), delta as i32)
                .unwrap()
                .0
                .to_le(),
        ));
    }

    // Condition table: https://www.davespace.co.uk/arm/introduction-to-arm/conditional.html (also true for AArch64)
    // Note: The lowest bit 1 allows for inverting the condition.

    // Output as:
    // - BCC <invert condition> <skip next instruction>
    // - Branch Relative

    // + 4 for 'next instruction', since we are placing a BCC first.
    let delta_next_instruction = orig_target.wrapping_sub(new_address as isize + 4);
    if (-0x8000000..=0x7FFFFFF).contains(&delta_next_instruction) {
        let instr1 = Bcc::assemble_bcc(orig_ins.condition() ^ 1, 8)
            .unwrap()
            .0
            .to_le();
        let instr2 = B::assemble_b(delta_next_instruction as i32)
            .unwrap()
            .0
            .to_le();
        return Ok(InstructionRewriteResult::BccAndBranch(instr1, instr2));
    }

    // Output as:
    // - BCC <skip>
    // - ADRP
    // - ADD (Optional)
    // - Branch Register
    let scratch_reg = scratch_register
        .ok_or_else(|| CodeRewriterError::NoScratchRegister("rewrite_bcc".to_string()))?;

    // + 4 for 'next instruction', since we are placing a BCC first.
    if (-0x100000000..=0xFFFFFFFF).contains(&delta_next_instruction) {
        let result =
            rewrite_b_4gib(new_address + 4, orig_target as usize, scratch_reg, false).unwrap();

        match result {
            InstructionRewriteResult::AdrpAndAddAndBranch(a, b, c) => {
                return Ok(InstructionRewriteResult::BccAndAdrpAndAddAndBranch(
                    Box::new([
                        Bcc::assemble_bcc(orig_ins.condition() ^ 1, 16)
                            .unwrap()
                            .0
                            .to_le(),
                        a,
                        b,
                        c,
                    ]),
                ))
            }
            InstructionRewriteResult::AdrpAndBranch(a, b) => {
                return Ok(InstructionRewriteResult::BccAndAdrpAndBranch(
                    Bcc::assemble_bcc(orig_ins.condition() ^ 1, 12)
                        .unwrap()
                        .0
                        .to_le(),
                    a,
                    b,
                ))
            }
            _ => unreachable!(),
        }
    }

    // Output as:
    // - BCC <skip>
    // - MOV to Register
    // - Branch Register
    let mov_instr = emit_mov_const_to_reg(scratch_reg, orig_target as usize);
    let instr1 =
        Bcc::assemble_bcc(orig_ins.condition() ^ 1, 8 + mov_instr.size_bytes() as i32).unwrap();
    let instr2 = BranchRegister::new_br(scratch_reg);
    let mut result = Vec::new();
    result.push(instr1.0.to_le());
    mov_instr.append_to_buffer(&mut result);
    result.push(instr2.0.to_le());
    Ok(InstructionRewriteResult::BccAndBranchAbsolute(
        result.into_boxed_slice(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::ToHexString;
    use rstest::rstest;

    #[rstest]
    // [Within 1MiB] || b.eq #4 -> b.eq #-4092
    #[case::simple_bcc(0x20000054_u32.to_be(), 0, 4096, "2080ff54")]
    // [Within 128MiB] || b.eq #0 -> b.ne #8 + b #-0x80000000
    #[case::bcc_and_branch(0x00000054_u32.to_be(), 0, 0x8000000 - 4, "4100005400000016")]
    // [Within 4GiB + 4096 aligned] || b.eq #0 -> b.ne #12 + adrp x17, #-0x8000000 + br x17
    #[case::bcc_with_adrp(0x00000054_u32.to_be(), 0, 0x8000000, "610000541100fc9020021fd6")]
    // [Within 4GiB] || b.eq #512 -> b.ne #16 + adrp x17, #0x8000000 + add x17, #512 + br x17
    #[case::bcc_with_adrp_and_add(0x00100054_u32.to_be(), 0x8000000, 0, "81000054110004903102089120021fd6")]
    // [Last Resort] || b.eq #0 -> b.ne #12 + movz x17, #0 + br x17
    #[case::bcc_out_of_range(0x00000054_u32.to_be(), 0, 0x100000000, "61000054110080d220021fd6")]
    fn test_rewrite_bcc(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_bcc(old_instruction, old_address, new_address, Some(17)).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
