use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;

use crate::all_registers::AllRegisters;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en#iclass_post_indexed
bitfield! {
    pub struct LdrImmediatePostIndexed(u32);
    impl Debug;
    u8;

    /// Constant field, set to 1.
    /// In future, used by size register.
    const_one, set_const_one: 31;

    /// Size field. 1 if 64-bit register, else 0.
    size, set_size: 30;

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
        source: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        if !(-256..=255).contains(&stack_offset) {
            return Err(return_stack_out_of_range(stack_offset));
        }

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediatePostIndexed(0);
        value.set_const_one(true);
        value.set_opcode(0b111000); // post-index variant
        value.set_opc(0b010);
        value.set_unk(0b01);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_size(is_64bit);
        value.set_rn_offset(stack_offset as i16);
        value.set_rt(source);
        Ok(value)
    }
}

#[inline(never)]
fn return_stack_out_of_range(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Stack Offset Exceeds Maximum Range. Offset {}",
        stack_offset
    ))
}
