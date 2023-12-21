extern crate alloc;

use super::Register;
use crate::common::{
    jit_conversions_common::map_register_x86_to_allregisters,
    rewriter::code_rewriter::relocate_code, util::get_stolen_instructions::get_stolen_instructions,
};
use alloc::vec::Vec;
use core::slice;
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};

pub struct CodeRewriterX86;

impl CodeRewriter<Register> for CodeRewriterX86 {
    unsafe fn rewrite_code_with_buffer(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<Register>,
        existing_buffer: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        let ins_slice = unsafe { slice::from_raw_parts(old_code, old_code_size) };
        let instructions =
            get_stolen_instructions(false, old_code_size, ins_slice, old_address).unwrap();
        relocate_code(
            false,
            &instructions.0,
            ins_slice,
            new_address,
            scratch_register.map(map_register_x86_to_allregisters),
            existing_buffer,
        )?;
        Ok(())
    }

    fn max_ins_size_increase() -> usize {
        4 // jmp imm8 to jmp dword [ptr]
    }
}
