extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::buffers::buffer_abstractions::Buffer;

use crate::instructions::mov_immediate::MovImmediate;

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
pub(crate) enum InstructionRewriteResult {
    Adr(u32),
    AdrpAndAdd(u32, u32),
    MovImmediate1(u32), // in instruction count order
    MovImmediate2(u32, u32),
    MovImmediate3(u32, u32, u32),
    MovImmediate4(u32, u32, u32, u32),
}

impl InstructionRewriteResult {
    /// Appends the instructions represented by `self` into the provided buffer.
    ///
    /// # Parameters
    ///
    /// * `buf`: The buffer to which the instruction(s) will be appended.
    pub(crate) fn append_to_buffer(&self, buf: &mut Vec<i32>) {
        match self {
            InstructionRewriteResult::Adr(inst) => {
                buf.push(*inst as i32);
            }
            InstructionRewriteResult::AdrpAndAdd(inst1, inst2) => {
                buf.push(*inst1 as i32);
                buf.push(*inst2 as i32);
            }
            InstructionRewriteResult::MovImmediate1(inst) => {
                buf.push(*inst as i32);
            }
            InstructionRewriteResult::MovImmediate2(inst1, inst2) => {
                buf.push(*inst1 as i32);
                buf.push(*inst2 as i32);
            }
            InstructionRewriteResult::MovImmediate3(inst1, inst2, inst3) => {
                buf.push(*inst1 as i32);
                buf.push(*inst2 as i32);
                buf.push(*inst3 as i32);
            }
            InstructionRewriteResult::MovImmediate4(inst1, inst2, inst3, inst4) => {
                buf.push(*inst1 as i32);
                buf.push(*inst2 as i32);
                buf.push(*inst3 as i32);
                buf.push(*inst4 as i32);
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
        49..=64 => InstructionRewriteResult::MovImmediate4(
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
        ),
        _ => unreachable!(), // This case should never be reached unless platform is >64 bits
    }
}
