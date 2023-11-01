extern crate alloc;

use crate::instructions::mov_immediate::MovImmediate;
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
    MovImmediate1(u32), // in instruction count order
    MovImmediate2(u32, u32),
    MovImmediate3(u32, u32, u32),
    MovImmediate4(Box<[u32; 4]>),
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
