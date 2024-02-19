extern crate alloc;

use super::Register;
use crate::common::{
    rewriter::{
        conditional_branch::patch_conditional_branch_64, helpers::has_single_immediate_operand,
        jcxz::patch_jcxz_64, loope::patch_loope_64, loopne::patch_loopne_64, r#loop::patch_loop_64,
        relative_branch::patch_relative_branch_64,
        rip_relative_instruction::patch_rip_relative_64_or_copyraw,
    },
    util::zydis_decoder_result::ZydisDecoderResult,
};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::{ops::Sub, slice};
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};
use zydis::ffi::DecodedOperandKind::*;
use zydis::{Decoder, MachineMode, Mnemonic::*, StackWidth, VisibleOperands};

pub struct CodeRewriterX64;

impl CodeRewriter<Register> for CodeRewriterX64 {
    unsafe fn rewrite_code_with_buffer(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<Register>,
        existing_buffer: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        // Remove spaces and convert the string to a vector of bytes
        let dec = Decoder::new(MachineMode::LONG_64, StackWidth::_64).unwrap();
        let hex_bytes = unsafe { slice::from_raw_parts(old_code, old_code_size) };

        let orig_pc = old_address;
        let mut pc: usize = old_address;
        let mut dest_address = new_address;

        for ins in dec.decode_all::<VisibleOperands>(hex_bytes, old_address as u64) {
            // Fail fast on any error
            if ins.is_err() {
                return Err(fail_fast(hex_bytes, pc, orig_pc));
            }

            // .0 : ip
            // .1 : insn_bytes
            // .2 : Instruction
            let ins: ZydisDecoderResult<VisibleOperands> = ins.unwrap().into();

            if has_single_immediate_operand(&ins) {
                match ins.instruction.mnemonic {
                    // Branch & Call
                    CALL => {
                        patch_relative_branch_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                            true,
                        )?;
                    }
                    JMP => {
                        patch_relative_branch_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                            false,
                        )?;
                    }
                    // Conditional Branch
                    JO | JNO | JB | JNB | JZ | JNZ | JBE | JNBE | JS | JNS | JP | JNP | JL
                    | JNL | JLE | JNLE => {
                        patch_conditional_branch_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                    LOOP => {
                        patch_loop_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                    LOOPE => {
                        patch_loope_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                    LOOPNE => {
                        patch_loopne_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                    JCXZ | JRCXZ | JECXZ => {
                        patch_jcxz_64(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                    // Everything else
                    _ => {
                        patch_rip_relative_64_or_copyraw(
                            &ins.instruction,
                            ins.instruction_bytes,
                            &mut dest_address,
                            &mut pc,
                            scratch_register,
                            existing_buffer,
                        )?;
                    }
                }
            } else {
                patch_rip_relative_64_or_copyraw(
                    &ins.instruction,
                    ins.instruction_bytes,
                    &mut dest_address,
                    &mut pc,
                    scratch_register,
                    existing_buffer,
                )?;
            }
        }

        Ok(())
    }

    fn max_ins_size_increase() -> usize {
        14 // see: patches::patch_jcx
    }
}

#[cold]
fn fail_fast(hex_bytes: &[u8], pc: usize, orig_pc: usize) -> CodeRewriterError {
    CodeRewriterError::FailedToDisasm(
        "Incorrect or incomplete instruction.".to_owned(),
        hex::encode(&hex_bytes[pc.sub(orig_pc)..]),
    )
}

#[cfg(test)]
mod tests {

    use super::CodeRewriterX64;
    use crate::common::util::test_utilities::test_rewrite_code_with_buffer;
    use crate::x64::Register;
    use rstest::rstest;

    #[rstest]
    // Loop
    #[case::jmp_with_pad("50eb02", 4096, 0, "50e9ff0f0000")] // push + jmp +2 -> push + jmp +4098
    #[case::to_abs_call_i32("e8fb0f0000", 0x80000000, 0, "48b80010008000000000ffd0")] // call +4091 -> mov rax, 0x80001000 + call rax
    #[case::to_absolute_call("e8ffffffff", 0x80000000, 0, "48b80400008000000000ffd0")] // call -1 -> call rax, 0x80000004 + call rax
    #[case::to_absolute_call_i32_upper64(
        "e8fb0f0000",
        0x8000000080000000,
        0x8000000000000000,
        "48b80010008000000080ffd0"
    )] // call +2 -> mov rax, 0x8000000080001000 + call rax
    #[case::simple_jmp("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_abs_jmp_i8("eb02", 0x80000000, 0, "48b80400008000000000ffe0")] // jmp +2 -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_jmp_i32("e9fb0f0000", 0x80000000, 0, "48b80010008000000000ffe0")] // jmp +4091 -> mov rax, 0x80001000 + jmp rax
    // Some tests when in upper bytes
    #[case::simple_branch_upper64("eb02", 0x8000000000001000, 0x8000000000000000, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_abs_jmp_8b_upper64(
        "eb02",
        0x8000000080000000,
        0x8000000000000000,
        "48b80400008000000080ffe0"
    )] // jmp +2 -> mov rax, 0x8000000000000004 + jmp rax
    #[case::to_abs_jmp_i32_upper64(
        "e9fb0f0000",
        0x8000000080000000,
        0x8000000000000000,
        "48b80010008000000080ffe0"
    )] // jmp +2 -> mov rax, 0x8000000080001000 + jmp rax
    #[case::loopne_backward("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne 0x7fffffc -> loopne 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loopne_backward_i32("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne -3 -> loopne 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loopne_backward_abs_upper64(
        "e0fa",
        0x8000000080001000,
        0x8000000000000000,
        "e002eb0c48b8fc0f008000000080ffe0"
    )]
    // loopne 0x8000000080000ffc -> loopne 0x8000000080001004 + jmp 0x8000000080001010 + movabs rax, 0x8000000080000ffc + jmp rax
    #[case::loope_backward("e1fa", 0x8000000, 0, "e102eb05e9f3ffff07")] // loope 0x7fffffc -> loope 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loope_backward_i32("e1fa", 0x8000000, 0, "e102eb05e9f3ffff07")] // loope -3 -> loope 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loope_backward_abs_upper64(
        "e1fa",
        0x8000000080001000,
        0x8000000000000000,
        "e102eb0c48b8fc0f008000000080ffe0"
    )]
    // loope 0x8000000080000ffc -> loope 0x8000000080001004 + jmp 0x8000000080001010 + movabs rax, 0x8000000080000ffc + jmp rax
    #[case::loop_backward_i8("e2fb", 4096, 0, "490f85f60f0000")] // loop 0xffd -> dec ecx + jnz 0xffd
    #[case::loop_backward_abs_upper32("e2fb", 0x80001000, 0x80000000, "490f85f60f0000")] // loop 0x80000ffd -> dec ecx + jnz 0x80000ffd
    #[case::loop_backward_abs("e2fa", 0x80001000, 0, "e202eb0c48b8fc0f008000000000ffe0")] // loop -3 -> loop +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loop_backward_abs_upper64(
        "e2fa",
        0x8000000080001000,
        0x8000000000000000,
        "e202eb0c48b8fc0f008000000080ffe0"
    )] // loop -3 -> loop +2 + jmp 0x11 + movabs rax, 0x8000000080000ffd + jmp rax
    #[case::jrcxz_i32("e3fa", 0x8000000, 0, "85c90f85f4ffff07")] // jecxz 0x7fffffc -> test ecx, ecx + jne 0x7fffffc
    #[case::jrcxz_abs("e3fa", 0x80001000, 0, "e302eb0c48b8fc0f008000000000ffe0")] // jecxz 0x80000ffc -> jrcxz 4 + jmp 0x10 + movabs rax, 0x80000ffd + jmp rax
    #[case::jo("7002", 4096, 0, "0f80fe0f0000")] // jo 4100
    #[case::jo_with_underflow("70f4", 0, 4096, "0f80f0efffff")] // jo 0xfffffff6
    #[case::jo_upper64("7002", 0x8000000000001000, 0x8000000000000000, "0f80fe0f0000")] // jo 0x80001004
    #[case::jo_abs_64b("7002", 0x8000000000000000, 0, "710a48b80400000000000080ffe0")]
    // jno 0x8000000000000004 -> jo +12 <skip> + mov rax, 0x8000000000000004 + jmp rax
    #[case::jb("7202", 4096, 0, "0f82fe0f0000")] // jb 4100
    #[case::jb_with_underflow("72f4", 0, 4096, "0f82f0efffff")] // jb 0xfffffff6
    #[case::jb_upper64("7202", 0x8000000000001000, 0x8000000000000000, "0f82fe0f0000")] // jb 0x80001004
    #[case::jo_abs_64b("7202", 0x8000000000000000, 0, "730a48b80400000000000080ffe0")]
    // jbe 0x8000000000000004 -> jne +12 <skip> + mov rax, 0x8000000000000004 + jmp rax
    #[case::jz("7402", 4096, 0, "0f84fe0f0000")] // jz 4100
    #[case::jz_with_underflow("74f4", 0, 4096, "0f84f0efffff")] // jz 0xfffffff6
    #[case::jz_upper64("7402", 0x8000000000001000, 0x8000000000000000, "0f84fe0f0000")] // jz 0x80001004
    #[case::jbe("7602", 4096, 0, "0f86fe0f0000")] // jbe 4100
    #[case::jbe_with_underflow("76f4", 0, 4096, "0f86f0efffff")] // jbe 0xfffffff6
    #[case::jbe_upper64("7602", 0x8000000000001000, 0x8000000000000000, "0f86fe0f0000")] // jbe 0x80001004
    #[case::js("7802", 4096, 0, "0f88fe0f0000")] // js 4100
    #[case::js_with_underflow("78f4", 0, 4096, "0f88f0efffff")] // js 0xfffffff6
    #[case::js_upper64("7802", 0x8000000000001000, 0x8000000000000000, "0f88fe0f0000")] // js 0x80001004
    #[case::jp("7a02", 4096, 0, "0f8afe0f0000")] // jp 4100
    #[case::jp_with_underflow("7af4", 0, 4096, "0f8af0efffff")] // jp 0xfffffff6
    #[case::jp_upper64("7a02", 0x8000000000001000, 0x8000000000000000, "0f8afe0f0000")] // jp 0x80001004
    #[case::jl("7c02", 4096, 0, "0f8cfe0f0000")] // jl 4100
    #[case::jl_with_underflow("7cf4", 0, 4096, "0f8cf0efffff")] // jl 0xfffffff6
    #[case::jl_upper64("7c02", 0x8000000000001000, 0x8000000000000000, "0f8cfe0f0000")] // jl 0x80001004
    #[case::jle("7e02", 4096, 0, "0f8efe0f0000")] // jle 4100
    #[case::jle_with_underflow("7ef4", 0, 4096, "0f8ef0efffff")] // jle 0xfffffff6
    #[case::jle_upper64("7e02", 0x8000000000001000, 0x8000000000000000, "0f8efe0f0000")] // jle 0x80001004
    #[case::jno("7102", 4096, 0, "0f81fe0f0000")] // jno 4100
    #[case::jno_with_underflow("71f4", 0, 4096, "0f81f0efffff")] // jno 0xfffffff6
    #[case::jno_upper64("7102", 0x8000000000001000, 0x8000000000000000, "0f81fe0f0000")] // jno 0x80001004
    #[case::jnb("7302", 4096, 0, "0f83fe0f0000")] // jnb 4100
    #[case::jnb_with_underflow("73f4", 0, 4096, "0f83f0efffff")] // jnb 0xfffffff6
    #[case::jnb_upper64("7302", 0x8000000000001000, 0x8000000000000000, "0f83fe0f0000")] // jnb 0x80001004
    #[case::jnz("7502", 4096, 0, "0f85fe0f0000")] // jnz 4100
    #[case::jnz_with_underflow("75f4", 0, 4096, "0f85f0efffff")] // jnz 0xfffffff6
    #[case::jnz_upper64("7502", 0x8000000000001000, 0x8000000000000000, "0f85fe0f0000")] // jnz 0x80001004
    #[case::jnbe("7702", 4096, 0, "0f87fe0f0000")] // jnbe 4100
    #[case::jnbe_with_underflow("77f4", 0, 4096, "0f87f0efffff")] // jnbe 0xfffffff6
    #[case::jnbe_upper64("7702", 0x8000000000001000, 0x8000000000000000, "0f87fe0f0000")] // jnbe 0x80001004
    #[case::jns("7902", 4096, 0, "0f89fe0f0000")] // jns 4100
    #[case::jns_with_underflow("79f4", 0, 4096, "0f89f0efffff")] // jns 0xfffffff6
    #[case::jns_upper64("7902", 0x8000000000001000, 0x8000000000000000, "0f89fe0f0000")] // jns 0x80001004
    #[case::jnp("7b02", 4096, 0, "0f8bfe0f0000")] // jnp 4100
    #[case::jnp_with_underflow("7bf4", 0, 4096, "0f8bf0efffff")] // jnp 0xfffffff6
    #[case::jnp_upper64("7b02", 0x8000000000001000, 0x8000000000000000, "0f8bfe0f0000")] // jnp 0x80001004
    #[case::jnl("7d02", 4096, 0, "0f8dfe0f0000")] // jnl 4100
    #[case::jnl_with_underflow("7df4", 0, 4096, "0f8df0efffff")] // jnl 0xfffffff6
    #[case::jnl_upper64("7d02", 0x8000000000001000, 0x8000000000000000, "0f8dfe0f0000")] // jnl 0x80001004
    #[case::jnle("7f02", 4096, 0, "0f8ffe0f0000")] // jnle 4100
    #[case::jnle_with_underflow("7ff4", 0, 4096, "0f8ff0efffff")] // jnle 0xfffffff6
    #[case::jnle_upper64("7f02", 0x8000000000001000, 0x8000000000000000, "0f8ffe0f0000")] // jnle 0x80001004
    #[case::jo_long("0f80faf0ffff", 4096, 0, "0f80fa000000")] // jo 0x100
    #[case::jo_with_underflow_long("0f80f0efffff", 4096, 0, "0f80f0ffffff")] // jo 0xfffffff6
    #[case::jz_long("0f84faf0ffff", 4096, 0, "0f84fa000000")] // jz 0x100
    #[case::jz_with_underflow_long("0f84f0efffff", 4096, 0, "0f84f0ffffff")] // jz 0xfffffff6
    #[case::jbe_long("0f86faf0ffff", 4096, 0, "0f86fa000000")] // jbe 0x100
    #[case::jbe_with_underflow_long("0f86f0efffff", 4096, 0, "0f86f0ffffff")] // jbe 0xfffffff6
    #[case::js_long("0f88faf0ffff", 4096, 0, "0f88fa000000")] // js 0x100
    #[case::js_with_underflow_long("0f88f0efffff", 4096, 0, "0f88f0ffffff")] // js 0xfffffff6
    #[case::jp_long("0f8afaf0ffff", 4096, 0, "0f8afa000000")] // jp 0x100
    #[case::jp_with_underflow_long("0f8af0efffff", 4096, 0, "0f8af0ffffff")] // jp 0xfffffff6
    #[case::jl_long("0f8cfaf0ffff", 4096, 0, "0f8cfa000000")] // jl 0x100
    #[case::jl_with_underflow_long("0f8cf0efffff", 4096, 0, "0f8cf0ffffff")] // jl 0xfffffff6
    #[case::jle_long("0f8efaf0ffff", 4096, 0, "0f8efa000000")] // jle 0x100
    #[case::jle_with_underflow_long("0f8ef0efffff", 4096, 0, "0f8ef0ffffff")] // jle 0xfffffff6
    #[case::jno_long("0f81faf0ffff", 4096, 0, "0f81fa000000")] // jno 0x100
    #[case::jno_with_underflow_long("0f81f0efffff", 4096, 0, "0f81f0ffffff")] // jno 0xfffffff6
    #[case::jnb_long("0f83faf0ffff", 4096, 0, "0f83fa000000")] // jnb 0x100
    #[case::jnb_with_underflow_long("0f83f0efffff", 4096, 0, "0f83f0ffffff")] // jnb 0xfffffff6
    #[case::jnz_long("0f85faf0ffff", 4096, 0, "0f85fa000000")] // jnz 0x100
    #[case::jnz_with_underflow_long("0f85f0efffff", 4096, 0, "0f85f0ffffff")] // jnz 0xfffffff6
    #[case::jnbe_long("0f87faf0ffff", 4096, 0, "0f87fa000000")] // jnbe 0x100
    #[case::jnbe_with_underflow_long("0f87f0efffff", 4096, 0, "0f87f0ffffff")] // jnbe 0xfffffff6
    #[case::jns_long("0f89faf0ffff", 4096, 0, "0f89fa000000")] // jns 0x100
    #[case::jns_with_underflow_long("0f89f0efffff", 4096, 0, "0f89f0ffffff")] // jns 0xfffffff6
    #[case::jnp_long("0f8bfaf0ffff", 4096, 0, "0f8bfa000000")] // jnp 0x100
    #[case::jnp_with_underflow_long("0f8bf0efffff", 4096, 0, "0f8bf0ffffff")] // jnp 0xfffffff6
    #[case::jnl_long("0f8dfaf0ffff", 4096, 0, "0f8dfa000000")] // jnl 0x100
    #[case::jnl_with_underflow_long("0f8df0efffff", 4096, 0, "0f8df0ffffff")] // jnl 0xfffffff6
    fn rewrite_code(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        test_rewrite_code_with_buffer::<CodeRewriterX64, Register>(
            instructions,
            old_address,
            new_address,
            expected,
            Some(Register::rax),
        );
    }
}
