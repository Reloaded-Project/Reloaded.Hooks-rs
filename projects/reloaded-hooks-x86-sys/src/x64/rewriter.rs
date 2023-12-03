extern crate alloc;

use super::Register;
use crate::{
    all_registers::AllRegisters,
    common::{
        jit_conversions_common::map_register_x64_to_allregisters,
        rewriter::code_rewriter::relocate_code,
        util::get_stolen_instructions::get_stolen_instructions,
    },
};
use alloc::vec::Vec;
use core::slice;
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};

pub struct CodeRewriterX64;

impl CodeRewriter<Register> for CodeRewriterX64 {
    unsafe fn rewrite_code(
        old_code: *const u8,
        old_address_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<Register>,
    ) -> Result<Vec<u8>, CodeRewriterError> {
        let ins_slice = unsafe { slice::from_raw_parts(old_code, old_address_size) };
        let instructions =
            get_stolen_instructions(true, old_address_size, ins_slice, old_address).unwrap();
        let result = relocate_code(
            true,
            &instructions.0,
            new_address,
            scratch_register.map(map_register_x64_to_allregisters),
        )?;
        Ok(result)
    }

    fn max_ins_size_increase() -> usize {
        14 // see: patches::patch_jcx
    }
}
