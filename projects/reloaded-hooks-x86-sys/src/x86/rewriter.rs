extern crate alloc;

use crate::{
    all_registers::AllRegisters,
    common::{
        rewriter::code_rewriter::relocate_code,
        util::get_stolen_instructions::get_stolen_instructions,
    },
};
use alloc::vec::Vec;
use core::slice;
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};

struct CodeRewriterX86;

impl CodeRewriter<AllRegisters> for CodeRewriterX86 {
    fn rewrite_code(
        old_address: *const u8,
        old_address_size: usize,
        new_address: *const u8,
        scratch_register: Option<AllRegisters>,
    ) -> Result<Vec<u8>, CodeRewriterError> {
        let ins_slice = unsafe { slice::from_raw_parts(old_address, old_address_size) };
        let instructions =
            get_stolen_instructions(false, old_address_size, ins_slice, old_address as usize)
                .unwrap();
        let result = relocate_code(
            false,
            &instructions.0,
            new_address as usize,
            scratch_register,
        )?;
        Ok(result)
    }
}
