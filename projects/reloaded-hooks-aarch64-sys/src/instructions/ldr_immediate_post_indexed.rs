extern crate alloc;

use super::errors::return_stack_out_of_range;
use crate::all_registers::AllRegisters;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
bitfield! {
    pub struct LdrImmediatePostIndexed(u32);
    impl Debug;
    u8;

    /// Size field. 1 if 64-bit register, else 0.
    size, set_size: 31, 30;

    /// The raw opcode used for this operation.
    opcode, set_opcode: 29, 24;

    /// The operation used, dictates if this is a load or store.
    opc, set_opc: 23, 21;

    /// Source register to which the immediate offset is added.
    i16, rn_offset, set_rn_offset: 20, 12;

    /// 2-bit field, set to 0b01.
    unk, set_unk: 11, 10;

    /// Register number for the first operand (source), 31 for SP.
    rn, set_rn: 9, 5;

    /// Register number for the destination where the result will be stored.
    rt, set_rt: 4, 0;
}

impl LdrImmediatePostIndexed {
    pub fn new_pop_register(
        is_64bit: bool,
        target: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        if !(-256..=255).contains(&stack_offset) {
            return Err(return_stack_out_of_range(
                "LDR Immediate Post Indexed",
                "-256..255",
                stack_offset as isize,
            ));
        }

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediatePostIndexed(0);
        value.set_opcode(0b111000); // post-index variant
        value.set_opc(0b010);
        value.set_unk(0b01);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_size(if is_64bit { 11 } else { 10 });
        value.set_rn_offset(stack_offset as i16);
        value.set_rt(target);
        Ok(value)
    }

    pub fn new_pop_vector(target: u8, stack_offset: i32) -> Result<Self, JitError<AllRegisters>> {
        if !(-256..=255).contains(&stack_offset) {
            return Err(return_stack_out_of_range(
                "LDR Immediate Post Indexed",
                "-256..255",
                stack_offset as isize,
            ));
        }

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediatePostIndexed(0);
        value.set_opcode(0b111100); // post-index variant
        value.set_opc(0b110);
        value.set_unk(0b01);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_size(00);
        value.set_rn_offset(stack_offset as i16);
        value.set_rt(target);
        Ok(value)
    }
}
