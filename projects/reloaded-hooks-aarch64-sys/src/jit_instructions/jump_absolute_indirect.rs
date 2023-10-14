use reloaded_hooks_portable::api::jit::{
    compiler::JitError,
    operation_aliases::{JumpAbs, JumpAbsInd},
};
extern crate alloc;
use crate::all_registers::AllRegisters;
use alloc::vec::Vec;

use super::branch_absolute::encode_jump_absolute;

/// ADRP + ADD + BR
pub fn encode_jump_absolute_indirect(
    x: &JumpAbsInd<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    encode_jump_absolute(
        &JumpAbs {
            scratch_register: x.scratch_register,
            target_address: unsafe { *(x.pointer_address as *const usize) },
        },
        pc,
        buf,
    )
}
