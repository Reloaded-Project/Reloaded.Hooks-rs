extern crate alloc;
use core::ptr::write_unaligned;

use crate::common::util::get_stolen_instructions::ZydisInstruction;
use crate::x64;
use alloc::vec::Vec;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

const INS_SIZE_REL: usize = 6;

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x86")]
pub(crate) fn patch_conditional_branch_32<TRegister>(
    instruction: &ZydisInstruction,
    instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    _scratch_register: Option<TRegister>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    if instruction_bytes.len() == 6 {
        patch_conditional_branch_common_32(
            instruction,
            instruction_bytes[1], // lift opcode out from original instruction
            dest_address,
            source_address,
            buf,
        )
    } else if instruction_bytes.len() == 2 {
        patch_conditional_branch_common_32(
            instruction,
            instruction.opcode + 0x10,
            dest_address,
            source_address,
            buf,
        )
    } else {
        unreachable!();
    }
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x86")]
pub(crate) fn patch_conditional_branch_common_32(
    instruction: &ZydisInstruction,
    opcode: u8,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    /*
        Jump conditionals:

        70  JO    Jump short if overflow (OF=1)                           rel8
        71  JNO   Jump short if not overflow (OF=0)                       rel8
        72  JB    Jump short if below/not above or equal/carry (CF=1)     rel8
        73  JNB   Jump short if not below/above or equal/not carry (CF=0) rel8
        74  JZ    Jump short if zero/equal (ZF=1)                         rel8
        75  JNZ   Jump short if not zero/not equal (ZF=0)                 rel8
        76  JBE   Jump short if below or equal/not above (CF=1 OR ZF=1)   rel8
        77  JNBE  Jump short if not below or equal/above (CF=0 AND ZF=0)  rel8
        78  JS    Jump short if sign (SF=1)                               rel8
        79  JNS   Jump short if not sign (SF=0)                           rel8
        7A  JP    Jump short if parity/parity even (PF=1)                 rel8
        7B  JNP   Jump short if not parity/parity odd (PF=0)              rel8
        7C  JL    Jump short if less/not greater (SF!=OF)                 rel8
        7D  JNL   Jump short if not less/greater or equal (SF=OF)         rel8
        7E  JLE   Jump short if less or equal/not greater ((ZF=1) OR (SF!=OF)) rel8
        7F  JNLE  Jump short if not less nor equal/greater ((ZF=0) AND (SF=OF)) rel8

        Toggling the 1/odd bit allows us to invert the opcode.
        Same for the extended `0F 8X` variants.

        For x86, we can reach any address from any address, so we don't need to do
        anything special, but worth keeping in mind.
    */

    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    let next_pc = ((*dest_address).wrapping_add(INS_SIZE_REL)) as u64;
    let delta = target.wrapping_sub(next_pc) as i64;

    encode_rel32(
        source_address,
        instruction,
        dest_address,
        buf,
        opcode,
        delta,
    );

    Ok(())
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_conditional_branch_64(
    instruction: &ZydisInstruction,
    instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: Option<x64::Register>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    if instruction_bytes.len() == 6 {
        patch_conditional_branch_common_64(
            instruction,
            instruction_bytes[1], // lift opcode out from original instruction
            dest_address,
            source_address,
            buf,
            scratch_register,
        )
    } else if instruction_bytes.len() == 2 {
        patch_conditional_branch_common_64(
            instruction,
            instruction.opcode + 0x10,
            dest_address,
            source_address,
            buf,
            scratch_register,
        )
    } else {
        unreachable!();
    }
}

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_conditional_branch_common_64(
    instruction: &ZydisInstruction,
    opcode: u8,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    buf: &mut Vec<u8>,
    scratch_register: Option<x64::Register>,
) -> Result<(), CodeRewriterError> {
    use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;

    /*
        Jump conditionals:

        70  JO    Jump short if overflow (OF=1)                           rel8
        71  JNO   Jump short if not overflow (OF=0)                       rel8
        72  JB    Jump short if below/not above or equal/carry (CF=1)     rel8
        73  JNB   Jump short if not below/above or equal/not carry (CF=0) rel8
        74  JZ    Jump short if zero/equal (ZF=1)                         rel8
        75  JNZ   Jump short if not zero/not equal (ZF=0)                 rel8
        76  JBE   Jump short if below or equal/not above (CF=1 OR ZF=1)   rel8
        77  JNBE  Jump short if not below or equal/above (CF=0 AND ZF=0)  rel8
        78  JS    Jump short if sign (SF=1)                               rel8
        79  JNS   Jump short if not sign (SF=0)                           rel8
        7A  JP    Jump short if parity/parity even (PF=1)                 rel8
        7B  JNP   Jump short if not parity/parity odd (PF=0)              rel8
        7C  JL    Jump short if less/not greater (SF!=OF)                 rel8
        7D  JNL   Jump short if not less/greater or equal (SF=OF)         rel8
        7E  JLE   Jump short if less or equal/not greater ((ZF=1) OR (SF!=OF)) rel8
        7F  JNLE  Jump short if not less nor equal/greater ((ZF=0) AND (SF=OF)) rel8

        Toggling the 1/odd bit allows us to invert the opcode.
        Same for the extended `0F 8X` variants.
    */
    use crate::instructions::jump_absolute::encode_absolute_jump_x64;

    let target = instruction
        .calc_absolute_address(*source_address as u64, &instruction.operands()[0])
        .unwrap();

    let next_pc = ((*dest_address).wrapping_add(INS_SIZE_REL)) as u64;
    let delta = target.wrapping_sub(next_pc) as i64;

    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        // Hot path, we can re-encode because it's in jump range.
        encode_rel32(
            source_address,
            instruction,
            dest_address,
            buf,
            opcode,
            delta,
        );
        return Ok(());
    }

    // Cold path, we need to emulate the instruction if beyond 2GiB
    /*
        e.g.
        Before:
        jo 0x8000000000000004

        After:
        jno 12
        mov rax, 0x8000000000000004
        jmp rax
    */
    const INS_SIZE_ABS: usize = 14;
    let mut pc_jmp_abs = *dest_address + 2;

    *source_address += instruction.length as usize;
    *dest_address += INS_SIZE_ABS;

    let scratch_reg =
        scratch_register.ok_or_else(|| CodeRewriterError::NoScratchRegister(Default::default()))?;

    let old_len = buf.len();
    buf.reserve(INS_SIZE_ABS);

    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        // write skip over next 2 instructions
        ptr.write((opcode - 0x10) ^ 0b1); // invert condition
        ptr.add(1).write(0xA); // skips next 2 instructions
        buf.set_len(old_len + 2);

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

#[inline]
fn encode_rel32(
    source_address: &mut usize,
    instruction: &zydis::Instruction<zydis::OperandArrayVec<5>>,
    dest_address: &mut usize,
    buf: &mut Vec<u8>,
    opcode: u8,
    delta: i64,
) {
    *source_address += instruction.length as usize;
    *dest_address += INS_SIZE_REL;

    let old_len = buf.len();
    buf.reserve(INS_SIZE_REL);
    unsafe {
        let ptr = buf.as_mut_ptr().add(old_len);

        ptr.write(0x0F);
        ptr.add(1).write(opcode);
        write_unaligned(ptr.add(2) as *mut i32, delta as i32);

        buf.set_len(old_len + INS_SIZE_REL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::test_relocate_instruction;
    use crate::x64;
    use crate::x86;
    use rstest::rstest;

    #[rstest]
    // JNO (Jump if Overflow)
    #[case::jo("7002", 4096, 0, "0f80fe0f0000")] // jo 4100
    #[case::jo_with_underflow("70f4", 0, 4096, "0f80f0efffff")] // jo 0xfffffff6
    #[case::jo_upper32("7002", 0x80001000, 0x80000000, "0f80fe0f0000")] // jo 0x80001004

    // JBE (Jump if Below)
    #[case::jb("7202", 4096, 0, "0f82fe0f0000")] // jb 4100
    #[case::jb_with_underflow("72f4", 0, 4096, "0f82f0efffff")] // jb 0xfffffff6
    #[case::jb_upper32("7202", 0x80001000, 0x80000000, "0f82fe0f0000")] // jb 0x80001004

    // JZ (Jump if Zero / Equal)
    #[case::jz("7402", 4096, 0, "0f84fe0f0000")] // jz 4100
    #[case::jz_with_underflow("74f4", 0, 4096, "0f84f0efffff")] // jz 0xfffffff6
    #[case::jz_upper32("7402", 0x80001000, 0x80000000, "0f84fe0f0000")] // jz 0x80001004

    // JBE (Jump if Below or Equal / Not Above)
    #[case::jbe("7602", 4096, 0, "0f86fe0f0000")] // jbe 4100
    #[case::jbe_with_underflow("76f4", 0, 4096, "0f86f0efffff")] // jbe 0xfffffff6
    #[case::jbe_upper32("7602", 0x80001000, 0x80000000, "0f86fe0f0000")] // jbe 0x80001004

    // JS (Jump if Sign / Negative)
    #[case::js("7802", 4096, 0, "0f88fe0f0000")] // js 4100
    #[case::js_with_underflow("78f4", 0, 4096, "0f88f0efffff")] // js 0xfffffff6
    #[case::js_upper32("7802", 0x80001000, 0x80000000, "0f88fe0f0000")] // js 0x80001004

    // JP (Jump if Parity / Parity Even)
    #[case::jp("7a02", 4096, 0, "0f8afe0f0000")] // jp 4100
    #[case::jp_with_underflow("7af4", 0, 4096, "0f8af0efffff")] // jp 0xfffffff6
    #[case::jp_upper32("7a02", 0x80001000, 0x80000000, "0f8afe0f0000")] // jp 0x80001004

    // JL (Jump if Less / Not Greater)
    #[case::jl("7c02", 4096, 0, "0f8cfe0f0000")] // jl 4100
    #[case::jl_with_underflow("7cf4", 0, 4096, "0f8cf0efffff")] // jl 0xfffffff6
    #[case::jl_upper32("7c02", 0x80001000, 0x80000000, "0f8cfe0f0000")] // jl 0x80001004

    // JLE (Jump if Less or Equal / Not Greater)
    #[case::jle("7e02", 4096, 0, "0f8efe0f0000")] // jle 4100
    #[case::jle_with_underflow("7ef4", 0, 4096, "0f8ef0efffff")] // jle 0xfffffff6
    #[case::jle_upper32("7e02", 0x80001000, 0x80000000, "0f8efe0f0000")] // jle 0x80001004

    // JNO (Jump if Not Overflow)
    #[case::jno("7102", 4096, 0, "0f81fe0f0000")] // jno 4100
    #[case::jno_with_underflow("71f4", 0, 4096, "0f81f0efffff")] // jno 0xfffffff6
    #[case::jno_upper32("7102", 0x80001000, 0x80000000, "0f81fe0f0000")] // jno 0x80001004

    // JNB (Jump if Not Below / Above or Equal)
    #[case::jnb("7302", 4096, 0, "0f83fe0f0000")] // jnb 4100
    #[case::jnb_with_underflow("73f4", 0, 4096, "0f83f0efffff")] // jnb 0xfffffff6
    #[case::jnb_upper32("7302", 0x80001000, 0x80000000, "0f83fe0f0000")] // jnb 0x80001004

    // JNZ (Jump if Not Zero / Not Equal)
    #[case::jnz("7502", 4096, 0, "0f85fe0f0000")] // jnz 4100
    #[case::jnz_with_underflow("75f4", 0, 4096, "0f85f0efffff")] // jnz 0xfffffff6
    #[case::jnz_upper32("7502", 0x80001000, 0x80000000, "0f85fe0f0000")] // jnz 0x80001004

    // JNBE (Jump if Not Below or Equal / Above)
    #[case::jnbe("7702", 4096, 0, "0f87fe0f0000")] // jnbe 4100
    #[case::jnbe_with_underflow("77f4", 0, 4096, "0f87f0efffff")] // jnbe 0xfffffff6
    #[case::jnbe_upper32("7702", 0x80001000, 0x80000000, "0f87fe0f0000")] // jnbe 0x80001004

    // JNS (Jump if Not Sign / Positive)
    #[case::jns("7902", 4096, 0, "0f89fe0f0000")] // jns 4100
    #[case::jns_with_underflow("79f4", 0, 4096, "0f89f0efffff")] // jns 0xfffffff6
    #[case::jns_upper32("7902", 0x80001000, 0x80000000, "0f89fe0f0000")] // jns 0x80001004

    // JNP (Jump if Not Parity / Parity Odd)
    #[case::jnp("7b02", 4096, 0, "0f8bfe0f0000")] // jnp 4100
    #[case::jnp_with_underflow("7bf4", 0, 4096, "0f8bf0efffff")] // jnp 0xfffffff6
    #[case::jnp_upper32("7b02", 0x80001000, 0x80000000, "0f8bfe0f0000")] // jnp 0x80001004

    // JNL (Jump if Not Less / Greater or Equal)
    #[case::jnl("7d02", 4096, 0, "0f8dfe0f0000")] // jnl 4100
    #[case::jnl_with_underflow("7df4", 0, 4096, "0f8df0efffff")] // jnl 0xfffffff6
    #[case::jnl_upper32("7d02", 0x80001000, 0x80000000, "0f8dfe0f0000")] // jnl 0x80001004

    // JNLE (Jump if Not Less or Equal / Greater)
    #[case::jnle("7f02", 4096, 0, "0f8ffe0f0000")] // jnle 4100
    #[case::jnle_with_underflow("7ff4", 0, 4096, "0f8ff0efffff")] // jnle 0xfffffff6
    #[case::jnle_upper32("7f02", 0x80001000, 0x80000000, "0f8ffe0f0000")] // jnle 0x80001004
    fn relocate_jmp_conditional_short_32(
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
            patch_conditional_branch_32, // the function being tested
        );
    }

    #[rstest]
    // JNO (Jump if Overflow)
    #[case::jo("0f80faf0ffff", 4096, 0, "0f80fa000000")] // jo 0x100
    #[case::jo_with_underflow("0f80f0efffff", 4096, 0, "0f80f0ffffff")] // jo 0xfffffff6

    // JZ (Jump if Zero / Equal)
    #[case::jz("0f84faf0ffff", 4096, 0, "0f84fa000000")] // jz 0x100
    #[case::jz_with_underflow("0f84f0efffff", 4096, 0, "0f84f0ffffff")] // jz 0xfffffff6

    // JBE (Jump if Below or Equal / Not Above)
    #[case::jbe("0f86faf0ffff", 4096, 0, "0f86fa000000")] // jbe 0x100
    #[case::jbe_with_underflow("0f86f0efffff", 4096, 0, "0f86f0ffffff")] // jbe 0xfffffff6

    // JS (Jump if Sign / Negative)
    #[case::js("0f88faf0ffff", 4096, 0, "0f88fa000000")] // js 0x100
    #[case::js_with_underflow("0f88f0efffff", 4096, 0, "0f88f0ffffff")] // js 0xfffffff6

    // JP (Jump if Parity / Parity Even)
    #[case::jp("0f8afaf0ffff", 4096, 0, "0f8afa000000")] // jp 0x100
    #[case::jp_with_underflow("0f8af0efffff", 4096, 0, "0f8af0ffffff")] // jp 0xfffffff6

    // JL (Jump if Less / Not Greater)
    #[case::jl("0f8cfaf0ffff", 4096, 0, "0f8cfa000000")] // jl 0x100
    #[case::jl_with_underflow("0f8cf0efffff", 4096, 0, "0f8cf0ffffff")] // jl 0xfffffff6

    // JLE (Jump if Less or Equal / Not Greater)
    #[case::jle("0f8efaf0ffff", 4096, 0, "0f8efa000000")] // jle 0x100
    #[case::jle_with_underflow("0f8ef0efffff", 4096, 0, "0f8ef0ffffff")] // jle 0xfffffff6

    // JNO (Jump if Not Overflow)
    #[case::jno("0f81faf0ffff", 4096, 0, "0f81fa000000")] // jno 0x100
    #[case::jno_with_underflow("0f81f0efffff", 4096, 0, "0f81f0ffffff")] // jno 0xfffffff6

    // JNB (Jump if Not Below / Above or Equal)
    #[case::jnb("0f83faf0ffff", 4096, 0, "0f83fa000000")] // jnb 0x100
    #[case::jnb_with_underflow("0f83f0efffff", 4096, 0, "0f83f0ffffff")] // jnb 0xfffffff6

    // JNZ (Jump if Not Zero / Not Equal)
    #[case::jnz("0f85faf0ffff", 4096, 0, "0f85fa000000")] // jnz 0x100
    #[case::jnz_with_underflow("0f85f0efffff", 4096, 0, "0f85f0ffffff")] // jnz 0xfffffff6

    // JNBE (Jump if Not Below or Equal / Above)
    #[case::jnbe("0f87faf0ffff", 4096, 0, "0f87fa000000")] // jnbe 0x100
    #[case::jnbe_with_underflow("0f87f0efffff", 4096, 0, "0f87f0ffffff")] // jnbe 0xfffffff6

    // JNS (Jump if Not Sign / Positive)
    #[case::jns("0f89faf0ffff", 4096, 0, "0f89fa000000")] // jns 0x100
    #[case::jns_with_underflow("0f89f0efffff", 4096, 0, "0f89f0ffffff")] // jns 0xfffffff6

    // JNP (Jump if Not Parity / Parity Odd)
    #[case::jnp("0f8bfaf0ffff", 4096, 0, "0f8bfa000000")] // jnp 0x100
    #[case::jnp_with_underflow("0f8bf0efffff", 4096, 0, "0f8bf0ffffff")] // jnp 0xfffffff6

    // JNL (Jump if Not Less / Greater or Equal)
    #[case::jnl("0f8dfaf0ffff", 4096, 0, "0f8dfa000000")] // jnl 0x100
    #[case::jnl_with_underflow("0f8df0efffff", 4096, 0, "0f8df0ffffff")] // jnl 0xfffffff6
    fn relocate_jmp_conditional_long_32(
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
            patch_conditional_branch_32, // the function being tested
        );
    }

    #[rstest]
    // JNO (Jump if Overflow)
    #[case::jo("7002", 4096, 0, "0f80fe0f0000")] // jo 4100
    #[case::jo_with_underflow("70f4", 0, 4096, "0f80f0efffff")] // jo 0xfffffff6
    #[case::jo_upper64("7002", 0x8000000000001000, 0x8000000000000000, "0f80fe0f0000")] // jo 0x80001004
    #[case::jo_abs_64b("7002", 0x8000000000000000, 0, "710a48b80400000000000080ffe0")]
    // jno 0x8000000000000004 -> jo +12 <skip> + mov rax, 0x8000000000000004 + jmp rax

    // JBE (Jump if Below)
    #[case::jb("7202", 4096, 0, "0f82fe0f0000")] // jb 4100
    #[case::jb_with_underflow("72f4", 0, 4096, "0f82f0efffff")] // jb 0xfffffff6
    #[case::jb_upper64("7202", 0x8000000000001000, 0x8000000000000000, "0f82fe0f0000")] // jb 0x80001004
    #[case::jo_abs_64b("7202", 0x8000000000000000, 0, "730a48b80400000000000080ffe0")]
    // jbe 0x8000000000000004 -> jne +12 <skip> + mov rax, 0x8000000000000004 + jmp rax

    // JZ (Jump if Zero / Equal)
    #[case::jz("7402", 4096, 0, "0f84fe0f0000")] // jz 4100
    #[case::jz_with_underflow("74f4", 0, 4096, "0f84f0efffff")] // jz 0xfffffff6
    #[case::jz_upper64("7402", 0x8000000000001000, 0x8000000000000000, "0f84fe0f0000")] // jz 0x80001004

    // JBE (Jump if Below or Equal / Not Above)
    #[case::jbe("7602", 4096, 0, "0f86fe0f0000")] // jbe 4100
    #[case::jbe_with_underflow("76f4", 0, 4096, "0f86f0efffff")] // jbe 0xfffffff6
    #[case::jbe_upper64("7602", 0x8000000000001000, 0x8000000000000000, "0f86fe0f0000")] // jbe 0x80001004

    // JS (Jump if Sign / Negative)
    #[case::js("7802", 4096, 0, "0f88fe0f0000")] // js 4100
    #[case::js_with_underflow("78f4", 0, 4096, "0f88f0efffff")] // js 0xfffffff6
    #[case::js_upper64("7802", 0x8000000000001000, 0x8000000000000000, "0f88fe0f0000")] // js 0x80001004

    // JP (Jump if Parity / Parity Even)
    #[case::jp("7a02", 4096, 0, "0f8afe0f0000")] // jp 4100
    #[case::jp_with_underflow("7af4", 0, 4096, "0f8af0efffff")] // jp 0xfffffff6
    #[case::jp_upper64("7a02", 0x8000000000001000, 0x8000000000000000, "0f8afe0f0000")] // jp 0x80001004

    // JL (Jump if Less / Not Greater)
    #[case::jl("7c02", 4096, 0, "0f8cfe0f0000")] // jl 4100
    #[case::jl_with_underflow("7cf4", 0, 4096, "0f8cf0efffff")] // jl 0xfffffff6
    #[case::jl_upper64("7c02", 0x8000000000001000, 0x8000000000000000, "0f8cfe0f0000")] // jl 0x80001004

    // JLE (Jump if Less or Equal / Not Greater)
    #[case::jle("7e02", 4096, 0, "0f8efe0f0000")] // jle 4100
    #[case::jle_with_underflow("7ef4", 0, 4096, "0f8ef0efffff")] // jle 0xfffffff6
    #[case::jle_upper64("7e02", 0x8000000000001000, 0x8000000000000000, "0f8efe0f0000")] // jle 0x80001004

    // JNO (Jump if Not Overflow)
    #[case::jno("7102", 4096, 0, "0f81fe0f0000")] // jno 4100
    #[case::jno_with_underflow("71f4", 0, 4096, "0f81f0efffff")] // jno 0xfffffff6
    #[case::jno_upper64("7102", 0x8000000000001000, 0x8000000000000000, "0f81fe0f0000")] // jno 0x80001004

    // JNB (Jump if Not Below / Above or Equal)
    #[case::jnb("7302", 4096, 0, "0f83fe0f0000")] // jnb 4100
    #[case::jnb_with_underflow("73f4", 0, 4096, "0f83f0efffff")] // jnb 0xfffffff6
    #[case::jnb_upper64("7302", 0x8000000000001000, 0x8000000000000000, "0f83fe0f0000")] // jnb 0x80001004

    // JNZ (Jump if Not Zero / Not Equal)
    #[case::jnz("7502", 4096, 0, "0f85fe0f0000")] // jnz 4100
    #[case::jnz_with_underflow("75f4", 0, 4096, "0f85f0efffff")] // jnz 0xfffffff6
    #[case::jnz_upper64("7502", 0x8000000000001000, 0x8000000000000000, "0f85fe0f0000")] // jnz 0x80001004

    // JNBE (Jump if Not Below or Equal / Above)
    #[case::jnbe("7702", 4096, 0, "0f87fe0f0000")] // jnbe 4100
    #[case::jnbe_with_underflow("77f4", 0, 4096, "0f87f0efffff")] // jnbe 0xfffffff6
    #[case::jnbe_upper64("7702", 0x8000000000001000, 0x8000000000000000, "0f87fe0f0000")] // jnbe 0x80001004

    // JNS (Jump if Not Sign / Positive)
    #[case::jns("7902", 4096, 0, "0f89fe0f0000")] // jns 4100
    #[case::jns_with_underflow("79f4", 0, 4096, "0f89f0efffff")] // jns 0xfffffff6
    #[case::jns_upper64("7902", 0x8000000000001000, 0x8000000000000000, "0f89fe0f0000")] // jns 0x80001004

    // JNP (Jump if Not Parity / Parity Odd)
    #[case::jnp("7b02", 4096, 0, "0f8bfe0f0000")] // jnp 4100
    #[case::jnp_with_underflow("7bf4", 0, 4096, "0f8bf0efffff")] // jnp 0xfffffff6
    #[case::jnp_upper64("7b02", 0x8000000000001000, 0x8000000000000000, "0f8bfe0f0000")] // jnp 0x80001004

    // JNL (Jump if Not Less / Greater or Equal)
    #[case::jnl("7d02", 4096, 0, "0f8dfe0f0000")] // jnl 4100
    #[case::jnl_with_underflow("7df4", 0, 4096, "0f8df0efffff")] // jnl 0xfffffff6
    #[case::jnl_upper64("7d02", 0x8000000000001000, 0x8000000000000000, "0f8dfe0f0000")] // jnl 0x80001004

    // JNLE (Jump if Not Less or Equal / Greater)
    #[case::jnle("7f02", 4096, 0, "0f8ffe0f0000")] // jnle 4100
    #[case::jnle_with_underflow("7ff4", 0, 4096, "0f8ff0efffff")] // jnle 0xfffffff6
    #[case::jnle_upper64("7f02", 0x8000000000001000, 0x8000000000000000, "0f8ffe0f0000")] // jnle 0x80001004
    fn relocate_jmp_conditional_short_64(
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
            patch_conditional_branch_64, // the function being tested
        );
    }

    #[rstest]
    // JNO (Jump if Overflow)
    #[case::jo("0f80faf0ffff", 4096, 0, "0f80fa000000")] // jo 0x100
    #[case::jo_with_underflow("0f80f0efffff", 4096, 0, "0f80f0ffffff")] // jo 0xfffffff6

    // JZ (Jump if Zero / Equal)
    #[case::jz("0f84faf0ffff", 4096, 0, "0f84fa000000")] // jz 0x100
    #[case::jz_with_underflow("0f84f0efffff", 4096, 0, "0f84f0ffffff")] // jz 0xfffffff6

    // JBE (Jump if Below or Equal / Not Above)
    #[case::jbe("0f86faf0ffff", 4096, 0, "0f86fa000000")] // jbe 0x100
    #[case::jbe_with_underflow("0f86f0efffff", 4096, 0, "0f86f0ffffff")] // jbe 0xfffffff6

    // JS (Jump if Sign / Negative)
    #[case::js("0f88faf0ffff", 4096, 0, "0f88fa000000")] // js 0x100
    #[case::js_with_underflow("0f88f0efffff", 4096, 0, "0f88f0ffffff")] // js 0xfffffff6

    // JP (Jump if Parity / Parity Even)
    #[case::jp("0f8afaf0ffff", 4096, 0, "0f8afa000000")] // jp 0x100
    #[case::jp_with_underflow("0f8af0efffff", 4096, 0, "0f8af0ffffff")] // jp 0xfffffff6

    // JL (Jump if Less / Not Greater)
    #[case::jl("0f8cfaf0ffff", 4096, 0, "0f8cfa000000")] // jl 0x100
    #[case::jl_with_underflow("0f8cf0efffff", 4096, 0, "0f8cf0ffffff")] // jl 0xfffffff6

    // JLE (Jump if Less or Equal / Not Greater)
    #[case::jle("0f8efaf0ffff", 4096, 0, "0f8efa000000")] // jle 0x100
    #[case::jle_with_underflow("0f8ef0efffff", 4096, 0, "0f8ef0ffffff")] // jle 0xfffffff6

    // JNO (Jump if Not Overflow)
    #[case::jno("0f81faf0ffff", 4096, 0, "0f81fa000000")] // jno 0x100
    #[case::jno_with_underflow("0f81f0efffff", 4096, 0, "0f81f0ffffff")] // jno 0xfffffff6

    // JNB (Jump if Not Below / Above or Equal)
    #[case::jnb("0f83faf0ffff", 4096, 0, "0f83fa000000")] // jnb 0x100
    #[case::jnb_with_underflow("0f83f0efffff", 4096, 0, "0f83f0ffffff")] // jnb 0xfffffff6

    // JNZ (Jump if Not Zero / Not Equal)
    #[case::jnz("0f85faf0ffff", 4096, 0, "0f85fa000000")] // jnz 0x100
    #[case::jnz_with_underflow("0f85f0efffff", 4096, 0, "0f85f0ffffff")] // jnz 0xfffffff6

    // JNBE (Jump if Not Below or Equal / Above)
    #[case::jnbe("0f87faf0ffff", 4096, 0, "0f87fa000000")] // jnbe 0x100
    #[case::jnbe_with_underflow("0f87f0efffff", 4096, 0, "0f87f0ffffff")] // jnbe 0xfffffff6

    // JNS (Jump if Not Sign / Positive)
    #[case::jns("0f89faf0ffff", 4096, 0, "0f89fa000000")] // jns 0x100
    #[case::jns_with_underflow("0f89f0efffff", 4096, 0, "0f89f0ffffff")] // jns 0xfffffff6

    // JNP (Jump if Not Parity / Parity Odd)
    #[case::jnp("0f8bfaf0ffff", 4096, 0, "0f8bfa000000")] // jnp 0x100
    #[case::jnp_with_underflow("0f8bf0efffff", 4096, 0, "0f8bf0ffffff")] // jnp 0xfffffff6

    // JNL (Jump if Not Less / Greater or Equal)
    #[case::jnl("0f8dfaf0ffff", 4096, 0, "0f8dfa000000")] // jnl 0x100
    #[case::jnl_with_underflow("0f8df0efffff", 4096, 0, "0f8df0ffffff")] // jnl 0xfffffff6
    fn relocate_jmp_conditional_long_64(
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
            patch_conditional_branch_64, // the function being tested
        );
    }
}
