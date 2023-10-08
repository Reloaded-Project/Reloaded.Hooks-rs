use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;

use crate::all_registers::AllRegisters;

use super::errors::return_divisible_by_page;

bitfield! {
    pub struct Adr(u32);
    impl Debug;
    u8;

    /// If 1, this is a 4K page address, else 0.
    is_pageaddress, set_is_pageaddress: 31;

    /// Low bit of the immediate.
    immlo, set_immlo: 30, 29;

    /// Static opcode for this instruction.
    opcode, set_opcode: 28, 24;

    /// Immediate value to add.
    i32, immhi, set_immhi: 23, 5;

    /// Register number for the destination.
    rd, set_rd: 4, 0;
}

impl Adr {
    /// Create a new ADR instruction with the specified parameters.
    pub fn new_adr(destination: u8, offset: i32) -> Result<Self, JitError<AllRegisters>> {
        if !(-1048576..=1048575).contains(&offset) {
            return Err(value_out_of_range(offset));
        }

        let mut value = Adr(0);
        value.set_is_pageaddress(false);
        value.set_opcode(0b10000);
        value.set_rd(destination);
        value.set_immhi(offset >> 2);
        value.set_immlo(offset as u8);

        Ok(value)
    }

    /// Create a new ADRP instruction with the specified parameters.
    pub fn new_adrp(destination: u8, offset: i64) -> Result<Self, JitError<AllRegisters>> {
        if !(-4294967296..=4294967295).contains(&offset) {
            return Err(value_out_of_adrp_range(offset));
        }

        if (offset & 0xFFF) != 0 {
            return Err(return_divisible_by_page(offset));
        }

        let mut value = Adr(0);
        value.set_is_pageaddress(true);
        value.set_opcode(0b10000);
        value.set_rd(destination);
        let final_offset = offset / 4096;
        value.set_immhi((final_offset >> 2) as i32);
        value.set_immlo(final_offset as u8);

        Ok(value)
    }
}

#[inline(never)]
fn value_out_of_range(value: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Adr Value Exceeds Maximum Range (-+ 1MB). Value {}",
        value
    ))
}

#[inline(never)]
fn value_out_of_adrp_range(value: i64) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Adrp Value Exceeds Maximum Range (-+ 4GB). Value {}",
        value
    ))
}
