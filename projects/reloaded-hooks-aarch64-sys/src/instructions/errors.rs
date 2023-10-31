extern crate alloc;

use crate::all_registers::AllRegisters;
use alloc::{borrow::ToOwned, string::ToString};
use reloaded_hooks_portable::api::jit::compiler::JitError;

// Note: We don't use format! in this library, to save space in the final binary.
// Parameters should also be normalized to isize, unless usize is required.
#[inline(never)]
pub fn invalid_register_combination(
    reg1: AllRegisters,
    reg2: AllRegisters,
) -> JitError<AllRegisters> {
    JitError::InvalidRegisterCombination(reg1, reg2)
}

/// Generates an error for when a given stack offset is out of range
///
/// # Parameters
/// * `instruction`: The instruction where this error was generated.
/// * `max_range`: The maximum allowed range for the operand of the instruction. e.g. '+-4GiB'.
/// * `value`: The actual value of the operand.
#[inline(never)]
pub fn return_stack_out_of_range(
    instruction: &str,
    max_range: &str,
    stack_offset: isize,
) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(
        instruction.to_owned()
            + " Stack Offset Exceeds Maximum Range. Max Range: "
            + max_range
            + " Operand: "
            + &stack_offset.to_string(),
    )
}

/// Generates an error for when a value needs to be divisible by 4, but isn't.
///
/// # Parameters
/// * `instruction`: Name of the instruction that threw the error..
/// * `offset`: The value of the offset.
/// * `div_by`: What the value should be divisible by.
#[inline(never)]
pub fn must_be_divisible_by(
    instruction: &str,
    offset: isize,
    div_by: isize,
) -> JitError<AllRegisters> {
    JitError::InvalidOffset(
        instruction.to_owned()
            + " Offset must be divisible by "
            + &div_by.to_string()
            + " . Offset: "
            + &offset.to_string(),
    )
}

/// Generates an error for when a value needs to be divisible by 4, but isn't.
///
/// # Parameters
/// * `instruction`: Name of the instruction that threw the error..
/// * `shift`: The shift amount.
#[inline(never)]
pub fn invalid_shift_amount(instruction: &str, shift: u8) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(
        instruction.to_owned()
            + " Invalid shift amount: "
            + &shift.to_string()
            + ". Amount should be 0, 16, 32 or 48.",
    )
}

/// Generates an error for when a given operand is out oof range
///
/// # Parameters
/// * `instruction`: The instruction where this error was generated.
/// * `max_range`: The maximum allowed range for the operand of the instruction. e.g. '+-4GiB'.
/// * `value`: The actual value of the operand.
#[inline(never)]
pub fn exceeds_maximum_range(
    instruction: &str,
    max_range: &str,
    value: isize,
) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(
        instruction.to_owned()
            + " Operand Exceeds Maximum Range. Max Range: "
            + max_range
            + " Operand: "
            + &value.to_string(),
    )
}
