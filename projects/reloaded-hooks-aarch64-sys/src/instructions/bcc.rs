use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;
use super::errors::{return_divisible_by_instruction, return_divisible_by_page};
use crate::all_registers::AllRegisters;

bitfield! {
    /// `Bcc` represents the bitfields of the B.cond (conditional branch) instruction
    /// in AArch64 architecture.
    pub struct Bcc(u32);
    impl Debug;
    u8;

    /// Opcode
    opcode, set_opcode: 31, 24;

    /// Imm19 field for the branch offset.
    pub i32, imm19, set_imm19: 23, 5;

    /// Always set to 0
    unk, set_unk: 4, 4;

    /// Condition flags for the branch.
    pub condition, set_condition: 3, 0;
}

impl Bcc {
    /// Assembles a Bcc instruction with the specified parameters.
    pub fn assemble_bcc(condition: u8, offset: i32) -> Result<Self, JitError<AllRegisters>> {
        if !(-1048576..=1048575).contains(&offset) {
            return Err(value_out_of_range(offset));
        }

        if (offset & 0b11) != 0 {
            return Err(return_divisible_by_instruction(offset));
        }

        let mut instruction = Bcc(0);
        instruction.set_opcode(0b01010100);
        instruction.set_imm19(offset / 4);
        instruction.set_condition(condition);

        Ok(instruction)
    }

    /// Returns the calculated target offset.
    pub fn offset(&self) -> isize {
        self.imm19() as isize * 4
    }
}

#[inline(never)]
fn value_out_of_range(value: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Bcc Value Exceeds Maximum Range (-+ 1MB). Value {}",
        value
    ))
}
