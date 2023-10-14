extern crate alloc;
use alloc::string::ToString;
use core::ops::RangeInclusive;

use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

use crate::instructions::{add_immediate::AddImmediate, adr::Adr};

use super::aarch64_rewriter::{emit_mov_const_to_reg, InstructionRewriteResult};

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
/// The function modifies the provided `ADR` instruction encoding, recalculating any relative
/// offsets so that when executed from the new location, the instruction computes the correct address.
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

    if !adr.is_adrp() {
        // ADR case
        if (-0x100000..=0xFFFFF).contains(&delta) {
            // If the item is within single ADR range, encode as single ADR.
            adr.set_is_pageaddress(false);
            adr.set_raw_offset(delta as i32);
            Ok(InstructionRewriteResult::Adr(adr.0.to_le()))
        } else if (-0x100000..=0xFFFFF).contains(&delta_page) {
            // Otherwise if the item is within 4GiB range, assemble as ADRP + ADD.
            adr.set_is_pageaddress(true);
            adr.set_raw_offset(delta_page as i32);
            let add =
                AddImmediate::new(true, adr.rd(), adr.rd(), (old_target & 0xfff) as u16).unwrap();
            return Ok(InstructionRewriteResult::AdrpAndAdd(
                adr.0.to_le(),
                add.0.to_le(),
            ));
        } else {
            // If the item is out of range, emit this as an immediate move.
            return Ok(emit_mov_const_to_reg(adr.rd(), new_address));
        }
    } else {
        // ADRP case
        if (-0x100000..=0xFFFFF).contains(&delta) {
            // Note: Item was originally ADRP, but is now in ADR range.
            adr.set_is_pageaddress(false);
            adr.set_raw_offset(delta as i32);
            Ok(InstructionRewriteResult::Adr(adr.0.to_le()))
        } else {
            // If the item is out of range, emit this as an immediate move.
            Ok(emit_mov_const_to_reg(adr.rd(), new_address))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::to_hex_string;
    use rstest::rstest;

    #[rstest]
    // Note: We reverse byte order of left due to little endian.
    // Move ADRP x0, 0x101000 to ADR x0, 0xFFFFF
    #[case::adrp_to_adr(0x000800B0_u32.to_be(), 0, 4097, "e0ff7f70")]
    //#[case::within_4gib_range(0x123456, 500000, 5000000000, "expected_hex_for_within_4gib_range")]
    //#[case::out_of_range(0x123456, 500000, 7000000, "expected_hex_for_out_of_range")]
    //#[case::adrp_case(0x123456, 500000, 5000000000, "expected_hex_for_adrp_case")] // Mock ADRP instruction
    fn test_rewrite_adr(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_adr(old_instruction, old_address, new_address).unwrap();
        assert_eq!(to_hex_string(result), expected_hex);
    }
}
