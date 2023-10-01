use alloc::format;
use bitfield::bitfield;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;

use crate::all_registers::AllRegisters;

use super::errors::{return_divisible_by_value, return_stack_out_of_range};

// https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/STP--Store-Pair-of-Registers-?lang=en
bitfield! {
    pub struct StpImmediate(u32);
    impl Debug;
    u8;

    /// Size field. 1 if 64-bit register, else 0.
    size, set_size: 31, 30;

    /// The raw opcode used for this operation.
    opcode, set_opcode: 29, 22;

    /// The offset from rn, counted in register size increments.
    offset, set_rn_offset: 21, 15;

    /// The second destination register.
    rt2, set_rt2: 14, 10;

    /// Register number for the first operand (source), 31 for SP.
    rn, set_rn: 9, 5;

    /// Register number for the destination where the result will be stored.
    rt, set_rt: 4, 0;
}

impl StpImmediate {
    pub fn new_push_registers(
        is_64bit: bool,
        dst_1: u8,
        dst_2: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        // Check if divisible by 8 or 4, and fits in range.
        let encoded_offset = if is_64bit {
            if (stack_offset & 0b111) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(-512..=504).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 3
        } else {
            if (stack_offset & 0b11) != 0 {
                return Err(return_divisible_by_value(stack_offset));
            }

            if !(-256..=252).contains(&stack_offset) {
                return Err(return_stack_out_of_range(stack_offset));
            }

            stack_offset >> 2
        };

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = StpImmediate(0);
        value.set_opcode(0b10100110); // pre-index variant
        value.set_size(if is_64bit { 10 } else { 0 });

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_rn_offset(encoded_offset as u8);
        value.set_rt(dst_1);
        value.set_rt2(dst_2);

        Ok(value)
    }

    pub fn new_push_registers_vector(
        dst_1: u8,
        dst_2: u8,
        stack_offset: i32,
    ) -> Result<Self, JitError<AllRegisters>> {
        // Check if divisible by 16
        if (stack_offset & 0b1111) != 0 {
            return Err(return_divisible_by_value(stack_offset));
        }

        if !(-1024..=1008).contains(&stack_offset) {
            return Err(return_stack_out_of_range(stack_offset));
        }

        let encoded_offset = stack_offset >> 4;

        // Note: Compiler is smart enough to optimize this away as a constant
        // Which is why we moved the non-constant stuff to the bottom.
        let mut value = StpImmediate(0);
        value.set_opcode(0b10110110); // pre-index variant
        value.set_size(10);

        // Set Stack Pointer as Source Register
        value.set_rn(31);

        // Set parameters
        value.set_rn_offset(encoded_offset as u8);
        value.set_rt(dst_1);
        value.set_rt2(dst_2);

        Ok(value)
    }
}
