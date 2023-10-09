use alloc::format;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;
use crate::all_registers::AllRegisters;

#[inline(never)]
pub fn invalid_register_combination(
    reg1: AllRegisters,
    reg2: AllRegisters,
) -> JitError<AllRegisters> {
    JitError::InvalidRegisterCombination(reg1, reg2)
}

#[inline(never)]
pub fn return_stack_out_of_range(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Stack Offset Exceeds Maximum Range. Offset {}",
        stack_offset
    ))
}

#[inline(never)]
pub fn return_divisible_by_page(stack_offset: i64) -> JitError<AllRegisters> {
    JitError::InvalidOffset(format!(
        "Offset must be divisible by page size (4K). {}",
        stack_offset
    ))
}

#[inline(never)]
pub fn return_divisible_by_register(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::InvalidOffset(format!(
        "Offset must be divisible by the register size (4/8). {}",
        stack_offset
    ))
}

#[inline(never)]
pub fn invalid_shift_amount(shift: u8) -> JitError<AllRegisters> {
    JitError::InvalidOffset(format!(
        "Invalid shift amount {}. Amount should be 0, 16, 32 or 48.",
        shift
    ))
}
