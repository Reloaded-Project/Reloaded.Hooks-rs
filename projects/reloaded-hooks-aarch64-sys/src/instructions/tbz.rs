extern crate alloc;

use super::errors::{exceeds_maximum_range, must_be_divisible_by};
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/TBNZ--Test-bit-and-Branch-if-Nonzero-
bitfield! {
    pub struct Tbz(u32);
    impl Debug;
    u8;

    /// True if 64-bit else false
    pub is_64bit, set_is_64bit: 31;

    /// Opcode 0b011011
    opcode, set_opcode: 30, 25;

    /// Non-zero flag
    pub non_zero, set_non_zero: 24;

    /// The bit number to be tested.
    pub u8, bit_no, set_bit_no: 23, 19;

    /// Branch offset
    pub i32, imm14, set_imm14: 18, 5;

    /// The register to be tested
    pub reg, set_reg: 4, 0;
}

impl Tbz {
    fn assemble_common(
        instruction_name: &str,
        offset: i32,
        reg_num: u8,
        bit_no: u8,
        is_64bit: bool,
        non_zero: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        #[cfg(debug_assertions)]
        if !(-0x8000..=0x7FFF).contains(&offset) {
            return Err(exceeds_maximum_range(
                instruction_name,
                "-+32KiB",
                offset as isize,
            ));
        }

        #[cfg(debug_assertions)]
        if (offset & 0b11) != 0 {
            return Err(must_be_divisible_by(instruction_name, offset as isize, 4));
        }

        let mut instruction = Tbz(0);
        instruction.set_opcode(0b011011);
        instruction.set_is_64bit(is_64bit);
        instruction.set_non_zero(non_zero);
        instruction.set_reg(reg_num);
        instruction.set_bit_no(bit_no);
        instruction.set_imm14(offset / 4);

        Ok(instruction)
    }

    pub fn assemble(
        offset: i32,
        reg_num: u8,
        bit_no: u8,
        is_64bit: bool,
        non_zero: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        if non_zero {
            Self::assemble_tbnz(offset, reg_num, bit_no, is_64bit)
        } else {
            Self::assemble_tbz(offset, reg_num, bit_no, is_64bit)
        }
    }

    pub fn assemble_tbz(
        offset: i32,
        reg_num: u8,
        bit_no: u8,
        is_64bit: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[TBZ]", offset, reg_num, bit_no, is_64bit, false)
    }

    pub fn assemble_tbnz(
        offset: i32,
        reg_num: u8,
        bit_no: u8,
        is_64bit: bool,
    ) -> Result<Self, JitError<AllRegisters>> {
        Self::assemble_common("[TBNZ]", offset, reg_num, bit_no, is_64bit, true)
    }

    pub fn offset(&self) -> i32 {
        self.imm14() * 4
    }
}
