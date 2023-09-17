use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;

use crate::all_registers::AllRegisters;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en
bitfield! {
    /// `LDR` represents the bitfields of the LDR immediate instruction
    /// in AArch64 architecture. The bitfields are described as follows:
    pub struct LdrImmediateUnsignedOffset(u32);
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
    opc, set_opc: 23, 22;

    /// Source register to which the immediate offset is added.
    i16, rn_offset, set_rn_offset: 21, 10;

    /// Register number for the first operand (source), 31 for SP.
    rn, set_rn: 9, 5;

    /// Register number for the destination where the result will be stored.
    rt, set_rt: 4, 0;
}

impl LdrImmediateUnsignedOffset {
    pub fn new_mov_from_stack(
        is_64bit: bool,
        destination: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        // Check if divisible by 8 or 4.
        let mut encoded_offset = if is_64bit {
            if (stack_offset & 0b111) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(-32768..=32760).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 3
        } else {
            if (stack_offset & 0b11) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(-16384..=16380).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 2
        };

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediateUnsignedOffset(0);
        value.set_const_one(true);
        value.set_opcode(0b111001);
        value.set_opc(0b01);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_rt(destination);
        value.set_size(is_64bit);
        value.set_rn_offset(encoded_offset as i16);
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

#[inline(never)]
fn return_divisible_by_value(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::InvalidOffset(format!(
        "Offset must be divisible by the register size (4/8). {}",
        stack_offset
    ))
}
