extern crate alloc;

use super::errors::invalid_shift_amount;
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/MOVZ--Move-wide-with-zero-?lang=en
// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/MOVK--Move-wide-with-keep-?lang=en#sa_shift
bitfield! {
    pub struct MovImmediate(u32);
    impl Debug;
    u8;

    /// Set flag determines whether the operation is 32 or 64 bits.
    /// 0 for 32-bit and 1 for 64-bit.
    sf, set_sf: 31;

    /// Opcode for the MOV instruction variant.
    opcode, set_opcode: 30, 23;

    /// Shift to apply to the immediate value.
    /// 0 -> 0
    /// 1 -> LSL 16
    /// 2 -> LSL 32
    /// 3 -> LSL 48
    left_shift, set_left_shift: 22, 21;

    /// Immediate value to mov.
    u16, imm16, set_imm16: 20, 5;

    /// Register number for the destination.
    rd, set_rd: 4, 0;
}

impl MovImmediate {
    /// Creates a new 'MOV with keep' instruction, which injects a 16-bit immediate value
    /// into the destination register, shifted left by 0, 16, 32 or 48 bits, and leaving
    /// the rest of the register unchanged.
    pub fn new_movk(
        is_64bit: bool,
        destination: u8,
        value: u16,
        shift: u8,
    ) -> Result<Self, JitError<AllRegisters>> {
        #[cfg(debug_assertions)]
        if shift % 16 != 0 {
            return Err(invalid_shift_amount("[MOVK]", shift));
        }

        let mut result = MovImmediate(0);
        result.set_sf(is_64bit);
        result.set_opcode(0b11100101);
        result.set_imm16(value);
        result.set_left_shift(shift / 16);
        result.set_rd(destination);
        Ok(result)
    }

    /// Creates a new 'MOV with zero' instruction, which injects a 16-bit immediate value
    /// into the destination register, shifted left by 0, 16, 32 or 48 bits, and leaving
    /// the rest of the register as zero.
    pub fn new_movz(
        is_64bit: bool,
        destination: u8,
        value: u16,
        shift: u8,
    ) -> Result<Self, JitError<AllRegisters>> {
        if shift % 16 != 0 {
            return Err(invalid_shift_amount("[MOVZ]", shift));
        }

        let mut result = MovImmediate(0);
        result.set_sf(is_64bit);
        result.set_opcode(0b10100101);
        result.set_imm16(value);
        result.set_left_shift(shift / 16);
        result.set_rd(destination);
        Ok(result)
    }
}
