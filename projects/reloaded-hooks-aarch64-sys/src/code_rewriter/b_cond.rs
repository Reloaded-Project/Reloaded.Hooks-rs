extern crate alloc;

use super::aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult};
use crate::instructions::{b::B, bcc::Bcc, branch_register::BranchRegister};
use alloc::string::ToString;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the `Bcc` (Branch Conditional) instruction for a new address.
///
/// The `Bcc` instruction in ARM architectures performs a conditional branch based on
/// specific condition flags. This function is designed to modify the `Bcc` instruction's
/// encoding to adjust for a new memory location, making it suitable for relocation or code injection.
///
/// # Parameters
///
/// * `instruction`: The original `Bcc` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `Bcc` instruction.
/// * `new_address`: The new target address to which the instruction needs to point.
/// * `scratch_register`: An optional parameter specifying the register to use as a scratch
///   for temporary operations when the target is too far for direct branching. Defaults to x17
///   if not provided.
///
/// # Behavior
///
/// The Branch Conditional instruction is rewritten as one of the following:
/// - BCC
/// - BCC <skip> + B
/// - BCC <skip> + MOV to Register + Branch Register
///
/// # Safety
///
/// Ensure that the provided `instruction` is a valid `Bcc` opcode. Supplying invalid opcodes or
/// wrongly assuming that a different type of instruction is a `Bcc` can result in unintended behaviors.
///
/// # Errors
///
/// This function can return an error encapsulated in a `CodeRewriterError` if any problem
/// occurs during the rewriting process, such as an unsupported scenario.
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
    // - MOV to Register
    // - Branch Register
    let scratch_reg = scratch_register
        .ok_or_else(|| CodeRewriterError::NoScratchRegister("rewrite_bcc".to_string()))?;

    let mov_instr = emit_mov_const_to_reg(scratch_reg, orig_target as usize); // Assuming emit_mov_const_to_reg can handle this as well
    let instr1 = Bcc::assemble_bcc(orig_ins.condition() ^ 1, 8).unwrap();
    let instr2 = BranchRegister::new_br(scratch_reg);
    let mut result = Vec::new();
    mov_instr.append_to_buffer(&mut result);
    result.push(instr1.0.to_le());
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
    // [Last Resort] || b.eq #0 -> movz x17, #0 + b.ne #0xc + br x17
    #[case::bcc_out_of_range(0x00000054_u32.to_be(), 0, 0x8000000, "110080d24100005420021fd6")]
    fn test_rewrite_bcc(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_bcc(old_instruction, old_address, new_address, Some(17)).unwrap(); // using x17 as the scratch register for this example
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
