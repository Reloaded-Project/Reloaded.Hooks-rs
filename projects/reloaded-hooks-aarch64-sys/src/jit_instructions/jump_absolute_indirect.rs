extern crate alloc;

use super::branch_absolute::encode_jump_absolute;
use crate::all_registers::AllRegisters;
use alloc::string::ToString;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError,
    operation_aliases::{JumpAbs, JumpAbsInd},
};

/// ADRP + ADD + BR
pub fn encode_jump_absolute_indirect(
    x: &JumpAbsInd<AllRegisters>,
    pc: &mut usize,
    buf: &mut Vec<i32>,
) -> Result<(), JitError<AllRegisters>> {
    if x.scratch_register.is_none() {
        return Err(JitError::NoScratchRegister(
            "Needed for JumpAbsInd".to_string(),
        ));
    }

    encode_jump_absolute(
        &JumpAbs {
            scratch_register: x.scratch_register.unwrap(),
            target_address: unsafe { *(x.pointer_address as *const usize) },
        },
        pc,
        buf,
    )
}
