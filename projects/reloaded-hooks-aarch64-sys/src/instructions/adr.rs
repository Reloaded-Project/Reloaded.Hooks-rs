use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;
use super::errors::return_divisible_by_page;
use crate::all_registers::AllRegisters;

bitfield! {
    pub struct Adr(u32);
    impl Debug;
    u8;

    /// If 1, this is a 4K page address, else 0.
    pub is_pageaddress, set_is_pageaddress: 31;

    /// Low bit of the immediate.
    pub immlo, set_immlo: 30, 29;

    /// Static opcode for this instruction.
    pub opcode, set_opcode: 28, 24;

    /// Immediate value to add.
    pub i32, immhi, set_immhi: 23, 5;

    /// Register number for the destination.
    pub rd, set_rd: 4, 0;
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

    /// Extracts the address calculated by the ADR instruction relative to the provided base address.
    ///
    /// # Parameters
    ///
    /// * `base_address`: The address where this ADR instruction is located.
    ///
    /// # Returns
    ///
    /// The calculated absolute address which the ADR instruction will load into the register.
    pub fn extract_address(&self, base_address: usize) -> usize {
        let immhi = self.immhi() as i32;
        let immlo = self.immlo() as i32;

        // Combine the immhi and immlo to get the full immediate value.
        let offset = (immhi << 2) | (immlo & 0b11);
        if self.is_pageaddress() {
            (base_address as i64 + (offset as i64 * 4096)) as usize
        } else {
            (base_address as i64 + offset as i64) as usize
        }
    }

    /// Set the raw offset fields (`immhi` and `immlo`) of the ADR/ADRP instruction.
    ///
    /// This function directly sets the `immhi` and `immlo` fields based on the provided raw offset,
    /// without performing any additional checks. Ensure to use this function with care.
    ///
    /// # Parameters
    ///
    /// * `offset`: The raw offset value to be set.
    pub fn set_raw_offset(&mut self, offset: i32) {
        self.set_immhi((offset >> 2) & 0x7FFFF); // Set bits [23:5].
        self.set_immlo((offset & 0x3) as u8); // Set bits [1:0].
    }

    /// Determines if the instruction is an ADRP.
    ///
    /// # Returns
    ///
    /// * `true` if the instruction is an ADRP.
    /// * `false` otherwise.
    pub fn is_adrp(&self) -> bool {
        self.is_pageaddress()
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
