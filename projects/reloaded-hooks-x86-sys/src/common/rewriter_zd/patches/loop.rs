extern crate alloc;
use core::ptr::write_unaligned;

use crate::common::util::get_stolen_instructions::ZydisInstruction;
use crate::x64;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

const REL32_SIZE: usize = 7;

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x86")]
pub(crate) fn patch_loop_32<TRegister>(
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
    // DO IT
    *source_address += instruction.length as usize;
    *dest_address += REL32_SIZE;

    let old_len = buf.len();
    buf.reserve(REL32_SIZE);
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        write_unaligned(ptr as *mut u32, 0x0085_0F49_u32.to_le()); // dec ecx, jnz [rel32 placeholder]
        write_unaligned(ptr.add(3) as *mut i32, delta as i32); // rel32

        buf.set_len(old_len + REL32_SIZE);
    }
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_loop_64(
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
        Before: (E2 FA)
        - loop -3

        Relocated: (E2 02 EB 0C 48 B8 FD 0F 00 80 00 00 00 00 FF E0)
        - loop +2                   ---+
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
        write_unaligned(ptr as *mut u32, 0x0CEB_02E2_u32.to_le());
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
    #[case::loop_backward_i8("e2fb", 4096, 0, "490f85f60f0000")] // loop 0xffd -> dec ecx + jnz 0xffd
    #[case::loop_backward_abs_upper32("e2fb", 0x80001000, 0x80000000, "490f85f60f0000")] // loop 0x80000ffd -> dec ecx + jnz 0x80000ffd
    fn relocate_loop_32(
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
            patch_loop_32, // the function being tested
        );
    }

    #[rstest]
    #[case::loop_backward_i8("e2fb", 4096, 0, "490f85f60f0000")] // loop 0xffd -> dec ecx + jnz 0xffd
    #[case::loop_backward_abs_upper32("e2fb", 0x80001000, 0x80000000, "490f85f60f0000")] // loop 0x80000ffd -> dec ecx + jnz 0x80000ffd
    #[case::loop_backward_abs("e2fa", 0x80001000, 0, "e202eb0c48b8fc0f008000000000ffe0")] // loop -3 -> loop +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loop_backward_abs_upper64(
        "e2fa",
        0x8000000080001000,
        0x8000000000000000,
        "e202eb0c48b8fc0f008000000080ffe0"
    )] // loop -3 -> loop +2 + jmp 0x11 + movabs rax, 0x8000000080000ffd + jmp rax
    fn relocate_loop_64(
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
            patch_loop_64, // the function being tested
        );
    }
}
