extern crate alloc;

use super::aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult};
use crate::instructions::{add_immediate::AddImmediate, adr::Adr};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the `ADR` instruction for a new address.
///
/// The `ADR` instruction in ARM architectures computes the address of a label and writes
/// it to the destination register. This function is intended to modify the `ADR` instruction's
/// encoding to adjust for a new memory location, making it compatible with relocation or code injection.
///
/// # Parameters
///
/// * `instruction`: The original `ADR` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `ADR` instruction.
/// * `new_address`: The new target address to which the instruction needs to point.
///
/// # Behavior
///
/// The ADR(P) instruction is rewritten as one of the following:
/// - ADRP
/// - ADRP + ADD
/// - MOV (1-4 instructions)
///
/// # Safety
///
/// Ensure that the provided `instruction` is a valid `ADR` opcode. Providing invalid opcodes or
/// incorrectly assuming that a different kind of instruction is an `ADR` can lead to unintended results.
pub(crate) fn rewrite_adr(
    instruction: u32,
    old_address: usize,
    new_address: usize,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let mut adr = Adr(instruction.to_le());
    let old_target = adr.extract_address(old_address);

    // Compute the difference between the new address and old target.
    let delta = (old_target as isize).wrapping_sub(new_address as isize);
    let delta_page = ((old_target >> 12) as isize).wrapping_sub((new_address >> 12) as isize);

    // ADRP case
    if (-0x100000..=0xFFFFF).contains(&delta) {
        // Note: Item was originally ADRP, but is now in ADR range.
        adr.set_is_pageaddress(false);
        adr.set_raw_offset(delta as i32);
        Ok(InstructionRewriteResult::Adr(adr.0.to_le()))
    } else if (-0x100000..=0xFFFFF).contains(&delta_page) {
        // Otherwise if the item is within 4GiB range, assemble as ADRP + ADD.
        adr.set_is_pageaddress(true);
        adr.set_raw_offset(delta_page as i32);
        if (old_target & 0xfff) != 0 {
            let add =
                AddImmediate::new(true, adr.rd(), adr.rd(), (old_target & 0xfff) as u16).unwrap();
            return Ok(InstructionRewriteResult::AdrpAndAdd(
                adr.0.to_le(),
                add.0.to_le(),
            ));
        }

        return Ok(InstructionRewriteResult::Adrp(adr.0.to_le()));
    } else {
        // If the item is out of range, emit this as an immediate move.
        Ok(emit_mov_const_to_reg(adr.rd(), old_target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::ToHexString;
    use rstest::rstest;

    #[rstest]
    // Note: We reverse byte order of left due to little endian.
    // Move ADRP x0, 0x101000 to ADR x0, 0xFFFFF
    #[case::adrp_to_adr(0x000800B0_u32.to_be(), 0, 4097, "e0ff7f70")]
    // Move ADRP x0, 0x101000 to ADRP x0, 0x102000 + ADD x0, x0, 1
    #[case::within_4gib_range(0x000800B0_u32.to_be(), 4097, 0, "000800d000040091")]
    // Move ADRP x0, 0x101000 to ADRP x0, 0x102000
    #[case::within_4gib_range_no_offset (0x000800B0_u32.to_be(), 4096, 0, "000800d0")]
    // Move [PC = 0x100000000], ADRP, x0, 0x101000 to MOV IMMEDIATE 0x100101000
    #[case::out_of_range(0x000800B0_u32.to_be(), 0x100000000, 0, "000082d20002a0f22000c0f2")]
    fn test_rewrite_adr(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_adr(old_instruction, old_address, new_address).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
