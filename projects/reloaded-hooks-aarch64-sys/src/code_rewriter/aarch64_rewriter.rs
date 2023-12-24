extern crate alloc;

use super::{
    instruction_rewrite_result::InstructionRewriteResult,
    instructions::{
        adr::rewrite_adr, b::rewrite_b, b_cond::rewrite_bcc, cbz::rewrite_cbz,
        ldr_literal::rewrite_ldr_literal, tbz::rewrite_tbz,
    },
};
use crate::helpers::{vec_u32_to_u8, vec_u8_to_u32};
use alloc::vec::Vec;
use core::{
    mem::{self},
    ptr::read_unaligned,
};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;

/// Rewrites the code from one address to another.
///
/// Given an original block of code starting at `old_address`, this function
/// will modify any relative addressing instructions to make them compatible
/// with a new location starting at `new_address`.
///
/// This is useful, for example, when code is being moved or injected into a new
/// location in memory and any relative jumps or calls within the code need to be
/// adjusted to the new location.
///
/// # Parameters
///
/// * `old_code`: A pointer to the start of the original block of code.
/// * `old_code_size`: Amount of bytes to rewrite.
/// * `old_address`: The address to assume as the source location of the old code.
/// * `new_address`: The new address for the instructions.
/// * `scratch_register`
///     - A scratch general purpose register that can be used for operations.
///     - This scratch register may or may not be used depending on the code being rewritten.
///
/// # Behaviour
///
/// The function will iterate over the block of code byte by byte, identifying any
/// instructions that use relative addressing. When such an instruction is identified,
/// its offset is adjusted to account for the difference between `old_address` and `new_address`.
///
/// # Returns
///
/// Either a re-encode error, in which case the operation fails, or a vector to consume.
pub(crate) fn rewrite_code_aarch64(
    old_code: *const u8,
    old_code_size: usize,
    old_address: usize,
    new_address: usize,
    scratch_register: Option<u8>,
    existing_buffer: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    // Note: Not covered by unit tests (because we read from old_address), careful when modifying.
    let mut vec = vec_u8_to_u32(mem::take(existing_buffer));
    let mut old_addr_ptr = old_address as *mut u32;
    let mut old_ins_ptr = old_code as *mut u32;
    let old_ins_end_ptr = (old_code.wrapping_add(old_code_size)) as *mut u32;
    let mut current_new_address = new_address;

    while old_ins_ptr < old_ins_end_ptr {
        let instruction = unsafe { read_unaligned(old_ins_ptr) };

        // Rewrite instruction.
        let result = rewrite_instruction(
            instruction.to_le(),
            current_new_address,
            old_addr_ptr as usize,
            scratch_register,
        );

        match result {
            Ok(x) => {
                x.append_to_buffer(&mut vec);

                // Advance pointers
                old_ins_ptr = old_ins_ptr.wrapping_add(1);
                old_addr_ptr = old_addr_ptr.wrapping_add(1);
                current_new_address = current_new_address.wrapping_add(x.size_bytes());
            }
            Err(x) => return Err(x),
        }
    }

    *existing_buffer = vec_u32_to_u8(vec);
    Ok(())
}

fn rewrite_instruction(
    instruction: u32,
    dest_address: usize,
    source_address: usize,
    scratch_register: Option<u8>,
) -> Result<InstructionRewriteResult, CodeRewriterError> {
    // Note: Converted to little endian inside each of the functions here
    if is_adr(instruction) {
        Ok(rewrite_adr(instruction, source_address, dest_address))
    } else if is_bcc(instruction) {
        rewrite_bcc(instruction, source_address, dest_address, scratch_register)
    } else if is_b_or_bl(instruction) {
        rewrite_b(instruction, source_address, dest_address, scratch_register)
    } else if is_cbz(instruction) {
        rewrite_cbz(instruction, source_address, dest_address, scratch_register)
    } else if is_tbz(instruction) {
        rewrite_tbz(instruction, source_address, dest_address, scratch_register)
    } else if is_ldr_literal(instruction) {
        rewrite_ldr_literal(instruction, source_address, dest_address)
    } else {
        Ok(InstructionRewriteResult::Copy(instruction))
    }
}

pub(crate) fn is_adr(instruction: u32) -> bool {
    (instruction & 0x1f000000) == 0x10000000
}

pub(crate) fn is_bcc(instruction: u32) -> bool {
    (instruction & 0xff000010) == 0x54000000
}

pub(crate) fn is_b_or_bl(instruction: u32) -> bool {
    (instruction & 0x7c000000) == 0x14000000
}

pub(crate) fn is_cbz(instruction: u32) -> bool {
    (instruction & 0x7e000000) == 0x34000000
}

pub(crate) fn is_tbz(instruction: u32) -> bool {
    (instruction & 0x7e000000) == 0x36000000
}

pub(crate) fn is_ldr_literal(instruction: u32) -> bool {
    (instruction & 0x3b000000) == 0x18000000
}

#[cfg(test)]
mod tests {
    use crate::{
        code_rewriter::aarch64_rewriter::rewrite_code_aarch64,
        test_helpers::instruction_buffer_as_hex_u8,
    };

    use super::{is_adr, is_b_or_bl, is_bcc, is_cbz, is_ldr_literal, is_tbz};
    use rstest::rstest;

    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum InsType {
        Adr,
        Bcc,
        B, // and BL
        Cbz,
        Tbz,
        LdrLiteral, // including PRFM
        Unknown,
    }

    fn get_ins_type(instruction: u32) -> InsType {
        if is_adr(instruction) {
            InsType::Adr
        } else if is_bcc(instruction) {
            InsType::Bcc
        } else if is_b_or_bl(instruction) {
            InsType::B
        } else if is_cbz(instruction) {
            InsType::Cbz
        } else if is_tbz(instruction) {
            InsType::Tbz
        } else if is_ldr_literal(instruction) {
            InsType::LdrLiteral
        } else {
            InsType::Unknown
        }
    }

    // This is not a complete list (I used disasm.pro, and it didn't support all opcodes).
    // This is an AI generated list of mostly all opcodes, cleaned up by human
    #[rstest]
    #[case::adc(0x2000029A_u32.to_be(), InsType::Unknown)] // adc x0, x1, x2
    #[case::adcs(0x200002BA_u32.to_be(), InsType::Unknown)] // adcs x0, x1, x2
    #[case::add(0x00040091_u32.to_be(), InsType::Unknown)] // add x0, x0, #1
    #[case::addp(0x00BCE14E_u32.to_be(), InsType::Unknown)] // addp v0.2d, v0.2d, v1.2d
    #[case::adds(0x000800B1_u32.to_be(), InsType::Unknown)] // adds x0, x0, #2
    #[case::adr(0x20000010_u32.to_be(), InsType::Adr)] // adr x0, #4
    #[case::adrp(0x00000090_u32.to_be(), InsType::Adr)] // adrp x0, #0
    #[case::and(0x001C4092_u32.to_be(), InsType::Unknown)] // and x0, x0, #0xff
    #[case::ands(0x000001EA_u32.to_be(), InsType::Unknown)] // ands x0, x0, x1
    #[case::asr(0x00FC4393_u32.to_be(), InsType::Unknown)] // asr x0, x0, #3
    #[case::b(0xF6FFFF17_u32.to_be(), InsType::B)] // b #0
    #[case::b_eq(0xA0FEFF54_u32.to_be(), InsType::Bcc)] // b.eq #0
    #[case::bl(0xF4FFFF97_u32.to_be(), InsType::B)] // bl #0
    #[case::blr(0x00003FD6_u32.to_be(), InsType::Unknown)] // blr x0
    #[case::br(0x00001FD6_u32.to_be(), InsType::Unknown)] // br x0
    #[case::brk(0x000020D4_u32.to_be(), InsType::Unknown)] // brk #0
    #[case::cbnz(0x00FEFFB5_u32.to_be(), InsType::Cbz)] // cbnz x0, #0
    #[case::cbz(0xE0FDFFB4_u32.to_be(), InsType::Cbz)] // cbz x0, #0
    #[case::ccmn(0x001840BA_u32.to_be(), InsType::Unknown)] // ccmn x0, #0, #0x0, ne
    #[case::ccmp(0x000041FA_u32.to_be(), InsType::Unknown)] // ccmp x0, x1, #0x0, eq
    #[case::cinc(0x0004809A_u32.to_be(), InsType::Unknown)] // cinc x0, x0, ne
    #[case::cinv(0x001080DA_u32.to_be(), InsType::Unknown)] // cinv x0, x0, eq
    #[case::clrex(0x5F3F03D5_u32.to_be(), InsType::Unknown)] // clrex
    #[case::cls(0x0014C0DA_u32.to_be(), InsType::Unknown)] // cls x0, x0
    #[case::clz(0x0010C0DA_u32.to_be(), InsType::Unknown)] // clz x0, x0
    #[case::cmn(0x1F0400B1_u32.to_be(), InsType::Unknown)] // cmn x0, #1
    #[case::cmp(0x1FFC03F1_u32.to_be(), InsType::Unknown)] // cmp x0, #0xff
    #[case::cneg(0x001480DA_u32.to_be(), InsType::Unknown)] // cneg x0, x0, eq
    #[case::crc32b(0x2040C21A_u32.to_be(), InsType::Unknown)] // crc32b w0, w1, w2
    #[case::crc32cb(0x2050C21A_u32.to_be(), InsType::Unknown)] // crc32cb w0, w1, w2
    #[case::crc32ch(0x2054C21A_u32.to_be(), InsType::Unknown)] // crc32ch w0, w1, w2
    #[case::crc32cw(0x2058C21A_u32.to_be(), InsType::Unknown)] // crc32cw w0, w1, w2
    #[case::crc32cx(0x205CC29A_u32.to_be(), InsType::Unknown)] // crc32cx w0, w1, x2
    #[case::crc32h(0x2044C21A_u32.to_be(), InsType::Unknown)] // crc32h w0, w1, w2
    #[case::crc32w(0x2048C21A_u32.to_be(), InsType::Unknown)] // crc32w w0, w1, w2
    #[case::crc32x(0x204CC29A_u32.to_be(), InsType::Unknown)] // crc32x w0, w1, x2
    #[case::csel(0x2000829A_u32.to_be(), InsType::Unknown)] // csel x0, x1, x2, eq
    #[case::cset(0xE0079F9A_u32.to_be(), InsType::Unknown)] // cset x0, ne
    #[case::csetm(0xE0139FDA_u32.to_be(), InsType::Unknown)] // csetm x0, eq
    #[case::csinc(0x2014829A_u32.to_be(), InsType::Unknown)] // csinc x0, x1, x2, ne
    #[case::csinv(0x200082DA_u32.to_be(), InsType::Unknown)] // csinv x0, x1, x2, eq
    #[case::csneg(0x201482DA_u32.to_be(), InsType::Unknown)] // csneg x0, x1, x2, ne
    #[case::dcps1(0x0100A0D4_u32.to_be(), InsType::Unknown)] // dcps1
    #[case::dcps2(0x0200A0D4_u32.to_be(), InsType::Unknown)] // dcps2
    #[case::dcps3(0x0300A0D4_u32.to_be(), InsType::Unknown)] // dcps3
    #[case::eon(0x200022CA_u32.to_be(), InsType::Unknown)] // eon x0, x1, x2
    #[case::eor(0x000001CA_u32.to_be(), InsType::Unknown)] // eor x0, x0, x1
    #[case::eret(0xE0039FD6_u32.to_be(), InsType::Unknown)] // eret
    #[case::extr(0x200CC293_u32.to_be(), InsType::Unknown)] // extr x0, x1, x2, #3
    #[case::fabd(0x20D4A27E_u32.to_be(), InsType::Unknown)] // fabd s0, s1, s2
    #[case::fabs(0x20C0201E_u32.to_be(), InsType::Unknown)] // fabs s0, s1
    #[case::facge(0x20EC227E_u32.to_be(), InsType::Unknown)] // facge s0, s1, s2
    #[case::facgt(0x20ECA27E_u32.to_be(), InsType::Unknown)] // facgt s0, s1, s2
    #[case::fadd(0x2028221E_u32.to_be(), InsType::Unknown)] // fadd s0, s1, s2
    #[case::faddp(0x00D4202E_u32.to_be(), InsType::Unknown)] // faddp v0.2s, v0.2s, v0.2s
    #[case::fccmp(0x0014211E_u32.to_be(), InsType::Unknown)] // fccmp s0, s1, #0x0, ne
    #[case::fccmpe(0x1014211E_u32.to_be(), InsType::Unknown)] // fccmpe s0, s1, #0x0, ne
    #[case::fcmeq(0x20E4225E_u32.to_be(), InsType::Unknown)] // fcmeq s0, s1, s2
    #[case::fcmge(0x20E4227E_u32.to_be(), InsType::Unknown)] // fcmge s0, s1, s2
    #[case::fcmgt(0x20E4A27E_u32.to_be(), InsType::Unknown)] // fcmgt s0, s1, s2
    #[case::fcmp(0x0020211E_u32.to_be(), InsType::Unknown)] // fcmp s0, s1
    #[case::fcmpe(0x1020211E_u32.to_be(), InsType::Unknown)] // fcmpe s0, s1
    #[case::fcsel(0x200C221E_u32.to_be(), InsType::Unknown)] // fcsel s0, s1, s2, eq
    #[case::fdiv(0x2018221E_u32.to_be(), InsType::Unknown)] // fdiv s0, s1, s2
    #[case::fmadd(0x200C021F_u32.to_be(), InsType::Unknown)] // fmadd s0, s1, s2, s3
    #[case::fmax(0x2048221E_u32.to_be(), InsType::Unknown)] // fmax s0, s1, s2
    #[case::fmaxnm(0x2068221E_u32.to_be(), InsType::Unknown)] // fmaxnm s0, s1, s2
    #[case::fmaxnmp(0x00C4212E_u32.to_be(), InsType::Unknown)] // fmaxnmp v0.2s, v0.2s, v1.2s
    #[case::fmaxp(0x00F4212E_u32.to_be(), InsType::Unknown)] // fmaxp v0.2s, v0.2s, v1.2s
    #[case::fmin(0x2058221E_u32.to_be(), InsType::Unknown)] // fmin s0, s1, s2
    #[case::fminnm(0x2078221E_u32.to_be(), InsType::Unknown)] // fminnm s0, s1, s2
    #[case::fminnmp(0x00C4A12E_u32.to_be(), InsType::Unknown)] // fminnmp v0.2s, v0.2s, v1.2s
    #[case::fminp(0x00F4A12E_u32.to_be(), InsType::Unknown)] // fminp v0.2s, v0.2s, v1.2s
    #[case::fmla(0x00CC210E_u32.to_be(), InsType::Unknown)] // fmla v0.2s, v0.2s, v1.2s
    #[case::fmls(0x00CCA10E_u32.to_be(), InsType::Unknown)] // fmls v0.2s, v0.2s, v1.2s
    #[case::fmov(0x2040201E_u32.to_be(), InsType::Unknown)] // fmov s0, s1
    #[case::fmsub(0x208C021F_u32.to_be(), InsType::Unknown)] // fmsub s0, s1, s2, s3
    #[case::fmul(0x2008221E_u32.to_be(), InsType::Unknown)] // fmul s0, s1, s2
    #[case::fmulx(0x20DC225E_u32.to_be(), InsType::Unknown)] // fmulx s0, s1, s2
    #[case::fneg(0x2040211E_u32.to_be(), InsType::Unknown)] // fneg s0, s1
    #[case::fnmadd(0x200C221F_u32.to_be(), InsType::Unknown)] // fnmadd s0, s1, s2, s3
    #[case::fnmsub(0x208C221F_u32.to_be(), InsType::Unknown)] // fnmsub s0, s1, s2, s3
    #[case::fnmul(0x2088221E_u32.to_be(), InsType::Unknown)] // fnmul s0, s1, s2
    #[case::frecpe(0x20D8A15E_u32.to_be(), InsType::Unknown)] // frecpe s0, s1
    #[case::frecps(0x20FC225E_u32.to_be(), InsType::Unknown)] // frecps s0, s1, s2
    #[case::frecpx(0x20F8A15E_u32.to_be(), InsType::Unknown)] // frecpx s0, s1
    #[case::frinta(0x2040261E_u32.to_be(), InsType::Unknown)] // frinta s0, s1
    #[case::frinti(0x20C0271E_u32.to_be(), InsType::Unknown)] // frinti s0, s1
    #[case::frintm(0x2040251E_u32.to_be(), InsType::Unknown)] // frintm s0, s1
    #[case::frintn(0x2040241E_u32.to_be(), InsType::Unknown)] // frintn s0, s1
    #[case::frintp(0x20C0241E_u32.to_be(), InsType::Unknown)] // frintp s0, s1
    #[case::frintx(0x2040271E_u32.to_be(), InsType::Unknown)] // frintx s0, s1
    #[case::frintz(0x20C0251E_u32.to_be(), InsType::Unknown)] // frintz s0, s1
    #[case::frsqrte(0x20D8A17E_u32.to_be(), InsType::Unknown)] // frsqrte s0, s1
    #[case::frsqrts(0x20FCA25E_u32.to_be(), InsType::Unknown)] // frsqrts s0, s1, s2
    #[case::fsqrt(0x20C0211E_u32.to_be(), InsType::Unknown)] // fsqrt s0, s1
    #[case::fsub(0x2038221E_u32.to_be(), InsType::Unknown)] // fsub s0, s1, s2
    #[case::hint(0x1F2003D5_u32.to_be(), InsType::Unknown)] // hint #0
    #[case::hlt(0x000040D4_u32.to_be(), InsType::Unknown)] // hlt #0
    #[case::hvc(0x020000D4_u32.to_be(), InsType::Unknown)] // hvc #0
    #[case::ins(0x001C014E_u32.to_be(), InsType::Unknown)] // ins v0.b[0], w0
    #[case::isb(0xDF3F03D5_u32.to_be(), InsType::Unknown)] // isb
    #[case::ld1(0x0078404C_u32.to_be(), InsType::Unknown)] // ld1 {v0.4s}, [x0]
    #[case::ld1r(0x00C8404D_u32.to_be(), InsType::Unknown)] // ld1r {v0.4s}, [x0]
    #[case::ld2(0x0088404C_u32.to_be(), InsType::Unknown)] // ld2 {v0.4s, v1.4s}, [x0]
    #[case::ld2r(0x00C8600D_u32.to_be(), InsType::Unknown)] // ld2r {v0.2s, v1.2s}, [x0]
    #[case::ld3(0x0048404C_u32.to_be(), InsType::Unknown)] // ld3 {v0.4s, v1.4s, v2.4s}, [x0]
    #[case::ld3r(0x00E8404D_u32.to_be(), InsType::Unknown)] // ld3r {v0.4s, v1.4s, v2.4s}, [x0]
    #[case::ld4(0x0008404C_u32.to_be(), InsType::Unknown)] // ld4 {v0.4s, v1.4s, v2.4s, v3.4s}, [x0]
    #[case::ld4r(0x00E8604D_u32.to_be(), InsType::Unknown)] // ld4r {v0.4s, v1.4s, v2.4s, v3.4s}, [x0]
    #[case::ldp(0x400440A9_u32.to_be(), InsType::Unknown)] // ldp x0, x1, [x2]
    #[case::ldr(0x200040F9_u32.to_be(), InsType::Unknown)] // ldr x0, [x1]
    #[case::ldrb(0x20004039_u32.to_be(), InsType::Unknown)] // ldrb w0, [x1]
    #[case::ldrh(0x20004079_u32.to_be(), InsType::Unknown)] // ldrh w0, [x1]
    #[case::ldrsb(0x20008039_u32.to_be(), InsType::Unknown)] // ldrsb x0, [x1]
    #[case::ldrsh(0x20008079_u32.to_be(), InsType::Unknown)] // ldrsh x0, [x1]
    #[case::ldrsw(0x200080B9_u32.to_be(), InsType::Unknown)] // ldrsw x0, [x1]
    #[case::ldur(0x208040F8_u32.to_be(), InsType::Unknown)] // ldur x0, [x1, #0x8]
    #[case::ldxp(0x40047FC8_u32.to_be(), InsType::Unknown)] // ldxp x0, x1, [x2]
    #[case::ldxr(0x207C5FC8_u32.to_be(), InsType::Unknown)] // ldxr x0, [x1]
    #[case::lsl(0x00F07DD3_u32.to_be(), InsType::Unknown)] // lsl x0, x0, #3
    #[case::lsr(0x00FC42D3_u32.to_be(), InsType::Unknown)] // lsr x0, x0, #2
    #[case::madd(0x200C029B_u32.to_be(), InsType::Unknown)] // madd x0, x1, x2, x3
    #[case::mla(0x2094A24E_u32.to_be(), InsType::Unknown)] // mla v0.4s, v1.4s, v2.4s
    #[case::mls(0x2094A26E_u32.to_be(), InsType::Unknown)] // mls v0.4s, v1.4s, v2.4s
    #[case::mov(0x804682D2_u32.to_be(), InsType::Unknown)] // mov x0, #0x1234
    #[case::movk(0x00CFAAF2_u32.to_be(), InsType::Unknown)] // movk x0, #0x5678, lsl #16
    #[case::movn(0x80468292_u32.to_be(), InsType::Unknown)] // movn x0, #0x1234
    #[case::movz(0xA079B5D2_u32.to_be(), InsType::Unknown)] // movz x0, #0xabcd, lsl #16
    #[case::mrs(0x00423BD5_u32.to_be(), InsType::Unknown)] // mrs x0, nzcv
    #[case::msr(0xDF4303D5_u32.to_be(), InsType::Unknown)] // msr daifset, #0x3
    #[case::msub(0x208C029B_u32.to_be(), InsType::Unknown)] // msub x0, x1, x2, x3
    #[case::mul(0x209CA24E_u32.to_be(), InsType::Unknown)] // mul v0.4s, v1.4s, v2.4s
    #[case::mvn(0xE00321AA_u32.to_be(), InsType::Unknown)] // mvn x0, x1
    #[case::neg(0xE00301CB_u32.to_be(), InsType::Unknown)] // neg x0, x1
    #[case::negs(0xE00301EB_u32.to_be(), InsType::Unknown)] // negs x0, x1
    #[case::ngc(0xE00301DA_u32.to_be(), InsType::Unknown)] // ngc x0, x1
    #[case::ngcs(0xE00301FA_u32.to_be(), InsType::Unknown)] // ngcs x0, x1
    #[case::nop(0x1F2003D5_u32.to_be(), InsType::Unknown)] // nop
    #[case::orn(0x200022AA_u32.to_be(), InsType::Unknown)] // orn x0, x1, x2
    #[case::orr(0x000074B2_u32.to_be(), InsType::Unknown)] // orr x0, x0, #0x1000
    #[case::prfm(0x000000D8_u32.to_be(), InsType::LdrLiteral)] // prfm pldl1keep, #0 // sub-mode of LDR literal
    #[case::prfm(0x000080F9_u32.to_be(), InsType::Unknown)] // prfm pldl1keep, [x0]
    #[case::prfum(0x000081F8_u32.to_be(), InsType::Unknown)] // prfum pldl1keep, [x0, #0x10]
    #[case::rbit(0x2000C0DA_u32.to_be(), InsType::Unknown)] // rbit x0, x1
    #[case::ret(0xC0035FD6_u32.to_be(), InsType::Unknown)] // ret
    #[case::rev(0x200CC0DA_u32.to_be(), InsType::Unknown)] // rev x0, x1
    #[case::rev16(0x2004C0DA_u32.to_be(), InsType::Unknown)] // rev16 x0, x1
    #[case::rev32(0x2008C0DA_u32.to_be(), InsType::Unknown)] // rev32 x0, x1
    #[case::rev64(0x200CC0DA_u32.to_be(), InsType::Unknown)] // rev64 x0, x1
    #[case::ror(0x200CC193_u32.to_be(), InsType::Unknown)] // ror x0, x1, #3
    #[case::saddl(0x2000220E_u32.to_be(), InsType::Unknown)] // saddl v0.8h, v1.8b, v2.8b
    #[case::sbc(0x200002DA_u32.to_be(), InsType::Unknown)] // sbc x0, x1, x2
    #[case::sbcs(0x200002FA_u32.to_be(), InsType::Unknown)] // sbcs x0, x1, x2
    #[case::sdiv(0x200CC29A_u32.to_be(), InsType::Unknown)] // sdiv x0, x1, x2
    #[case::sev(0x9F2003D5_u32.to_be(), InsType::Unknown)] // sev
    #[case::sevl(0xBF2003D5_u32.to_be(), InsType::Unknown)] // sevl
    #[case::smaddl(0x200C229B_u32.to_be(), InsType::Unknown)] // smaddl x0, w1, w2, x3
    #[case::smc(0x030000D4_u32.to_be(), InsType::Unknown)] // smc #0
    #[case::smin(0x206CA24E_u32.to_be(), InsType::Unknown)] // smin v0.4s, v1.4s, v2.4s
    #[case::smulh(0x207C429B_u32.to_be(), InsType::Unknown)] // smulh x0, x1, x2
    #[case::smull(0x207C229B_u32.to_be(), InsType::Unknown)] // smull x0, w1, w2
    #[case::sshl(0x2044A24E_u32.to_be(), InsType::Unknown)] // sshl v0.4s, v1.4s, v2.4s
    #[case::st1(0x0078004C_u32.to_be(), InsType::Unknown)] // st1 {v0.4s}, [x0]
    #[case::st1(0x0080000D_u32.to_be(), InsType::Unknown)] // st1 {v0.s}[0], [x0]
    #[case::st2(0x0088004C_u32.to_be(), InsType::Unknown)] // st2 {v0.4s, v1.4s}, [x0]
    #[case::st3(0x0048004C_u32.to_be(), InsType::Unknown)] // st3 {v0.4s, v1.4s, v2.4s}, [x0]
    #[case::st4(0x0008004C_u32.to_be(), InsType::Unknown)] // st4 {v0.4s, v1.4s, v2.4s, v3.4s}, [x0]
    #[case::stlr(0x20FC9FC8_u32.to_be(), InsType::Unknown)] // stlr x0, [x1]
    #[case::stlxp(0x618820C8_u32.to_be(), InsType::Unknown)] // stlxp w0, x1, x2, [x3]
    #[case::stlxr(0x41FC00C8_u32.to_be(), InsType::Unknown)] // stlxr w0, x1, [x2]
    #[case::stp(0x400400A9_u32.to_be(), InsType::Unknown)] // stp x0, x1, [x2]
    #[case::str(0x200000F9_u32.to_be(), InsType::Unknown)] // str x0, [x1]
    #[case::strb(0x20000039_u32.to_be(), InsType::Unknown)] // strb w0, [x1]
    #[case::strh(0x20000079_u32.to_be(), InsType::Unknown)] // strh w0, [x1]
    #[case::sttr(0x200800F8_u32.to_be(), InsType::Unknown)] // sttr x0, [x1]
    #[case::stur(0x208000F8_u32.to_be(), InsType::Unknown)] // stur x0, [x1, #0x8]
    #[case::stxp(0x610820C8_u32.to_be(), InsType::Unknown)] // stxp w0, x1, x2, [x3]
    #[case::stxr(0x417C00C8_u32.to_be(), InsType::Unknown)] // stxr w0, x1, [x2]
    #[case::sub(0x000001CB_u32.to_be(), InsType::Unknown)] // sub x0, x0, x1
    #[case::subs(0x000C00F1_u32.to_be(), InsType::Unknown)] // subs x0, x0, #0x3
    #[case::svc(0x010000D4_u32.to_be(), InsType::Unknown)] // svc #0
    #[case::sxtb(0x201C4093_u32.to_be(), InsType::Unknown)] // sxtb x0, w1
    #[case::sxth(0x203C4093_u32.to_be(), InsType::Unknown)] // sxth x0, w1
    #[case::sxtw(0x207C4093_u32.to_be(), InsType::Unknown)] // sxtw x0, w1
    #[case::sys(0x1F7508D5_u32.to_be(), InsType::Unknown)] // sys #0, c7, c5, #0
    #[case::sysl(0x007528D5_u32.to_be(), InsType::Unknown)] // sysl x0, #0, c7, c5, #0
    #[case::tbnz(0xC0E80737_u32.to_be(), InsType::Tbz)] // tbnz x0, #0, #0
    #[case::tbz(0xA0E80736_u32.to_be(), InsType::Tbz)] // tbz x0, #0, #0
    #[case::tlbi(0x1F8708D5_u32.to_be(), InsType::Unknown)] // tlbi vmalle1
    #[case::tst(0x1F1C7CF2_u32.to_be(), InsType::Unknown)] // tst x0, #0x0ff0
    #[case::uaddl(0x2000222E_u32.to_be(), InsType::Unknown)] // uaddl v0.8h, v1.8b, v2.8b
    #[case::ubfiz(0x20147ED3_u32.to_be(), InsType::Unknown)] // ubfiz x0, x1, #2, #6
    #[case::ubfm(0x201842D3_u32.to_be(), InsType::Unknown)] // ubfm x0, x1, #2, #6
    #[case::ubfx(0x201C42D3_u32.to_be(), InsType::Unknown)] // ubfx x0, x1, #2, #6
    #[case::udiv(0x2008C29A_u32.to_be(), InsType::Unknown)] // udiv x0, x1, x2
    #[case::umaddl(0x200CA29B_u32.to_be(), InsType::Unknown)] // umaddl x0, w1, w2, x3
    #[case::umax(0x2064A26E_u32.to_be(), InsType::Unknown)] // umax v0.4s, v1.4s, v2.4s
    #[case::umin(0x206CA26E_u32.to_be(), InsType::Unknown)] // umin v0.4s, v1.4s, v2.4s
    #[case::umulh(0x207CC29B_u32.to_be(), InsType::Unknown)] // umulh x0, x1, x2
    #[case::umull(0x207CA29B_u32.to_be(), InsType::Unknown)] // umull x0, w1, w2
    #[case::uqadd(0x200CA26E_u32.to_be(), InsType::Unknown)] // uqadd v0.4s, v1.4s, v2.4s
    #[case::urecpe(0x20C8A14E_u32.to_be(), InsType::Unknown)] // urecpe v0.4s, v1.4s
    #[case::urhadd(0x2014A26E_u32.to_be(), InsType::Unknown)] // urhadd v0.4s, v1.4s, v2.4s
    #[case::ushl(0x2044A26E_u32.to_be(), InsType::Unknown)] // ushl v0.4s, v1.4s, v2.4s
    #[case::usqadd(0x2038A06E_u32.to_be(), InsType::Unknown)] // usqadd v0.4s, v1.4s
    #[case::uxtb(0x201C0053_u32.to_be(), InsType::Unknown)] // uxtb w0, w1
    #[case::uxth(0x203C0053_u32.to_be(), InsType::Unknown)] // uxth w0, w1
    #[case::uxtw(0x207C40D3_u32.to_be(), InsType::Unknown)] // uxtw x0, w1
    #[case::wfe(0x5F2003D5_u32.to_be(), InsType::Unknown)] // wfe
    #[case::wfi(0x7F2003D5_u32.to_be(), InsType::Unknown)] // wfi
    #[case::yield_uwu(0x3F2003D5_u32.to_be(), InsType::Unknown)] // yield
    fn ensure_valid_instruction_recognized(
        #[case] instruction: u32,
        #[case] expected_instruction: InsType,
    ) {
        assert!(get_ins_type(instruction) == expected_instruction);
    }

    #[rstest]
    #[case("00040014", 8192, 4096, "00080014", Some(17))]
    #[case("00040014", 0x8000000, 0, "110004b020021fd6", Some(17))]
    #[case("00040014", 0x8000512, 0, "110004b0314a149120021fd6", Some(17))]
    #[case(
        "00040014",
        0x100000000,
        0,
        "110082d21100a0f23100c0f220021fd6",
        Some(17)
    )]
    fn test_rewrite_b_cases(
        #[case] old_instruction_hex: &str,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected_hex: &str,
        #[case] scratch_register: Option<u8>,
    ) {
        test_rewrite(
            old_instruction_hex,
            old_address,
            new_address,
            expected_hex,
            scratch_register,
        );
    }

    // Helper function to convert hex string to a byte vector.
    fn hex_str_to_bytes(hex_str: &str) -> Vec<u8> {
        hex::decode(hex_str).expect("Invalid hex string")
    }

    // Test helper to compare the rewritten code with the expected result.
    fn test_rewrite(
        old_instruction_hex: &str,
        old_address: usize,
        new_address: usize,
        expected_hex: &str,
        scratch_register: Option<u8>,
    ) {
        let old_instruction_bytes = hex_str_to_bytes(old_instruction_hex);
        let mut new_code = Vec::new();
        let _result = rewrite_code_aarch64(
            old_instruction_bytes.as_ptr(),
            old_instruction_bytes.len(),
            old_address,
            new_address,
            scratch_register,
            &mut new_code,
        );

        assert_eq!(
            instruction_buffer_as_hex_u8(&new_code),
            expected_hex,
            "Rewritten instruction does not match expected result"
        );
    }
}
