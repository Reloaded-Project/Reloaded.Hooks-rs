extern crate alloc;

use crate::all_registers::AllRegisters;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};

pub struct CodeRewriterAarch64;

impl CodeRewriter<AllRegisters> for CodeRewriterAarch64 {
    unsafe fn rewrite_code(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<AllRegisters>,
    ) -> Result<Vec<u8>, CodeRewriterError> {
        crate::code_rewriter::aarch64_rewriter::rewrite_code_aarch64(
            old_code,
            old_code_size,
            old_address,
            new_address,
            scratch_register.map(|reg| reg.register_number() as u8),
        )
    }

    fn max_ins_size_increase() -> usize {
        20 // b rel to MOVZ + MOVK + LDR + BR.
    }
}
