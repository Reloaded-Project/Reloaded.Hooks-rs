use crate::instructions::errors::return_stack_out_of_range;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;

use crate::all_registers::AllRegisters;

use super::errors::return_divisible_by_value;

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--?lang=en
bitfield! {
    /// `LDR` represents the bitfields of the LDR immediate instruction
    /// in AArch64 architecture. The bitfields are described as follows:
    pub struct LdrImmediateUnsignedOffset(u32);
    impl Debug;
    u8;

    /// Size field. 1 if 64-bit register, else 0.
    size, set_size: 31, 30;

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
        let encoded_offset = if is_64bit {
            if (stack_offset & 0b111) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(0..=32760).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 3
        } else {
            if (stack_offset & 0b11) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(0..=16380).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 2
        };

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediateUnsignedOffset(0);
        value.set_opcode(0b111001);
        value.set_opc(0b01);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_rt(destination);
        value.set_size(if is_64bit { 11 } else { 10 });
        value.set_rn_offset(encoded_offset as i16);
        Ok(value)
    }

    pub fn new_mov_from_stack_vector(
        destination: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        // Check if divisible by 16.
        if (stack_offset & 0b1111) != 0 {
            return Err(return_divisible_by_value(stack_offset));
        }

        // Verify it's in range
        if !(0..=65520).contains(&stack_offset) {
            return Err(return_stack_out_of_range(stack_offset));
        }

        let encoded_offset = stack_offset >> 4;

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = LdrImmediateUnsignedOffset(0);
        value.set_opcode(0b111101);
        value.set_opc(0b11); // 11 for 128-bit
        value.set_size(00); // 128-bit

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_rt(destination);
        value.set_rn_offset(encoded_offset as i16);
        Ok(value)
    }
}
