extern crate alloc;

use super::errors::return_pc_out_of_range;
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDRSW--literal---Load-Register-Signed-Word--literal--
// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--literal---Load-Register--literal--
// Both are the same instruction, just different size operand.
bitfield! {
    pub struct LdrLiteral(u32);
    impl Debug;
    u8;

    /// Size field.
    /// 00 = 32-bit
    /// 01 = 64-bit
    /// 10 = 32-bit (signed / sign extended)
    /// 11 = prefetch
    pub mode, set_mode: 31, 30;

    /// The raw opcode used for this operation.
    pub opcode, set_opcode: 29, 24;

    /// True if this is a SIMD instruction/opcode.
    pub is_simd, set_is_simd: 26;

    /// The operation used, dictates if this is a load or store.
    pub i32, imm19, set_imm19: 23, 5;

    /// Register number for the destination where the result will be stored.
    pub rt, set_rt: 4, 0;
}

impl LdrLiteral {
    /// Creates a new LDR literal instruction.
    ///
    /// # Parameters
    /// - `mode`:
    ///     - 00 = 32-bit
    ///     - 01 = 64-bit
    ///     - 10 = 32-bit (signed / sign extended)
    ///     - 11 = prefetch
    /// - `is_simd`: True if this is a SIMD / FP instruction.
    /// - `target`: The register to load the value into.
    /// - `pc_offset`: The offset from the current program counter.
    pub fn new_load_literal(
        mode: u8,
        target: u8,
        is_simd: bool,
        pc_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        #[cfg(debug_assertions)]
        if !(-0x100000..=0xFFFFF).contains(&pc_offset) {
            return Err(return_pc_out_of_range(
                "LDR Literal",
                "-0x100000..0xFFFFF",
                pc_offset as isize,
            ));
        }

        let mut value = LdrLiteral(0);
        value.set_opcode(0b011000);
        value.set_mode(mode);
        value.set_imm19(pc_offset / 4);
        value.set_is_simd(is_simd);
        value.set_rt(target);
        Ok(value)
    }

    pub fn offset(&self) -> i32 {
        self.imm19() * 4
    }
}
