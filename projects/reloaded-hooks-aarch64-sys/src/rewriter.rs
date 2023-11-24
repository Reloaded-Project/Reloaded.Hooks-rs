extern crate alloc;

use crate::all_registers::AllRegisters;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};

struct CodeRewriterX64;

impl CodeRewriter<AllRegisters> for CodeRewriterX64 {
    fn rewrite_code(
        old_address: *const u8,
        old_address_size: usize,
        new_address: *const u8,
        scratch_register: Option<AllRegisters>,
    ) -> Result<Vec<u8>, CodeRewriterError> {
        crate::code_rewriter::aarch64_rewriter::rewrite_code_aarch64(
            old_address,
            old_address_size,
            new_address,
            scratch_register.map(|reg| reg.register_number() as u8),
        )
    }

    fn max_ins_size_increase() -> usize {
        20 // b rel to MOVZ + MOVK + LDR + BR.
    }
}
