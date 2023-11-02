extern crate alloc;

use super::aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult};
use crate::instructions::{
    add_immediate::AddImmediate, adr::Adr, b::B, branch_register::BranchRegister,
};
use alloc::{string::ToString, vec::Vec};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the `B` (Branch) instruction for a new address.
///
/// # Parameters
///
/// * `instruction`: The original `B` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `B` instruction.
/// * `new_address`: The new target address of the instruction.
/// * `scratch_register`: Specifies the register to use as a scratch when the target is too far for direct branching.
///
/// # Behaviour
///
/// The Branch instruction is rewritten as one of the following:
/// - B
/// - ADRP + BR
/// - ADRP + ADD + BR
/// - MOV <immediate> + BR
///
/// # Safety
///
/// Ensure that the provided `instruction` is a valid `Bcc` opcode. Supplying invalid opcodes or
/// wrongly assuming that a different type of instruction is a `Bcc` can result in unintended behaviours.
pub(crate) fn rewrite_b(
    instruction: u32,
    old_address: usize,
    new_address: usize,
    scratch_register: Option<u8>,
    link: bool,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let orig_ins = B(instruction.to_le());
    let orig_target = (old_address as isize).wrapping_add(orig_ins.offset() as isize);
    let new_offset = orig_target.wrapping_sub(new_address as isize);

    // If within 4GiB, use ADRP + ADD + BR.
    if !(-0x02000000..=0x01FFFFFF).contains(&new_offset) {
        let scratch_reg = scratch_register
            .ok_or_else(|| CodeRewriterError::NoScratchRegister("rewrite_b".to_string()))?;

        // Otherwise if further away from 4G, use MOVZ + MOVK + BR.
        if !(-0x40000000..=0x3FFFFFFF).contains(&new_offset) {
            // MOV <immediate> + BR
            return rewrite_b_immediate(orig_target as usize, scratch_reg, link);
        }

        // ADRP + (ADD) + BR
        return rewrite_b_4gib(new_address, orig_target as usize, scratch_reg, link);
    }

    // Rewrite as regular branch.
    // - B
    let instruction = B::assemble(new_offset as i32, link);
    Ok(InstructionRewriteResult::B(instruction.unwrap().0.to_le()))
}

pub(crate) fn rewrite_b_4gib(
    new_address: usize,
    target_address: usize,
    scratch_register: u8,
    link: bool,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    // Assemble ADRP + ADD if out of range.
    // This will error if our address is too far.
    let adrp_pc = new_address & !4095; // round down to page
    let adrp_target_address = target_address & !4095; // round down to page
    let adrp_offset = adrp_target_address.wrapping_sub(adrp_pc) as isize;

    let adrp = Adr::new_adrp(scratch_register, adrp_offset as i64)
        .unwrap()
        .0;

    let branch = BranchRegister::new_b(scratch_register, link).0;
    let remainder = target_address - adrp_target_address;
    if remainder > 0 {
        let add = AddImmediate::new(true, scratch_register, scratch_register, remainder as u16)
            .unwrap()
            .0;

        return Ok(InstructionRewriteResult::AdrpAndAddAndBranch(
            adrp, add, branch,
        ));
    }

    Ok(InstructionRewriteResult::AdrpAndBranch(adrp, branch))
}

pub(crate) fn rewrite_b_immediate(
    orig_target: usize,
    scratch_register: u8,
    link: bool,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let mov_instr = emit_mov_const_to_reg(scratch_register, orig_target); // Assuming emit_mov_const_to_reg can handle this as well
    let b_instr = BranchRegister::new_b(scratch_register, link);

    let mut result = Vec::new();
    mov_instr.append_to_buffer(&mut result);
    result.push(b_instr.0.to_le());

    Ok(InstructionRewriteResult::BranchAbsolute(
        result.into_boxed_slice(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::ToHexString;
    use rstest::rstest;

    #[rstest]
    // [Within 128MiB] || b #4096 -> b #8192 || (We add 4096 to branch offset.)
    #[case::simple_b(0x00040014_u32.to_be(), 8192, 4096, "00080014")]
    // [Within 4GiB] || b #4096 -> adrp + br || (We add +128MiB to branch offset)
    #[case::adrp_and_br(0x00040014_u32.to_be(), 0x8000000, 0, "110004b020021fd6")]
    // [Within 4GiB] || b #4096 -> adrp + add + br || (We add +128MiB + 512 to branch offset)
    #[case::adrp_and_add_and_br(0x00040014_u32.to_be(), 0x8000512, 0, "110004b0314a149120021fd6")]
    // [Last Resort, Move] || b #4096 -> mov immediate + br x17 || (We add +4GiB to branch offset)
    #[case::mov_and_br(0x00040014_u32.to_be(), 0x100000000, 0, "110082d21100a0f23100c0f220021fd6")]
    fn test_rewrite_b(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_b(old_instruction, old_address, new_address, Some(17), false).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }

    #[rstest]
    // [Within 128MiB] || b #4096 -> b #8192 || (We add 4096 to branch offset.)
    #[case::simple_b(0x00040094_u32.to_be(), 8192, 4096, "00080094")]
    // [Within 4GiB] || b #4096 -> adrp + br || (We add +128MiB to branch offset)
    #[case::adrp_and_br(0x00040094_u32.to_be(), 0x8000000, 0, "110004b020023fd6")]
    // [Within 4GiB] || b #4096 -> adrp + add + br || (We add +128MiB + 512 to branch offset)
    #[case::adrp_and_add_and_br(0x00040094_u32.to_be(), 0x8000512, 0, "110004b0314a149120023fd6")]
    // [Last Resort, Move] || b #4096 -> mov immediate + br x17 || (We add +4GiB to branch offset)
    #[case::mov_and_br(0x00040094_u32.to_be(), 0x100000000, 0, "110082d21100a0f23100c0f220023fd6")]
    fn test_rewrite_bl(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_b(old_instruction, old_address, new_address, Some(17), true).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
