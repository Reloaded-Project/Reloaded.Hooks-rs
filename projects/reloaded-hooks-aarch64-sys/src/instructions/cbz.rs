extern crate alloc;

use super::errors::{exceeds_maximum_range, must_be_divisible_by};
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/CBZ--Compare-and-Branch-on-Zero-?lang=en
bitfield! {
    pub struct Cbz(u32);
    impl Debug;
    u8;

    /// True if 64-bit else false
    pub is_64bit, set_is_64bit: 31;

    /// Opcode
    opcode, set_opcode: 30, 25;

    /// Imm19 field for the branch offset.
    pub non_zero, set_non_zero: 24;

    /// Imm19 field for the branch offset.
    pub i32, imm19, set_imm19: 23, 5;

    /// The register to compare against zero.
    pub rt, set_rt: 4, 0;
}

impl Cbz {
    fn assemble_common(
        instruction_name: &str,
        offset: i32,
        reg_num: u8,
        is_64bit: bool,
        opcode: u8,
        non_zero: bool,
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

        let mut instruction = Cbz(0);
        instruction.set_opcode(opcode);
        instruction.set_non_zero(non_zero);
        instruction.set_is_64bit(is_64bit);
        instruction.set_rt(reg_num);
        instruction.set_imm19(offset / 4);

        Ok(instruction)
    }

    pub fn assemble(
        offset: i32,
        reg_num: u8,
        is_64bit: bool,
        non_zero: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        if non_zero {
            Self::assemble_cbnz(offset, reg_num, is_64bit)
        } else {
            Self::assemble_cbz(offset, reg_num, is_64bit)
        }
    }

    pub fn assemble_cbz(
        offset: i32,
        reg_num: u8,
        is_64bit: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[CBZ]", offset, reg_num, is_64bit, 0b011010, false)
    }

    pub fn assemble_cbnz(
        offset: i32,
        reg_num: u8,
        is_64bit: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[CBNZ]", offset, reg_num, is_64bit, 0b011010, true)
    }

    pub fn offset(&self) -> i32 {
        self.imm19() * 4
    }
}
