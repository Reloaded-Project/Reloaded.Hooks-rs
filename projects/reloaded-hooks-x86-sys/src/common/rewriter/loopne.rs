extern crate alloc;
use core::ptr::write_unaligned;

use crate::common::util::get_stolen_instructions::ZydisInstruction;
use crate::x64;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

const REL32_SIZE: usize = 9;

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x86")]
pub(crate) fn patch_loopne_32<TRegister>(
    instruction: &ZydisInstruction,
    _instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    _scratch_register: Option<TRegister>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    let next_pc = ((*dest_address).wrapping_add(REL32_SIZE)) as u64;
    let delta = target.wrapping_sub(next_pc) as i64;

    encode_rel32(source_address, instruction, dest_address, buf, delta);
    Ok(())
}

fn encode_rel32(
    source_address: &mut usize,
    instruction: &zydis::Instruction<zydis::OperandArrayVec<5>>,
    dest_address: &mut usize,
    buf: &mut Vec<u8>,
    delta: i64,
) {
    /*
        e.g.
        Before: (E1 FA)
        - loopne -3

        Relocated: (E2 02 EB 0C 48 B8 FD 0F 00 80 00 00 00 00 FF E0)
        - loopne +2          ---+
        - jmp +5               |  --+
        - jmp <orig target> <--+    |
                                <--+
    */

    const INS_SIZE_REL: usize = 9;
    *source_address += instruction.length as usize;
    *dest_address += INS_SIZE_REL;

    let old_len = buf.len();
    buf.reserve(INS_SIZE_REL);

    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // loopne 0x4 + jmp +5 (fixed)
        write_unaligned(ptr as *mut u32, 0x05EB_02E0_u32.to_le());
        ptr.add(4).write(0xE9);
        write_unaligned(ptr.add(5) as *mut u32, (delta as u32).to_le());
        buf.set_len(old_len + INS_SIZE_REL);
    }
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_loopne_64(
    instruction: &ZydisInstruction,
    _instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: Option<x64::Register>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    use crate::instructions::jump_absolute::encode_absolute_jump_x64;
    use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;

    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    let next_pc = ((*dest_address).wrapping_add(REL32_SIZE)) as u64;
    let delta = target.wrapping_sub(next_pc) as i64;

    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        encode_rel32(source_address, instruction, dest_address, buf, delta);
        return Ok(());
    }

    // Cold path, we need to emulate the instruction if beyond 2GiB
    /*
        e.g.
        Before: (E0 FA)
        - loopne -3

        Relocated: (E0 02 EB 05 E9 F3 FF FF 07)
        - loopne +2                   ---+
        - jmp 0x11                     |  --+
        - movabs rax, 0x80000ffd    <--+    |
        - jmp rax                           |
                                         <--+
    */
    const INS_SIZE_ABS: usize = 16;
    let mut pc_jmp_abs = *dest_address + 4; // abs address at jmp instruction

    *source_address += instruction.length as usize;
    *dest_address += INS_SIZE_ABS;

    let scratch_reg =
        scratch_register.ok_or_else(|| CodeRewriterError::NoScratchRegister(Default::default()))?;

    let old_len = buf.len();
    buf.reserve(INS_SIZE_ABS);

    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // loop 0x4 + jmp 0x10 (fixed)
        write_unaligned(ptr as *mut u32, 0x0CEB_02E0_u32.to_le());
        buf.set_len(old_len + 4);

        // Absolute jump to original target.
        encode_absolute_jump_x64(
            &JumpAbsoluteOperation {
                scratch_register: scratch_reg,
                target_address: target as usize,
            },
            &mut pc_jmp_abs,
            buf,
        )
        .unwrap();
    }

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
    #[case::loopne_backward("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne 0x7fffffc -> loopne 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loopne_backward_i32("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne -3 -> loopne 5 + jmp 0xa + jmp 0x7fffffd
    fn relocate_loopne_32(
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
            false,
            patch_loopne_32, // the function being tested
        );
    }

    #[rstest]
    #[case::loopne_backward("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne 0x7fffffc -> loopne 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loopne_backward_i32("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne -3 -> loopne 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loopne_backward_abs_upper64(
        "e0fa",
        0x8000000080001000,
        0x8000000000000000,
        "e002eb0c48b8fc0f008000000080ffe0"
    )] // loopne 0x8000000080000ffc -> loopne 0x8000000080001004 + jmp 0x8000000080001010 + movabs rax, 0x8000000080000ffc + jmp rax
    fn relocate_loopne_64(
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
            Some(x64::Register::rax),
            true,
            patch_loopne_64, // the function being tested
        );
    }
}
