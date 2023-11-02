extern crate alloc;

use super::{
    aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult},
    b::rewrite_b_4gib,
};
use crate::instructions::{b::B, branch_register::BranchRegister, cbz::Cbz};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the `CBZ` (Compare and Branch on Zero) instruction for a new address.
///
/// # Parameters
///
/// * `instruction`: The original `CBZ` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `CBZ` instruction.
/// * `new_address`: The new target address of the instruction.
/// * `scratch_register`: Specifies the register to use as a scratch when the target is too far for direct branching.
///
/// # Behavior
///
/// The Branch instruction is rewritten as one of the following:
/// - CBZ
/// - CBZ <skip> + B
/// - CBZ <skip> + ADRP + BR
/// - CBZ <skip> + ADRP + ADD + BR
/// - CBZ <skip> + MOV to Register + Branch Register
///
/// # Safety
///
/// Ensure that the provided `instruction` is a valid `Bcc` opcode. Supplying invalid opcodes or
/// wrongly assuming that a different type of instruction is a `Bcc` can result in unintended behaviors.
pub(crate) fn rewrite_cbz(
    instruction: u32,
    old_address: usize,
    new_address: usize,
    scratch_register: Option<u8>,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let orig_ins = Cbz(instruction.to_le());
    let orig_target = (old_address as isize).wrapping_add(orig_ins.offset() as isize);
    let delta = orig_target.wrapping_sub(new_address as isize);

    // Output as another CBZ if within 1MiB range
    if (-0x100000..=0xFFFFF).contains(&delta) {
        return Ok(InstructionRewriteResult::Cbz(
            Cbz::assemble(
                delta as i32,
                orig_ins.rt(),
                orig_ins.is_64bit(),
                orig_ins.non_zero(),
            )
            .unwrap()
            .0
            .to_le(),
        ));
    }

    // Output as:
    // - CBZ <invert condition> <skip next instruction>
    // - Branch Relative

    // + 4 for 'next instruction', since we are placing a CBZ first.
    let delta_next_instruction = orig_target.wrapping_sub(new_address as isize + 4);
    if (-0x8000000..=0x7FFFFFF).contains(&delta_next_instruction) {
        // Note: invert non_zero here
        let instr1 = Cbz::assemble(8, orig_ins.rt(), orig_ins.is_64bit(), !orig_ins.non_zero())
            .unwrap()
            .0
            .to_le();
        let instr2 = B::assemble_b(delta_next_instruction as i32)
            .unwrap()
            .0
            .to_le();
        return Ok(InstructionRewriteResult::CbzAndBranch(instr1, instr2));
    }

    // Output as:
    // - CBZ <skip>
    // - ADRP
    // - ADD (Optional)
    // - Branch Register
    let scratch_reg = scratch_register
        .ok_or_else(|| CodeRewriterError::NoScratchRegister("rewrite_cbz".to_string()))?;

    // + 4 for 'next instruction', since we are placing a BCC first.
    if (-0x100000000..=0xFFFFFFFF).contains(&delta_next_instruction) {
        let result =
            rewrite_b_4gib(new_address + 4, orig_target as usize, scratch_reg, false).unwrap();

        match result {
            InstructionRewriteResult::AdrpAndAddAndBranch(a, b, c) => {
                return Ok(InstructionRewriteResult::CbzAndAdrpAndAddAndBranch(
                    Box::new([
                        Cbz::assemble(16, orig_ins.rt(), orig_ins.is_64bit(), !orig_ins.non_zero())
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
                return Ok(InstructionRewriteResult::CbzAndAdrpAndBranch(
                    Cbz::assemble(12, orig_ins.rt(), orig_ins.is_64bit(), !orig_ins.non_zero())
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
    // - CBZ <skip>
    // - MOV to Register
    // - Branch Register
    let mov_instr = emit_mov_const_to_reg(scratch_reg, orig_target as usize);
    let instr1 = Cbz::assemble(
        8 + mov_instr.size_bytes() as i32,
        orig_ins.rt(),
        orig_ins.is_64bit(),
        !orig_ins.non_zero(),
    )
    .unwrap();

    let instr2 = BranchRegister::new_br(scratch_reg);
    let mut result = Vec::new();
    result.push(instr1.0.to_le());
    mov_instr.append_to_buffer(&mut result);
    result.push(instr2.0.to_le());
    Ok(InstructionRewriteResult::CbzAndBranchAbsolute(
        result.into_boxed_slice(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::ToHexString;
    use rstest::rstest;

    #[rstest]
    // [Within 1MiB] || cbz x0, #4096 -> cbz x0, #8192 || (We add 4096 to branch offset.)
    #[case::simple_cbz(0x008000B4_u32.to_be(), 8192, 4096, "000001b4")]
    // [Within 128MiB] || cbz x0, #4096 -> cbnz + b || (We add 4096 to branch offset.)
    #[case::b(0x008000B4_u32.to_be(), 0x8000000, 4096, "400000b5ffffff15")]
    // [Within 4GiB + 4096 aligned] || cbnz -> cbnz + adrp + br || (We add +128MiB to branch offset)
    #[case::adrp_and_br(0x008000B4_u32.to_be(), 0x8000000, 0, "600000b5110004b020021fd6")]
    // [Within 4GiB] || cbz x0, #4096 -> cbnz + adrp + add + br || (We add +128MiB + 512 to branch offset)
    #[case::adrp_and_add_and_br(0x008000B4_u32.to_be(), 0x8000512, 0, "800000b5110004b0314a149120021fd6")]
    // [Last Resort, Move] || cbz x0, #4096 -> cbnz + mov immediate + br || (We add +4GiB to branch offset)
    #[case::mov_and_br(0x008000B4_u32.to_be(), 0x100000000, 0, "a00000b5110082d21100a0f23100c0f220021fd6")]
    fn test_rewrite_cbz(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_cbz(old_instruction, old_address, new_address, Some(17)).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
