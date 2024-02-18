extern crate alloc;

use super::Register;
use crate::common::{
    rewriter::{
        conditional_branch::patch_conditional_branch_32, jcxz::patch_jcxz_32,
        loope::patch_loope_32, loopne::patch_loopne_32, r#loop::patch_loop_32,
        relative_branch::patch_relative_branch_32,
    },
    util::zydis_decoder_result::ZydisDecoderResult,
};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::{ops::Sub, slice};
use reloaded_hooks_portable::api::rewriter::code_rewriter::{CodeRewriter, CodeRewriterError};
use zydis::{Decoder, MachineMode, Mnemonic::*, StackWidth, VisibleOperands};

pub struct CodeRewriterX86;

impl CodeRewriter<Register> for CodeRewriterX86 {
    unsafe fn rewrite_code_with_buffer(
        old_code: *const u8,
        old_code_size: usize,
        old_address: usize,
        new_address: usize,
        scratch_register: Option<Register>,
        existing_buffer: &mut Vec<u8>,
    ) -> Result<(), CodeRewriterError> {
        // Remove spaces and convert the string to a vector of bytes
        let dec = Decoder::new(MachineMode::LONG_COMPAT_32, StackWidth::_32).unwrap();
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
            match ins.instruction.mnemonic {
                // Branch & Call
                CALL => {
                    patch_relative_branch_32(
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
                    patch_relative_branch_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                        false,
                    )?;
                }
                JO | JNO | JB | JNB | JZ | JNZ | JBE | JNBE | JS | JNS | JP | JNP | JL | JNL
                | JLE | JNLE => {
                    patch_conditional_branch_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                    )?;
                }
                LOOP => {
                    patch_loop_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                    )?;
                }
                LOOPE => {
                    patch_loope_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                    )?;
                }
                LOOPNE => {
                    patch_loopne_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                    )?;
                }
                JCXZ | JRCXZ | JECXZ => {
                    patch_jcxz_32(
                        &ins.instruction,
                        ins.instruction_bytes,
                        &mut dest_address,
                        &mut pc,
                        scratch_register,
                        existing_buffer,
                    )?;
                }
                // Conditional Branch
                _ => {
                    pc += ins.instruction_bytes.len();
                    dest_address += ins.instruction_bytes.len();
                    existing_buffer.extend_from_slice(ins.instruction_bytes);
                }
            }
        }

        Ok(())
    }

    fn max_ins_size_increase() -> usize {
        4 // jmp imm8 to jmp dword [ptr]
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

    use crate::common::util::test_utilities::test_rewrite_code_with_buffer;
    use crate::x86::Register;
    use rstest::rstest;

    use super::CodeRewriterX86;

    #[rstest]
    // Loop
    #[case::jmp_with_pad("50eb02", 4096, 0, "50e9ff0f0000")] // push + jmp +2 -> push + jmp +4098
    #[case::simple_jmp("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::simple_jmp_with_underflow("ebf4", 0, 4096, "e9f1efffff")] // jmp -12 -> jmp -4098
    #[case::simple_jmp_upper32("eb02", 0x80001000, 0x80000000, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::simple_call("e8ffffffff", 4096, 0, "e8ff0f0000")] // call +2 -> call +4098
    #[case::simple_call_upper32("e8ffffffff", 0x80001000, 0x80000000, "e8ff0f0000")] // call +2 -> call +4098
    #[case::loopne_backward("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne 0x7fffffc -> loopne 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loopne_backward_i32("e0fa", 0x8000000, 0, "e002eb05e9f3ffff07")] // loopne -3 -> loopne 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loope_backward("e1fa", 0x8000000, 0, "e102eb05e9f3ffff07")] // loope 0x7fffffc -> loope 0x8000004 + jmp 0x8000009 + jmp 0xffffffc
    #[case::loope_backward_i32("e1fa", 0x8000000, 0, "e102eb05e9f3ffff07")] // loope -3 -> loope 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loop_backward_i8("e2fb", 4096, 0, "490f85f60f0000")] // loop 0xffd -> dec ecx + jnz 0xffd
    #[case::loop_backward_abs_upper32("e2fb", 0x80001000, 0x80000000, "490f85f60f0000")] // loop 0x80000ffd -> dec ecx + jnz 0x80000ffd
    #[case::jrcxz_i32("e3fa", 0x8000000, 0, "85c90f85f4ffff07")] // jecxz 0x7fffffc -> test ecx, ecx + jne 0x7fffffc
    #[case::jo("7002", 4096, 0, "0f80fe0f0000")] // jo 4100
    #[case::jo_with_underflow("70f4", 0, 4096, "0f80f0efffff")] // jo 0xfffffff6
    #[case::jo_upper32("7002", 0x80001000, 0x80000000, "0f80fe0f0000")] // jo 0x80001004
    #[case::jb("7202", 4096, 0, "0f82fe0f0000")] // jb 4100
    #[case::jb_with_underflow("72f4", 0, 4096, "0f82f0efffff")] // jb 0xfffffff6
    #[case::jb_upper32("7202", 0x80001000, 0x80000000, "0f82fe0f0000")] // jb 0x80001004
    #[case::jz("7402", 4096, 0, "0f84fe0f0000")] // jz 4100
    #[case::jz_with_underflow("74f4", 0, 4096, "0f84f0efffff")] // jz 0xfffffff6
    #[case::jz_upper32("7402", 0x80001000, 0x80000000, "0f84fe0f0000")] // jz 0x80001004
    #[case::jbe("7602", 4096, 0, "0f86fe0f0000")] // jbe 4100
    #[case::jbe_with_underflow("76f4", 0, 4096, "0f86f0efffff")] // jbe 0xfffffff6
    #[case::jbe_upper32("7602", 0x80001000, 0x80000000, "0f86fe0f0000")] // jbe 0x80001004
    #[case::js("7802", 4096, 0, "0f88fe0f0000")] // js 4100
    #[case::js_with_underflow("78f4", 0, 4096, "0f88f0efffff")] // js 0xfffffff6
    #[case::js_upper32("7802", 0x80001000, 0x80000000, "0f88fe0f0000")] // js 0x80001004
    #[case::jp("7a02", 4096, 0, "0f8afe0f0000")] // jp 4100
    #[case::jp_with_underflow("7af4", 0, 4096, "0f8af0efffff")] // jp 0xfffffff6
    #[case::jp_upper32("7a02", 0x80001000, 0x80000000, "0f8afe0f0000")] // jp 0x80001004
    #[case::jl("7c02", 4096, 0, "0f8cfe0f0000")] // jl 4100
    #[case::jl_with_underflow("7cf4", 0, 4096, "0f8cf0efffff")] // jl 0xfffffff6
    #[case::jl_upper32("7c02", 0x80001000, 0x80000000, "0f8cfe0f0000")] // jl 0x80001004
    #[case::jle("7e02", 4096, 0, "0f8efe0f0000")] // jle 4100
    #[case::jle_with_underflow("7ef4", 0, 4096, "0f8ef0efffff")] // jle 0xfffffff6
    #[case::jle_upper32("7e02", 0x80001000, 0x80000000, "0f8efe0f0000")] // jle 0x80001004
    #[case::jno("7102", 4096, 0, "0f81fe0f0000")] // jno 4100
    #[case::jno_with_underflow("71f4", 0, 4096, "0f81f0efffff")] // jno 0xfffffff6
    #[case::jno_upper32("7102", 0x80001000, 0x80000000, "0f81fe0f0000")] // jno 0x80001004
    #[case::jnb("7302", 4096, 0, "0f83fe0f0000")] // jnb 4100
    #[case::jnb_with_underflow("73f4", 0, 4096, "0f83f0efffff")] // jnb 0xfffffff6
    #[case::jnb_upper32("7302", 0x80001000, 0x80000000, "0f83fe0f0000")] // jnb 0x80001004
    #[case::jnz("7502", 4096, 0, "0f85fe0f0000")] // jnz 4100
    #[case::jnz_with_underflow("75f4", 0, 4096, "0f85f0efffff")] // jnz 0xfffffff6
    #[case::jnz_upper32("7502", 0x80001000, 0x80000000, "0f85fe0f0000")] // jnz 0x80001004
    #[case::jnbe("7702", 4096, 0, "0f87fe0f0000")] // jnbe 4100
    #[case::jnbe_with_underflow("77f4", 0, 4096, "0f87f0efffff")] // jnbe 0xfffffff6
    #[case::jnbe_upper32("7702", 0x80001000, 0x80000000, "0f87fe0f0000")] // jnbe 0x80001004
    #[case::jns("7902", 4096, 0, "0f89fe0f0000")] // jns 4100
    #[case::jns_with_underflow("79f4", 0, 4096, "0f89f0efffff")] // jns 0xfffffff6
    #[case::jns_upper32("7902", 0x80001000, 0x80000000, "0f89fe0f0000")] // jns 0x80001004
    #[case::jnp("7b02", 4096, 0, "0f8bfe0f0000")] // jnp 4100
    #[case::jnp_with_underflow("7bf4", 0, 4096, "0f8bf0efffff")] // jnp 0xfffffff6
    #[case::jnp_upper32("7b02", 0x80001000, 0x80000000, "0f8bfe0f0000")] // jnp 0x80001004
    #[case::jnl("7d02", 4096, 0, "0f8dfe0f0000")] // jnl 4100
    #[case::jnl_with_underflow("7df4", 0, 4096, "0f8df0efffff")] // jnl 0xfffffff6
    #[case::jnl_upper32("7d02", 0x80001000, 0x80000000, "0f8dfe0f0000")] // jnl 0x80001004
    #[case::jnle("7f02", 4096, 0, "0f8ffe0f0000")] // jnle 4100
    #[case::jnle_with_underflow("7ff4", 0, 4096, "0f8ff0efffff")] // jnle 0xfffffff6
    #[case::jnle_upper32("7f02", 0x80001000, 0x80000000, "0f8ffe0f0000")] // jnle 0x80001004
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
        test_rewrite_code_with_buffer::<CodeRewriterX86, Register>(
            instructions,
            old_address,
            new_address,
            expected,
            Some(Register::eax),
        );
    }
}
