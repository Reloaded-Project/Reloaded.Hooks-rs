extern crate alloc;

use super::instruction_rewrite_result::InstructionRewriteResult;
use crate::instructions::{add_immediate::AddImmediate, adr::Adr, mov_immediate::MovImmediate};
use alloc::boxed::Box;

/// Produces an `InstructionRewriteResult` which represents the best possible way to
/// encode the provided value as an immediate move instruction for the given destination register.
///
/// # Parameters
///
/// * `destination`: The destination register.
/// * `value`: The immediate value to be moved to the destination.
pub(crate) fn emit_mov_const_to_reg(destination: u8, value: usize) -> InstructionRewriteResult {
    // Determine leading zeroes using native lzcnt instruction
    let leading_zeros = value.leading_zeros();
    let used_bits = usize::BITS - leading_zeros;

    match used_bits {
        0..=16 => InstructionRewriteResult::MovImmediate1(
            MovImmediate::new_movz(true, destination, value as u16, 0)
                .unwrap()
                .0
                .to_le(),
        ),
        17..=32 => InstructionRewriteResult::MovImmediate2(
            MovImmediate::new_movz(true, destination, value as u16, 0)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
        ),
        33..=48 => InstructionRewriteResult::MovImmediate3(
            MovImmediate::new_movz(true, destination, value as u16, 0)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)
                .unwrap()
                .0
                .to_le(),
        ),
        49..=64 => InstructionRewriteResult::MovImmediate4(Box::new([
            MovImmediate::new_movz(true, destination, value as u16, 0)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 48) as u16, 48)
                .unwrap()
                .0
                .to_le(),
        ])),
        _ => unreachable!(), // This case should never be reached unless platform is >64 bits
    }
}

// TODO: Optimize this in case any of the values are 0. Those can be zero'd by initial movz.
// This is a very rare occurrence, so not optimized for now.

/// Produces an `InstructionRewriteResult` which represents the best possible way to
/// encode the provided value as an immediate move instruction for the given destination register.
///
/// # Parameters
///
/// * `destination`: The destination register.
/// * `value`: The immediate value to be moved to the destination.
pub(crate) fn emit_mov_upper_48_bits_const_to_reg(
    destination: u8,
    value: usize,
) -> InstructionRewriteResult {
    // Determine leading zeroes using native lzcnt instruction
    let leading_zeros = value.leading_zeros();
    let used_bits = usize::BITS - leading_zeros;

    match used_bits {
        0..=16 => InstructionRewriteResult::MovImmediate1(
            MovImmediate::new_movz(true, destination, 0, 0)
                .unwrap()
                .0
                .to_le(),
        ),
        17..=32 => InstructionRewriteResult::MovImmediate1(
            MovImmediate::new_movz(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
        ),
        33..=48 => InstructionRewriteResult::MovImmediate2(
            MovImmediate::new_movz(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)
                .unwrap()
                .0
                .to_le(),
        ),
        49..=64 => InstructionRewriteResult::MovImmediate3(
            MovImmediate::new_movz(true, destination, (value >> 16) as u16, 16)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 32) as u16, 32)
                .unwrap()
                .0
                .to_le(),
            MovImmediate::new_movk(true, destination, (value >> 48) as u16, 48)
                .unwrap()
                .0
                .to_le(),
        ),
        _ => unreachable!(), // This case should never be reached unless platform is >64 bits
    }
}

/// Loads an address from an offset that's within 4GiB of current PC (specified as [`instruction_address`]) into
/// a register
///
/// # Parameters
///
/// * `instruction_address`: The address of the instruction.
/// * `target_address`: The address to load.
/// * `destination`: The destination register.
///
/// # Returns
///
/// A tuple where the first element is either [`InstructionRewriteResult::Adrp(u32)`] or
/// [`InstructionRewriteResult::AdrpAndAdd(u32, u32)`].
pub(crate) fn load_address_4g(
    instruction_address: usize,
    target_address: usize,
    destination: u8,
) -> InstructionRewriteResult {
    // Assemble ADRP + ADD if out of range.
    // This will error if our address is too far.
    let adrp_pc = instruction_address & !4095; // round down to page
    let adrp_target_address = target_address & !4095; // round down to page
    let adrp_offset = adrp_target_address.wrapping_sub(adrp_pc) as isize;

    let adrp = Adr::new_adrp(destination, adrp_offset as i64).unwrap().0;

    let remainder = target_address - adrp_target_address;
    if remainder > 0 {
        let add = AddImmediate::new(true, destination, destination, remainder as u16)
            .unwrap()
            .0;

        return InstructionRewriteResult::AdrpAndAdd(adrp, add);
    }

    InstructionRewriteResult::Adrp(adrp)
}

/// Loads an address from an offset that's within 4GiB of current PC (specified as [`instruction_address`]) into
/// a register
///
/// # Parameters
///
/// * `instruction_address`: The address of the instruction.
/// * `target_address`: The address to load.
/// * `destination`: The destination register.
///
/// # Returns
///
/// A tuple where the first element is either [`InstructionRewriteResult::Adrp(u32)`] and the second element is the remainder after
/// calculating the offset for the `ADRP` instruction.
pub(crate) fn load_address_4g_with_remainder(
    instruction_address: usize,
    target_address: usize,
    destination: u8,
) -> (u32, u32) {
    // Assemble ADRP + ADD if out of range.
    // This will error if our address is too far.
    let adrp_pc = instruction_address & !4095; // round down to page
    let adrp_target_address = target_address & !4095; // round down to page
    let adrp_offset = adrp_target_address.wrapping_sub(adrp_pc) as isize;

    let adrp = Adr::new_adrp(destination, adrp_offset as i64).unwrap().0;
    let remainder = target_address - adrp_target_address;
    (adrp.to_le(), remainder as u32)
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::ToHexString;

    use super::*;

    #[test]
    fn test_emit_mov_16_bits_or_less() {
        let destination = 0;
        let value = 0x1AAA;
        let result = emit_mov_const_to_reg(destination, value);
        if let InstructionRewriteResult::MovImmediate1(instr1) = result {
            assert_eq!(instr1.to_hex_string(), "405583d2");
        } else {
            panic!("Expected MovImmediate1 result");
        }
    }

    #[test]
    fn test_emit_mov_17_to_32_bits() {
        let destination = 1;
        let value = 0x1AAAAAAA;
        let result = emit_mov_const_to_reg(destination, value);
        if let InstructionRewriteResult::MovImmediate2(instr1, instr2) = result {
            assert_eq!(instr1.to_hex_string(), "415595d2");
            assert_eq!(instr2.to_hex_string(), "4155a3f2");
        } else {
            panic!("Expected MovImmediate2 result");
        }
    }

    #[test]
    fn test_emit_mov_33_to_48_bits() {
        let destination = 2;
        let value = 0x2AAA1AAAAAAA;
        let result = emit_mov_const_to_reg(destination, value);
        if let InstructionRewriteResult::MovImmediate3(instr1, instr2, instr3) = result {
            assert_eq!(instr1.to_hex_string(), "425595d2");
            assert_eq!(instr2.to_hex_string(), "4255a3f2");
            assert_eq!(instr3.to_hex_string(), "4255c5f2");
        } else {
            panic!("Expected MovImmediate3 result");
        }
    }

    #[test]
    fn test_emit_mov_49_to_64_bits() {
        let destination = 3;
        let value = 0x3AAA2AAA1AAAAAAA;
        let result = emit_mov_const_to_reg(destination, value);
        if let InstructionRewriteResult::MovImmediate4(bx) = result {
            assert_eq!(bx[0].to_hex_string(), "435595d2");
            assert_eq!(bx[1].to_hex_string(), "4355a3f2");
            assert_eq!(bx[2].to_hex_string(), "4355c5f2");
            assert_eq!(bx[3].to_hex_string(), "4355e7f2");
        } else {
            panic!("Expected MovImmediate4 result");
        }
    }
}
