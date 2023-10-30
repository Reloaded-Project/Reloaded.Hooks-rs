extern crate alloc;

use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

use crate::all_registers::AllRegisters;

use super::errors::return_divisible_by_instruction;

bitfield! {
    /// `B` represents the bitfields of the B (unconditional branch) instruction
    /// in AArch64 architecture.
    pub struct B(u32);
    impl Debug;
    u8;

    /// Opcode
    opcode, set_opcode: 31, 26;

    /// Imm26 field for the branch offset.
    pub i32, imm26, set_imm26: 25, 0;
}

impl B {
    /// Assembles a B instruction with the specified offset.
    pub fn assemble_b(offset: i32) -> Result<Self, JitError<AllRegisters>> {
        if !(-0x8000000..=0x7FFFFFF).contains(&offset) {
            return Err(value_out_of_range(offset));
        }

        if (offset & 0b11) != 0 {
            return Err(return_divisible_by_instruction(offset));
        }

        let mut instruction = B(0);
        instruction.set_opcode(0b000101);
        instruction.set_imm26(offset / 4);

        Ok(instruction)
    }
}

#[inline(never)]
fn value_out_of_range(value: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "B Value Exceeds Maximum Range (-+ 128MB). Value {}",
        value
    ))
}
