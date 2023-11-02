extern crate alloc;

use super::aarch64_rewriter::{
    emit_mov_const_to_reg, emit_mov_upper_48_bits_const_to_reg, load_address_4g_with_remainder,
    InstructionRewriteResult,
};
use crate::{
    all_registers::AllRegisters,
    instructions::{
        ldr_immediate_unsigned_offset::LdrImmediateUnsignedOffset, ldr_literal::LdrLiteral,
    },
};
use alloc::vec::Vec;
use reloaded_hooks_portable::api::{
    jit::compiler::JitError, rewriter::code_rewriter::CodeRewriterError,
};

/// Rewrites the `LDR Literal` (Load) instruction for a new address.
/// This also covers STR, but for simplicity, we will just refer to it as LDR from now.
///
/// The `LDR Literal` instruction loads a value from a PC offset.
/// specific condition flags. This function is designed to modify the `Bcc` instruction's
/// encoding to adjust for a new memory location.
///
/// # Parameters
///
/// * `instruction`: The original `LDR` instruction encoded as a 32-bit value.
/// * `old_address`: The original address associated with the `LDR` instruction.
/// * `new_address`: The new address of the instruction.
///
/// # Behaviour
///
/// The Branch Conditional instruction is rewritten as one of the following:
/// - LDR Literal
/// - ADRP + LDR (w/ Unsigned Offset)
/// - MOV Address to Register + LDR
pub(crate) fn rewrite_ldr_literal(
    instruction: u32,
    old_address: usize,
    new_address: usize,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    let orig_ins = LdrLiteral(instruction.to_le());
    let orig_target = (old_address as isize).wrapping_add(orig_ins.offset() as isize);
    let delta = orig_target.wrapping_sub(new_address as isize);

    // Output as another LDR if within 1MiB range
    if (-0x100000..=0xFFFFF).contains(&delta) {
        return Ok(InstructionRewriteResult::LdrLiteral(
            LdrLiteral::new_load_literal(
                orig_ins.mode(),
                orig_ins.rt(),
                orig_ins.is_simd(),
                delta as i32,
            )
            .unwrap()
            .0
            .to_le(),
        ));
    }

    // If it's a 'prefetch' operation, discard it entirely; because prefetch with multiple
    // operations will be less efficient.
    if orig_ins.mode() == 0b11 {
        return Ok(InstructionRewriteResult::None);
    }

    // Output as:
    // - ADRP
    // - ADD (Optional)
    // - LDR
    // + 4 for 'next instruction', since we are placing a BCC first.
    if (-0x100000000..=0xFFFFFFFF).contains(&delta) {
        let load = load_address_4g_with_remainder(new_address, orig_target as usize, orig_ins.rt());
        let ldr = get_ldr_immediate_for_literal(orig_ins, load.1 as i32)
            .unwrap()
            .0
            .to_le();
        return Ok(InstructionRewriteResult::AdrpAndLdrUnsignedOffset(
            load.0, ldr,
        ));
    }

    // Output as:
    // - MOV Immediate
    // - LDR
    let mov_instr = emit_mov_upper_48_bits_const_to_reg(orig_ins.rt(), orig_target as usize); // Assuming emit_mov_const_to_reg can handle this as well
    let ldr = get_ldr_immediate_for_literal(orig_ins, (orig_target & 0xFFFF) as i32);

    let mut result = Vec::new();
    mov_instr.append_to_buffer(&mut result);
    result.push(ldr.unwrap().0.to_le());

    Ok(InstructionRewriteResult::MovImmediateAndLdrLiteral(
        result.into_boxed_slice(),
    ))
}

fn get_ldr_immediate_for_literal(
    orig_ins: LdrLiteral,
    offset: i32,
) -> Result<LdrImmediateUnsignedOffset, JitError<AllRegisters>> {
    if orig_ins.is_simd() {
        LdrImmediateUnsignedOffset::new_mov_from_reg_vector(orig_ins.rt(), orig_ins.rt(), offset)
    } else {
        // 32 bit
        if orig_ins.mode() == 0b00 {
            LdrImmediateUnsignedOffset::new_mov_from_reg(
                false,
                orig_ins.rt(),
                offset,
                orig_ins.rt(),
            )
        }
        // 64 bit
        else if orig_ins.mode() == 0b01 {
            LdrImmediateUnsignedOffset::new_mov_from_reg(true, orig_ins.rt(), offset, orig_ins.rt())
        }
        // 32 bit, signed
        else {
            // Intuition says:
            // https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDR--immediate---Load-Register--immediate--
            // 'Shared Decode' section.

            // But, disassemblers don't like this, so we're using
            // https://developer.arm.com/documentation/ddi0602/2022-03/Base-Instructions/LDRSW--immediate---Load-Register-Signed-Word--immediate--
            LdrImmediateUnsignedOffset::new_mov_from_reg_with_opc(
                false,
                orig_ins.rt(),
                offset,
                orig_ins.rt(),
                0b10,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::ToHexString;
    use rstest::rstest;

    #[rstest]
    // [Within 1MiB]
    #[case::ldr_64bit_1MiB(0x00000058_u32.to_be(), 4096, 0, "00800058")] // LDR x0, #0 -> LDR x0, #4096
    #[case::ldr_32bit_1MiB(0x00000018_u32.to_be(), 4096, 0, "00800018")] // LDR w0, #0 -> LDR w0, #4096
    #[case::ldrsw_32bit_1MiB(0x00000098_u32.to_be(), 4096, 0, "00800098")] // LDRSW x0, #0 -> LDRSW x0, #4096
    #[case::prfm_1MiB(0x080000D8_u32.to_be(), 4096, 0, "088000d8")] // PRFM PLIL1KEEP, #0 -> PRFM PLIL1KEEP, #4096

    // [Within 4GiB + 4096 aligned]
    #[case::ldr_64bit_4GiB_aligned(0x00000058_u32.to_be(), 0x100000, 0, "00080090000040f9")] // LDR x0, #0 -> adrp x0, #0x100000 + ldr x0, [x0]
    #[case::ldr_32bit_4GiB_aligned(0x00000018_u32.to_be(), 0x100000, 0, "00080090000040b9")] // LDR w0, #0 -> adrp x0, #0x100000 + ldr w0, [x0]
    #[case::ldrsw_32bit_4GiB_aligned(0x00000098_u32.to_be(), 0x100000, 0, "00080090000080b9")] // LDRSW w0, #0 -> adrp x0, #0x100000 + ldrsw x0, [x0]
    #[case::prfm_4GiB_aligned(0x080000D8_u32.to_be(), 0x100000, 0, "")] // PRFM PLIL1KEEP, #0 -> none

    // [Within 4GiB]
    #[case::ldr_64bit_4GiB(0x00100058_u32.to_be(), 0x100000, 0, "00080090000041f9")] // LDR x0, #512 -> adrp x0, #0x100000 + ldr x0, [x0, #512]
    #[case::ldr_32bit_4GiB(0x00100018_u32.to_be(), 0x100000, 0, "00080090000042b9")] // LDR w0, #512 -> adrp x0, #0x100000 + ldr w0, [x0, #512]
    #[case::ldrsw_32bit_4GiB(0x00100098_u32.to_be(), 0x100000, 0, "00080090000082b9")] // LDRSW w0, #512 -> adrp x0, #0x100000 + ldrsw x0, [x0, #512]
    #[case::prfm_4GiB(0x081000D8_u32.to_be(), 0x100000, 0, "")] // PRFM PLIL1KEEP, #512 -> none
    // [Last Resort]
    #[case::ldr_64bit_4GiB(0x00100058_u32.to_be(), 0x100000000, 0, "0000a0d22000c0f2000041f9")] // LDR x0, #512 -> mov + ldr x0, [x0, #512]
    #[case::ldr_32bit_4GiB(0x00100018_u32.to_be(), 0x100000000, 0, "0000a0d22000c0f2000042b9")] // LDR w0, #512 -> mov + ldr w0, [x0, #512]
    #[case::ldrsw_32bit_4GiB(0x00100098_u32.to_be(), 0x100000000, 0, "0000a0d22000c0f2000082b9")] // LDRSW w0, #512 -> mov + ldrsw x0, [x0, #512]
    #[case::prfm_4GiB(0x081000D8_u32.to_be(), 0x100000000, 0, "")] // PRFM PLIL1KEEP, #512 -> none
    fn test_rewrite_ldr(
        #[case] old_instruction: u32,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
    ) {
        let result = rewrite_ldr_literal(old_instruction, old_address, new_address).unwrap();
        assert_eq!(result.to_hex_string(), expected_hex);
    }
}
