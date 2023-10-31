extern crate alloc;

use super::errors::{exceeds_maximum_range, must_be_divisible_by};
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

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
    fn assemble_common(
        instruction_name: &str,
        offset: i32,
        opcode: u8,
    ) -> Result<Self, JitError<AllRegisters>> {
        #[cfg(debug_assertions)]
        if !(-0x8000000..=0x7FFFFFF).contains(&offset) {
            return Err(exceeds_maximum_range(
                instruction_name,
                "-+128MiB",
                offset as isize,
            ));
        }

        #[cfg(debug_assertions)]
        if (offset & 0b11) != 0 {
            return Err(must_be_divisible_by(instruction_name, offset as isize, 4));
        }

        let mut instruction = B(0);
        instruction.set_opcode(opcode);
        instruction.set_imm26(offset / 4);

        Ok(instruction)
    }

    /// Assembles a B or BL instruction with the specified offset.
    pub fn assemble(offset: i32, link: bool) -> Result<Self, JitError<AllRegisters>> {
        if link {
            Self::assemble_bl(offset)
        } else {
            Self::assemble_b(offset)
        }
    }

    /// Assembles a B instruction with the specified offset.
    pub fn assemble_b(offset: i32) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[B]", offset, 0b000101)
    }

    /// Assembles a BL instruction with the specified offset.
    pub fn assemble_bl(offset: i32) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[BL]", offset, 0b100101)
    }

    pub fn offset(&self) -> i32 {
        self.imm26() * 4
    }
}
