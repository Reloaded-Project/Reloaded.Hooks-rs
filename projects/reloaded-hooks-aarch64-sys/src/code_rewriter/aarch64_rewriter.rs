extern crate alloc;

use crate::instructions::{add_immediate::AddImmediate, adr::Adr, mov_immediate::MovImmediate};
use alloc::{boxed::Box, vec::Vec};
use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;

pub(crate) fn rewrite_code_aarch64(
    _old_address: *const u8,
    _old_address_size: usize,
    _new_address: *const u8,
    _out_address: *mut u8,
    _out_address_size: usize,
    _buf: Box<dyn Buffer>,
) -> i32 {
    todo!()
}

/// Stores the possible rewritten instruction outcomes.
///
/// Any rewritten instruction or group of instructions may be emitted as one of these instructions.
///
/// # Remarks
///
/// Entries are limited to 16 bytes per entry. Items larger than 16 bytes (3 u32s) are boxed
/// https://nnethercote.github.io/perf-book/type-sizes.html?highlight=match#boxed-slices
pub(crate) enum InstructionRewriteResult {
    None,
    Adr(u32),
    Adrp(u32),
    AdrpAndAdd(u32, u32),
    AdrpAndBranch(u32, u32),
    AdrpAndAddAndBranch(u32, u32, u32),
    B(u32),
    Bcc(u32),
    BccAndBranch(u32, u32),
    BccAndAdrpAndBranch(u32, u32, u32),
    BccAndAdrpAndAddAndBranch(Box<[u32; 4]>),
    BccAndBranchAbsolute(Box<[u32]>),
    BranchAbsolute(Box<[u32]>),
    Cbz(u32),
    CbzAndBranch(u32, u32),
    CbzAndAdrpAndBranch(u32, u32, u32),
    CbzAndAdrpAndAddAndBranch(Box<[u32; 4]>),
    CbzAndBranchAbsolute(Box<[u32]>),
    LdrLiteral(u32),
    AdrpAndLdrUnsignedOffset(u32, u32),
    MovImmediateAndLdrLiteral(Box<[u32]>),
    MovImmediate1(u32), // in instruction count order
    MovImmediate2(u32, u32),
    MovImmediate3(u32, u32, u32),
    MovImmediate4(Box<[u32; 4]>),
    Tbz(u32),
    TbzAndBranch(u32, u32),
    TbzAndAdrpAndBranch(u32, u32, u32),
    TbzAndAdrpAndAddAndBranch(Box<[u32; 4]>),
    TbzAndBranchAbsolute(Box<[u32]>),
}

impl InstructionRewriteResult {
    /// Appends the instructions represented by `self` into the provided buffer.
    ///
    /// # Parameters
    ///
    /// * `buf`: The buffer to which the instruction(s) will be appended.
    pub(crate) fn append_to_buffer(&self, buf: &mut Vec<u32>) {
        match self {
            InstructionRewriteResult::Adr(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::Adrp(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::AdrpAndAdd(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::Bcc(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::BccAndBranch(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::BccAndBranchAbsolute(boxed) => {
                buf.extend_from_slice(boxed.as_ref())
            }
            InstructionRewriteResult::MovImmediate1(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::MovImmediate2(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::MovImmediate3(inst1, inst2, inst3) => {
                buf.push(*inst1);
                buf.push(*inst2);
                buf.push(*inst3);
            }
            InstructionRewriteResult::MovImmediate4(bx) => {
                buf.push(bx[0]);
                buf.push(bx[1]);
                buf.push(bx[2]);
                buf.push(bx[3]);
            }
            InstructionRewriteResult::AdrpAndBranch(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::AdrpAndAddAndBranch(inst1, inst2, inst3) => {
                buf.push(*inst1);
                buf.push(*inst2);
                buf.push(*inst3);
            }
            InstructionRewriteResult::B(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::BranchAbsolute(boxed) => {
                buf.extend_from_slice(boxed.as_ref())
            }
            InstructionRewriteResult::BccAndAdrpAndBranch(inst1, inst2, inst3) => {
                buf.push(*inst1);
                buf.push(*inst2);
                buf.push(*inst3);
            }
            InstructionRewriteResult::BccAndAdrpAndAddAndBranch(bx) => {
                buf.push(bx[0]);
                buf.push(bx[1]);
                buf.push(bx[2]);
                buf.push(bx[3]);
            }
            InstructionRewriteResult::Cbz(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::CbzAndBranch(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::CbzAndAdrpAndBranch(inst1, inst2, inst3) => {
                buf.push(*inst1);
                buf.push(*inst2);
                buf.push(*inst3);
            }
            InstructionRewriteResult::CbzAndAdrpAndAddAndBranch(bx) => {
                buf.push(bx[0]);
                buf.push(bx[1]);
                buf.push(bx[2]);
                buf.push(bx[3]);
            }
            InstructionRewriteResult::CbzAndBranchAbsolute(boxed) => {
                buf.extend_from_slice(boxed.as_ref())
            }
            InstructionRewriteResult::LdrLiteral(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::AdrpAndLdrUnsignedOffset(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::MovImmediateAndLdrLiteral(boxed) => {
                buf.extend_from_slice(boxed.as_ref())
            }
            InstructionRewriteResult::None => {}
            InstructionRewriteResult::Tbz(inst) => {
                buf.push(*inst);
            }
            InstructionRewriteResult::TbzAndBranch(inst1, inst2) => {
                buf.push(*inst1);
                buf.push(*inst2);
            }
            InstructionRewriteResult::TbzAndAdrpAndBranch(inst1, inst2, inst3) => {
                buf.push(*inst1);
                buf.push(*inst2);
                buf.push(*inst3);
            }
            InstructionRewriteResult::TbzAndAdrpAndAddAndBranch(bx) => {
                buf.push(bx[0]);
                buf.push(bx[1]);
                buf.push(bx[2]);
                buf.push(bx[3]);
            }
            InstructionRewriteResult::TbzAndBranchAbsolute(boxed) => {
                buf.extend_from_slice(boxed.as_ref())
            }
        }
    }

    /// Returns the size in bytes for the rewrite result.
    pub(crate) fn size_bytes(&self) -> usize {
        match self {
            InstructionRewriteResult::Adr(_) => 4,
            InstructionRewriteResult::Adrp(_) => 4,
            InstructionRewriteResult::AdrpAndAdd(_, _) => 8,
            InstructionRewriteResult::Bcc(_) => 4,
            InstructionRewriteResult::BccAndBranch(_, _) => 8,
            InstructionRewriteResult::BccAndBranchAbsolute(boxed) => boxed.len() * 4,
            InstructionRewriteResult::MovImmediate1(_) => 4,
            InstructionRewriteResult::MovImmediate2(_, _) => 8,
            InstructionRewriteResult::MovImmediate3(_, _, _) => 12,
            InstructionRewriteResult::MovImmediate4(_) => 16,
            InstructionRewriteResult::AdrpAndBranch(_, _) => 8,
            InstructionRewriteResult::AdrpAndAddAndBranch(_, _, _) => 12,
            InstructionRewriteResult::B(_) => 4,
            InstructionRewriteResult::BranchAbsolute(boxed) => boxed.len() * 4,
            InstructionRewriteResult::BccAndAdrpAndBranch(_, _, _) => 12,
            InstructionRewriteResult::BccAndAdrpAndAddAndBranch(_) => 16,
            InstructionRewriteResult::Cbz(_) => 4,
            InstructionRewriteResult::CbzAndBranch(_, _) => 8,
            InstructionRewriteResult::CbzAndAdrpAndBranch(_, _, _) => 12,
            InstructionRewriteResult::CbzAndAdrpAndAddAndBranch(_) => 16,
            InstructionRewriteResult::CbzAndBranchAbsolute(boxed) => boxed.len() * 4,
            InstructionRewriteResult::LdrLiteral(_) => 4,
            InstructionRewriteResult::AdrpAndLdrUnsignedOffset(_, _) => 8,
            InstructionRewriteResult::MovImmediateAndLdrLiteral(boxed) => boxed.len() * 4,
            InstructionRewriteResult::None => 0,
            InstructionRewriteResult::Tbz(_) => 4,
            InstructionRewriteResult::TbzAndBranch(_, _) => 8,
            InstructionRewriteResult::TbzAndAdrpAndBranch(_, _, _) => 12,
            InstructionRewriteResult::TbzAndAdrpAndAddAndBranch(_) => 16,
            InstructionRewriteResult::TbzAndBranchAbsolute(boxed) => boxed.len() * 4,
        }
    }
}

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
