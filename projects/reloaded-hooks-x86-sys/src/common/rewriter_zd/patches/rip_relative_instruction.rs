extern crate alloc;
use core::ptr::write_unaligned;

use crate::common::util::get_stolen_instructions::ZydisInstruction;
use crate::x64;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

const REL32_SIZE: usize = 8;

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_rip_relative_64<TRegister>(
    instruction: &ZydisInstruction,
    _instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    _scratch_register: Option<TRegister>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::test_relocate_instruction;
    use crate::x64;
    use crate::x86;
    use rstest::rstest;

    #[rstest]
    #[case::jrcxz_i32("e3fa", 0x8000000, 0, "85c90f85f4ffff07")] // jecxz 0x7fffffc -> test ecx, ecx + jne 0x7fffffc
    fn relocate_rip_rel_64(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_relocate_instruction(
            instructions,
            old_address,
            new_address,
            expected,
            Some(x86::Register::eax),
            true,
            patch_rip_relative_64, // the function being tested
        );
    }
}
