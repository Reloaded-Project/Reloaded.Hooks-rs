use alloc::format;
use reloaded_hooks_portable::api::jit::compiler::JitError;

extern crate alloc;
use crate::all_registers::AllRegisters;

#[inline(never)]
pub fn return_stack_out_of_range(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::OperandOutOfRange(format!(
        "Stack Offset Exceeds Maximum Range. Offset {}",
        stack_offset
    ))
}

#[inline(never)]
pub fn return_divisible_by_value(stack_offset: i32) -> JitError<AllRegisters> {
    JitError::InvalidOffset(format!(
        "Offset must be divisible by the register size (4/8). {}",
        stack_offset
    ))
}
