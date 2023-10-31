extern crate alloc;

use super::errors::exceeds_maximum_range;
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

bitfield! {
    /// `SubImmediate` represents the bitfields of the SUB (immediate) instruction
    /// in AArch64 architecture.
    pub struct SubImmediate(u32);
    impl Debug;
    u8;

    /// Set flag determines whether the operation is 32 or 64 bits.
    /// 0 for 32-bit and 1 for 64-bit.
    sf, set_sf: 31;

    /// Opcode for the SUB instruction, should be `0b10100010`.
    opcode, set_opcode: 30, 23;

    /// Shift to apply to the immediate value.
    /// 0 -> 0
    /// 1 -> LSL 12 (i.e. multiply by 4096)
    shift, set_shift: 22;

    /// Immediate value to subtract.
    i16, imm12, set_imm12: 21, 10;

    /// Register number for the source.
    rn, set_rn: 9, 5;

    /// Register number for the destination.
    rd, set_rd: 4, 0;
}

impl SubImmediate {
    /// Create a new SUB instruction with the specified parameters.
    pub fn new(
        is_64bit: bool,
        destination: u8,
        source: u8,
        immediate: u16,
    ) -> Result<Self, JitError<AllRegisters>> {
        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = SubImmediate(0);
        value.set_sf(is_64bit);
        value.set_opcode(0b10100010);
        value.set_imm12(immediate as i16);
        value.set_rn(source);
        value.set_rd(destination);

        if immediate > 4095 {
            return Err(exceeds_maximum_range(
                "[Sub Immediate]",
                "0..4095",
                immediate as isize,
            ));
        }

        Ok(value)
    }

    /// Create a new SUB instruction that makes additional space on the stack.
    pub fn new_stackalloc(is_64bit: bool, immediate: u16) -> Result<Self, JitError<AllRegisters>> {
        Self::new(is_64bit, 31, 31, immediate)
    }
}
