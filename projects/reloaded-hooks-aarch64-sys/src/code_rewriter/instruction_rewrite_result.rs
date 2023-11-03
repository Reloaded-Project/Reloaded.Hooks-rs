extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

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
    /// No change was made to this instruction
    Copy(u32),
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
            InstructionRewriteResult::Copy(inst) => {
                buf.push(*inst);
            }
        }
    }

    /// Returns the size in bytes for the rewrite result.
    pub(crate) fn size_bytes(&self) -> usize {
        match self {
            InstructionRewriteResult::Copy(_) => 4,
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
