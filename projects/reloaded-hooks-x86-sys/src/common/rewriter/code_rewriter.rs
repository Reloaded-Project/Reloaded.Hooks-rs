extern crate alloc;

use crate::all_registers::AllRegisters;
use alloc::vec::Vec;
use iced_x86::Instruction;
use iced_x86::{BlockEncoder, BlockEncoderOptions, InstructionBlock};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::{smallvec, SmallVec};

// Patches only needed for 64-bit.
#[cfg(feature = "x64")]
use super::patches::{
    patch_jcx, patch_jump_conditional, patch_loop, patch_relative_branch,
    patch_rip_relative_operand,
};

/// Relocates the code to a new location.
///
/// # Parameters
/// `is_64bit`: Whether the code is 64bit or not.
/// `instructions`: The instructions to relocate.
/// `new_pc`: The new program counter (RIP/EIP).
/// `scratch_gpr`: A scratch general purpose register that can be used for operations.
/// `buf`: The buffer to write the relocated code to.
///
/// # Safety (For >= 2GiB relocations)
///
/// By contract, `scratch_gpr` and `scratch_xmm` must be caller saved
/// registers which are NOT used at all in the function prologue.
/// (Neither as source, or destination.)
///
/// As eventually all registers would wind up being used, this effectively means that this code
/// can only be used for rewriting function prologues.
#[allow(dead_code)]
pub(crate) fn relocate_code(
    is_64bit: bool,
    instructions: &SmallVec<[Instruction; 4]>,
    new_pc: usize,
    scratch_gpr: Option<AllRegisters>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    let mut new_isns: SmallVec<[Instruction; 4]> = smallvec![];
    let mut current_new_pc = new_pc;

    // This code will be eliminated in a x86/x64 only build, because only 1 call will be made here
    // and the compiler will eliminate out the constant branch.
    #[cfg(feature = "x64")]
    if is_64bit {
        // Note: These translations can only happen in x64, because in x86, branches will always be reachable.
        // Otherwise we need to translate the jmp/call to an absolute address.

        // Note: It's techincally possible the original code moves the value to a register, then moves
        // or uses the value from that register in very next instruction, making our rewriting unstable.

        for instruction in instructions {
            // Note: Check docs for `UnconditionalBranch` and `Call` above for instructions accepted into this branch.
            // If this is not a near call or jump, copy the instruction straight up.
            let is_call_near = instruction.is_call_near();
            let is_jmp_near = instruction.is_jmp_short_or_near();
            if is_call_near || is_jmp_near {
                patch_relative_branch(
                    scratch_gpr,
                    &mut new_isns,
                    &mut current_new_pc,
                    instruction,
                    is_call_near,
                )?;
                continue;
            }

            // Conditional Branch
            if instruction.is_jcc_short_or_near() {
                patch_jump_conditional(
                    scratch_gpr,
                    &mut new_isns,
                    &mut current_new_pc,
                    instruction,
                )?;

                continue;
            } else if instruction.is_loopcc() || instruction.is_loop() {
                patch_loop(scratch_gpr, &mut new_isns, &mut current_new_pc, instruction)?;
                continue;
            } else if instruction.is_jcx_short() {
                patch_jcx(scratch_gpr, &mut new_isns, &mut current_new_pc, instruction)?;
                continue;
            } else if instruction.memory_base() == iced_x86::Register::RIP {
                patch_rip_relative_operand(
                    scratch_gpr,
                    &mut new_isns,
                    &mut current_new_pc,
                    instruction,
                )?;
                continue;
            }

            // Everything else is unhandled
            append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
        }
    }

    #[cfg(feature = "x86")]
    if !is_64bit {
        for instruction in instructions {
            append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
        }
    }

    let block = InstructionBlock::new(&new_isns, new_pc as u64);
    let result = match BlockEncoder::encode(
        if is_64bit & cfg!(feature = "x64") {
            64
        } else if cfg!(feature = "x86") {
            32
        } else {
            0
        },
        block,
        BlockEncoderOptions::NONE,
    ) {
        Err(err) => panic!("{}", err),
        Ok(result) => result,
    };

    let new_code = result.code_buffer;
    buf.extend(new_code);
    Ok(())
}

pub(crate) fn append_if_can_encode_relative(
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> bool {
    // If the branch offset is within 2GiB, do no action
    // because Iced will handle it for us on re-encode.
    let target = instruction.near_branch_target();
    let delta = (target - *current_new_pc as u64) as i64;
    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        append_instruction_with_new_pc(new_isns, current_new_pc, instruction);
        return true;
    }

    false
}

pub(crate) fn can_encode_relative(current_new_pc: &mut usize, instruction: &Instruction) -> bool {
    // If the branch offset is within 2GiB, do no action
    // because Iced will handle it for us on re-encode.
    let target = instruction.near_branch_target();
    let delta = (target - *current_new_pc as u64) as i64;
    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        return true;
    }

    false
}

pub(crate) fn append_instruction_with_new_pc(
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) {
    let mut new_ins = *instruction;
    new_ins.set_ip(*current_new_pc as u64);
    new_isns.push(new_ins);
    *current_new_pc += new_ins.len();
}

#[cfg(test)]
mod tests {
    use crate::all_registers::AllRegisters;
    use crate::common::rewriter::code_rewriter::relocate_code;
    use crate::common::util::get_stolen_instructions::get_stolen_instructions;
    use crate::common::util::test_utilities::str_to_vec;
    use rstest::rstest;

    #[rstest]
    #[cfg(target_pointer_width = "64")]
    #[case::rip_relative_2gib("488b0508000000", 0x7FFFFFF7, 0, "488b05ffffff7f")] // mov rax, qword ptr [rip + 8] -> mov rax, qword ptr [rip + 0x7fffffff]
    #[case::simple_branch_pad("50eb02", 4096, 0, "50e9ff0f0000")] // push + jmp +2 -> push + jmp +4098
    #[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_abs_jmp_i8("eb02", 0x80000000, 0, "48b80400008000000000ffe0")] // jmp +2 -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_jmp_i32("e9fb0f0000", 0x80000000, 0, "48b80010008000000000ffe0")] // jmp +4091 -> mov rax, 0x80001000 + jmp rax
    #[case::to_absolute_call("e8ffffffff", 0x80000000, 0, "48b80400008000000000ffd0")] // call -1 -> call rax, 0x80000004 + call rax
    #[case::jo("7002", 0x80000000, 0, "710c48b80400008000000000ffe0")] // jo +2 -> jno +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jo_pad("507002", 0x80000000, 0, "50710c48b80500008000000000ffe0")] // push + jo +2 -> push + jno +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jb("7202", 0x80000000, 0, "730c48b80400008000000000ffe0")] // jb +2 -> jnb +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jz("7402", 0x80000000, 0, "750c48b80400008000000000ffe0")] // jz +2 -> jnz +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jbe("7602", 0x80000000, 0, "770c48b80400008000000000ffe0")] // jbe +2 -> jnbe +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::js("7802", 0x80000000, 0, "790c48b80400008000000000ffe0")] // js +2 -> jns +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jp("7a02", 0x80000000, 0, "7b0c48b80400008000000000ffe0")] // jp +2 -> jnp +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jl("7c02", 0x80000000, 0, "7d0c48b80400008000000000ffe0")] // jl +2 -> jnl +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jle("7e02", 0x80000000, 0, "7f0c48b80400008000000000ffe0")] // jle +2 -> jnle +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jno("7102", 0x80000000, 0, "700c48b80400008000000000ffe0")] // jno +2 -> jo +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnb("7302", 0x80000000, 0, "720c48b80400008000000000ffe0")] // jnb +2 -> jb +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnz("7502", 0x80000000, 0, "740c48b80400008000000000ffe0")] // jnz +2 -> jz +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnbe("7702", 0x80000000, 0, "760c48b80400008000000000ffe0")] // jnbe +2 -> jbe +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jns("7902", 0x80000000, 0, "780c48b80400008000000000ffe0")] // jns +2 -> js +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnp("7b02", 0x80000000, 0, "7a0c48b80400008000000000ffe0")] // jnp +2 -> jp +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnl("7d02", 0x80000000, 0, "7c0c48b80400008000000000ffe0")] // jnl +2 -> jl +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jnle("7f02", 0x80000000, 0, "7e0c48b80400008000000000ffe0")] // jnle +2 -> jle +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::jo_i32("0f80fa0f0000", 0x80000000, 0, "710c48b80010008000000000ffe0")] // jo 4096 -> jno +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jb_i32("0f82fa0f0000", 0x80000000, 0, "730c48b80010008000000000ffe0")] // jb 4096 -> jnb +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jz_i32("0f84fa0f0000", 0x80000000, 0, "750c48b80010008000000000ffe0")] // jz 4096 -> jnz +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jbe_i32("0f86fa0f0000", 0x80000000, 0, "770c48b80010008000000000ffe0")] // jbe 4096 -> jnbe +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::js_i32("0f88fa0f0000", 0x80000000, 0, "790c48b80010008000000000ffe0")] // js 4096 -> jns +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jp_i32("0f8afa0f0000", 0x80000000, 0, "7b0c48b80010008000000000ffe0")] // jp 4096 -> jnp +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jl_i32("0f8cfa0f0000", 0x80000000, 0, "7d0c48b80010008000000000ffe0")] // jl 4096 -> jnl +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jle_i32("0f8efa0f0000", 0x80000000, 0, "7f0c48b80010008000000000ffe0")] // jle 4096 -> jnle +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jno_i32("0f81fa0f0000", 0x80000000, 0, "700c48b80010008000000000ffe0")] // jno 4096 -> jo +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnb_i32("0f83fa0f0000", 0x80000000, 0, "720c48b80010008000000000ffe0")] // jnb 4096 -> jb +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnz_i32("0f85fa0f0000", 0x80000000, 0, "740c48b80010008000000000ffe0")] // jnz 4096 -> jz +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnbe_i32("0f87fa0f0000", 0x80000000, 0, "760c48b80010008000000000ffe0")] // jnbe 4096 -> jbe +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jns_i32("0f89fa0f0000", 0x80000000, 0, "780c48b80010008000000000ffe0")] // jns 4096 -> js +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnp_i32("0f8bfa0f0000", 0x80000000, 0, "7a0c48b80010008000000000ffe0")] // jnp 4096 -> jp +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnl_i32("0f8dfa0f0000", 0x80000000, 0, "7c0c48b80010008000000000ffe0")] // jnl 4096 -> jl +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::jnle_i32("0f8ffa0f0000", 0x80000000, 0, "7e0c48b80010008000000000ffe0")] // jnle 4096 -> jle +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::loop_backward_abs("50e2fa", 0x80001000, 0, "50e202eb0c48b8fd0f008000000000ffe0")] // push rax + loop -3 -> push rax + loop +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loope_backward_abs("50e1fa", 0x80001000, 0, "50e102eb0c48b8fd0f008000000000ffe0")] // push rax + loope -3 -> push rax + loope +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loopne_backward_abs("50e0fa", 0x80001000, 0, "50e002eb0c48b8fd0f008000000000ffe0")] // push rax + loopne -3 -> push rax + loopne +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loop_backward_i8("50e2fa", 4096, 0, "5048ffc90f85f30f0000")] // push rax + loop -3 -> push rax + dec ecx + jnz 0xffd
    #[case::loop_backward_i32("50e2fa", 0x8000000, 0, "5048ffc90f85f3ffff07")] // push rax + loop -3 -> push rax + dec ecx + jnz 0x7fffffd
    #[case::loope_backward_i8("50e1fa", 4096, 0, "50e102eb05e9f30f0000")] // push rax + loope -3 -> push rax + loope 5 + jmp 0xa + jmp 0xffd
    #[case::loope_backward_i32("50e1fa", 0x8000000, 0, "50e102eb05e9f3ffff07")] // push rax + loope -3 -> push rax + loope 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loopne_backward_i8("50e0fa", 4096, 0, "50e002eb05e9f30f0000")] // push rax + loopne -3 -> push rax + loopne 5 + jmp 0xa + jmp 0xffd
    #[case::loopne_backward_i32("50e0fa", 0x8000000, 0, "50e002eb05e9f3ffff07")] // push rax + loopne -3 -> push rax + loopne 5 + jmp 0xa + jmp 0x7fffffd
    #[case::jrcxz_i32("50e3fa", 0x8000000, 0, "5085c90f85f4ffff07")] // push rax + jrcxz -3 -> push rax + test ecx, rcx + jne 0x7fffffd
    #[case::jrcxz_abs("50e3fa", 0x80001000, 0, "50e302eb0c48b8fd0f008000000000ffe0")]
    // push rax + jrcxz -3 -> push rax + jrcxz 5 + jmp 0x11 + mov rax, 0x80000ffd + jmp rax
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
    #[case::to_absolute_call_i32_upper64(
        "e8fb0f0000",
        0x8000000080000000,
        0x8000000000000000,
        "48b80010008000000080ffd0"
    )] // call +2 -> mov rax, 0x8000000080001000 + call rax
    #[case::loop_backward_abs_upper64(
        "50e2fa",
        0x8000000080001000,
        0x8000000000000000,
        "50e202eb0c48b8fd0f008000000080ffe0"
    )] // push rax + loop -3 -> push rax + loop +2 + jmp 0x11 + movabs rax, 0x8000000080000ffd + jmp rax
    #[case::jo_upper64(
        "7002",
        0x8000000080000000,
        0x8000000000000000,
        "710c48b80400008000000080ffe0"
    )] // jo +2 -> jno +12 <skip> + mov rax, 0x8000000080000004 + jmp rax
    #[case::loope_backward_i8_upper64(
        "50e1fa",
        0x8000000000001000,
        0x8000000000000000,
        "50e102eb05e9f30f0000"
    )] // push rax + loope -3 -> push rax + loope 5 + jmp 0xa + jmp 0x8000000080000ffd

    fn relocate_64b_branch(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        relocate_64b(instructions, old_address, new_address, expected);
    }

    #[rstest]
    #[cfg(target_pointer_width = "64")]
    #[case::mov_lhs("48891d08000000", 0x100000000, 0, "48b80f00000001000000488918")] // mov [rip + 8], rbx -> mov rax, 0x10000000f + mov [rax], rbx
    #[case::mov_lhs_32("891d08000000", 0x100000000, 0, "48b80e000000010000008918")] // mov [rip + 8], ebx -> mov rax, 0x10000000e + mov [rax], ebx
    #[case::mov_lhs_16("66891d08000000", 0x100000000, 0, "48b80f00000001000000668918")] // mov [rip + 8], bx -> mov rax, 0x10000000f + mov [rax], bx
    #[case::mov_rhs("488b1d08000000", 0x100000000, 0, "48b80f00000001000000488b18")] // mov rbx, [rip + 8] -> mov rax, 0x10000000f + mov rbx, [rax]
    #[case::mov_rhs_32("8b1d08000000", 0x100000000, 0, "48b80e000000010000008b18")] // mov ebx, [rip + 8] -> mov rax, 0x10000000e + mov ebx, [rax]
    #[case::mov_rhs_16("668b1d08000000", 0x100000000, 0, "48b80f00000001000000668b18")] // mov bx, [rip + 8] -> mov rax, 0x10000000f + mov bx, [rax]
    #[case::xchg_src("48871d08000000", 0x100000000, 0, "48b80f00000001000000488718")] // xchg rbx, [rip + 8] -> mov rax, 0x10000000f + xchg [rax], rbx
    #[case::add_lhs("48011d08000000", 0x100000000, 0, "48b80f00000001000000480118")] // add [rip + 8], rbx -> mov rax, 0x10000000f + add [rax], rbx
    #[case::add_rhs("48031d08000000", 0x100000000, 0, "48b80f00000001000000480318")] // add rbx, [rip + 8] -> mov rax, 0x10000000f + add rbx, [rax]
    #[case::adc_lhs("48111d08000000", 0x100000000, 0, "48b80f00000001000000481118")] // adc [rip + 8], rbx -> mov rax, 0x10000000f + adc [rax], rbx
    #[case::adc_rhs("48131d08000000", 0x100000000, 0, "48b80f00000001000000481318")] // adc rbx, [rip + 8] -> mov rax, 0x10000000f + adc rbx, [rax]
    #[case::or_lhs("48091d08000000", 0x100000000, 0, "48b80f00000001000000480918")] // or [rip + 8], rbx -> mov rax, 0x10000000f + or [rax], rbx
    #[case::or_rhs("480b1d08000000", 0x100000000, 0, "48b80f00000001000000480b18")] // or rbx, [rip + 8] -> mov rax, 0x10000000f + or rbx, [rax]
    #[case::sbb_lhs("48191d08000000", 0x100000000, 0, "48b80f00000001000000481918")] // sbb [rip + 8], rbx -> mov rax, 0x10000000f + sbb [rax], rbx
    #[case::sbb_rhs("481b1d08000000", 0x100000000, 0, "48b80f00000001000000481b18")] // sbb rbx, [rip + 8] -> mov rax, 0x10000000f + sbb rbx, [rax]
    #[case::and_lhs("48211d08000000", 0x100000000, 0, "48b80f00000001000000482118")] // and [rip + 8], rbx -> mov rax, 0x10000000f + and [rax], rbx
    #[case::and_rhs("48231d08000000", 0x100000000, 0, "48b80f00000001000000482318")] // and rbx, [rip + 8] -> mov rax, 0x10000000f + and rbx, [rax]
    #[case::sub_lhs("48291d08000000", 0x100000000, 0, "48b80f00000001000000482918")] // sub [rip + 8], rbx -> mov rax, 0x10000000f + sub [rax], rbx
    #[case::sub_rhs("482b1d08000000", 0x100000000, 0, "48b80f00000001000000482b18")] // sub rbx, [rip + 8] -> mov rax, 0x10000000f + sub rbx, [rax]
    #[case::xor_lhs("48311d08000000", 0x100000000, 0, "48b80f00000001000000483118")] // xor [rip + 8], rbx -> mov rax, 0x10000000f + xor [rax], rbx
    #[case::xor_rhs("48331d08000000", 0x100000000, 0, "48b80f00000001000000483318")] // xor rbx, [rip + 8] -> mov rax, 0x10000000f + xor rbx, [rax]
    #[case::cmp_lhs("48391d08000000", 0x100000000, 0, "48b80f00000001000000483918")] // cmp [rip + 8], rbx -> mov rax, 0x10000000f + cmp [rax], rbx
    #[case::cmp_rhs("483b1d08000000", 0x100000000, 0, "48b80f00000001000000483b18")] // cmp rbx, [rip + 8] -> mov rax, 0x10000000f + cmp rbx, [rax]
    #[case::imul_rhs("480faf1d08000000", 0x100000000, 0, "48b81000000001000000480faf18")] // imul rbx, [rip + 8] -> mov rax, 0x100000010 + imul rbx, [rax]
    #[case::test_lhs("48851d08000000", 0x100000000, 0, "48b80f00000001000000488518")] // test [rip + 8], rbx -> mov rax, 0x10000000f + test [rax], rbx
    #[case::crc32_r32_rm8("f20f38f01d08000000", 0x100000000, 0, "48b81100000001000000f20f38f018")] // crc32 ebx, byte ptr [rip + 8] -> mov rax, 0x100000011 + crc32 ebx, byte ptr [rax]
    #[case::crc32_r32_rm16(
        "66f20f38f11d08000000",
        0x100000000,
        0,
        "48b8120000000100000066f20f38f118"
    )] // crc32 ebx, word ptr [rip + 8] -> mov rax, 0x100000012 + crc32 ebx, word ptr [rax]
    #[case::crc32_r32_rm32("f20f38f11d08000000", 0x100000000, 0, "48b81100000001000000f20f38f118")] // crc32 ebx, dword ptr [rip + 8] -> mov rax, 0x100000011 + crc32 ebx, dword ptr [rax]
    #[case::lea_64("488d1d08000000", 0x100000000, 0, "48b80f00000001000000488d18")] // lea rbx, [rip + 8] -> mov rax, 0x10000000f + lea rbx, [rax]
    #[case::lea_32("8d1d08000000", 0x100000000, 0, "48b80e000000010000008d18")] // lea ebx, [rip + 8] -> mov rax, 0x10000000e + lea ebx, [rax]
    #[case::lea_16("668d1d08000000", 0x100000000, 0, "48b80f00000001000000668d18")] // lea bx, [rip + 8] -> mov rax, 0x10000000f + lea bx, [rax]
    #[case::cmovo_rip_rel_abs("480f401d08000000", 0x100000000, 0, "48b81000000001000000480f4018")] // cmovo rbx, [rip + 8] -> mov rax, 0x100000010 + cmovo rbx, [rax]
    #[case::cmovno_rip_rel_abs("480f411d08000000", 0x100000000, 0, "48b81000000001000000480f4118")] // cmovno rbx, [rip + 8] -> mov rax, 0x100000010 + cmovno rbx, [rax]
    #[case::cmovb_rip_rel_abs("480f421d08000000", 0x100000000, 0, "48b81000000001000000480f4218")] // cmovb rbx, [rip + 8] -> mov rax, 0x100000010 + cmovb rbx, [rax]
    #[case::cmovnb_rip_rel_abs("480f431d08000000", 0x100000000, 0, "48b81000000001000000480f4318")] // cmovnb rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnb rbx, [rax]
    #[case::cmovz_rip_rel_abs("480f441d08000000", 0x100000000, 0, "48b81000000001000000480f4418")] // cmovz rbx, [rip + 8] -> mov rax, 0x100000010 + cmovz rbx, [rax]
    #[case::cmovnz_rip_rel_abs("480f451d08000000", 0x100000000, 0, "48b81000000001000000480f4518")] // cmovnz rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnz rbx, [rax]
    #[case::cmovbe_rip_rel_abs("480f461d08000000", 0x100000000, 0, "48b81000000001000000480f4618")] // cmovbe rbx, [rip + 8] -> mov rax, 0x100000010 + cmovbe rbx, [rax]
    #[case::cmovnbe_rip_rel_abs("480f471d08000000", 0x100000000, 0, "48b81000000001000000480f4718")] // cmovnbe rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnbe rbx, [rax]
    #[case::cmovs_rip_rel_abs("480f481d08000000", 0x100000000, 0, "48b81000000001000000480f4818")] // cmovs rbx, [rip + 8] -> mov rax, 0x100000010 + cmovs rbx, [rax]
    #[case::cmovns_rip_rel_abs("480f491d08000000", 0x100000000, 0, "48b81000000001000000480f4918")] // cmovns rbx, [rip + 8] -> mov rax, 0x100000010 + cmovns rbx, [rax]
    #[case::cmovp_rip_rel_abs("480f4a1d08000000", 0x100000000, 0, "48b81000000001000000480f4a18")] // cmovp rbx, [rip + 8] -> mov rax, 0x100000010 + cmovp rbx, [rax]
    #[case::cmovnp_rip_rel_abs("480f4b1d08000000", 0x100000000, 0, "48b81000000001000000480f4b18")] // cmovnp rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnp rbx, [rax]
    #[case::cmovl_rip_rel_abs("480f4c1d08000000", 0x100000000, 0, "48b81000000001000000480f4c18")] // cmovl rbx, [rip + 8] -> mov rax, 0x100000010 + cmovl rbx, [rax]
    #[case::cmovnl_rip_rel_abs("480f4d1d08000000", 0x100000000, 0, "48b81000000001000000480f4d18")] // cmovnl rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnl rbx, [rax]
    #[case::cmovle_rip_rel_abs("480f4e1d08000000", 0x100000000, 0, "48b81000000001000000480f4e18")] // cmovle rbx, [rip + 8] -> mov rax, 0x100000010 + cmovle rbx, [rax]
    #[case::cmovnle_rip_rel_abs("480f4f1d08000000", 0x100000000, 0, "48b81000000001000000480f4f18")] // cmovnle rbx, [rip + 8] -> mov rax, 0x100000010 + cmovnle rbx, [rax]
    #[case::bt_rip_rel_abs("0fa31d08000000", 0x100000000, 0, "48b80f000000010000000fa318")] // bt [rip + 8], ebx -> mov rax, 0x10000000f + bt [rax], ebx
    #[case::shld_cl("0fa51d08000000", 0x100000000, 0, "48b80f000000010000000fa518")] // shld [rip + 8], ebx, CL -> mov rax, 0x10000000f + shld [rax], ebx, CL
    #[case::shld_imm8("0fa41d0800000005", 0x100000000, 0, "48b810000000010000000fa41805")] // shld [rip + 8], ebx, 5 -> mov rax, 0x100000010 + shld [rax], ebx, 5
    #[case::shrd_cl("0fad1d08000000", 0x100000000, 0, "48b80f000000010000000fad18")] // shrd [rip + 8], ebx, CL -> mov rax, 0x10000000f + shrd [rax], ebx, CL
    #[case::shrd_imm8("0fac1d0800000005", 0x100000000, 0, "48b810000000010000000fac1805")] // shrd [rip + 8], ebx, 5 -> mov rax, 0x100000010 + shrd [rax], ebx, 5
    #[case::bts_rip_rel_abs("0fab1d08000000", 0x100000000, 0, "48b80f000000010000000fab18")] // bts [rip + 8], ebx -> mov rax, 0x10000000f + bts [rax], ebx
    #[case::cmpxchg_rip_rel_abs("0fb11d08000000", 0x100000000, 0, "48b80f000000010000000fb118")] // cmpxchg [rip + 8], ebx -> mov rax, 0x10000000f + cmpxchg [rax], ebx
    #[case::btr_rip_rel_abs("0fb31d08000000", 0x100000000, 0, "48b80f000000010000000fb318")] // btr [rip + 8], ebx -> mov rax, 0x10000000f + btr [rax], ebx
    #[case::popcnt_rip_rel_abs("f30fb81d08000000", 0x100000000, 0, "48b81000000001000000f30fb818")] // popcnt ebx, [rip + 8] -> mov rax, 0x100000010 + popcnt ebx, [rax]
    #[case::btc_rip_rel_abs("0fbb1d08000000", 0x100000000, 0, "48b80f000000010000000fbb18")] // btc [rip + 8], ebx -> mov rax, 0x10000000f + btc [rax], ebx
    #[case::bt_imm8("480fba250800000008", 0x100000000, 0, "48b81100000001000000480fba2008")] // bt qword ptr [rip + 8], 8 -> mov rax, 0x100000011 + bt [rax], 8
    #[case::btc_imm8("480fba3d0800000008", 0x100000000, 0, "48b81100000001000000480fba3808")] // btc qword ptr [rip + 8], 8 -> mov rax, 0x100000011 + btc [rax], 8
    #[case::btr_imm8("480fba350800000008", 0x100000000, 0, "48b81100000001000000480fba3008")] // btr qword ptr [rip + 8], 8 -> mov rax, 0x100000011 + btr [rax], 8
    #[case::bts_imm8("480fba2d0800000008", 0x100000000, 0, "48b81100000001000000480fba2808")] // bts qword ptr [rip + 8], 8 -> mov rax, 0x100000011 + bts [rax], 8
    #[case::bsf_rip_rel_abs("0fbc1d08000000", 0x100000000, 0, "48b80f000000010000000fbc18")] // bsf ebx, [rip + 8] -> mov rax, 0x10000000f + bsf ebx, [rax]
    #[case::bsr_rip_rel_abs("0fbd1d08000000", 0x100000000, 0, "48b80f000000010000000fbd18")] // bsr ebx, [rip + 8] -> mov rax, 0x10000000f + bsr ebx, [rax]
    #[case::xadd_rip_rel_abs("0fc11d08000000", 0x100000000, 0, "48b80f000000010000000fc118")] // xadd [rip + 8], ebx -> mov rax, 0x10000000f + xadd [rax], ebx

    // Single Param
    #[case::inc_rip_rel_abs("ff0508000000", 0x100000000, 0, "48b80e00000001000000ff00")]
    // inc dword ptr [rip + 8] -> mov rax, 0x10000000e + inc dword ptr [rax]

    // [Register, Memory, Immediate]
    #[case::imul_reg_mem_imm("486b1d0800000020", 0x100000000, 0, "48b81000000001000000486b1820")]
    // imul rbx, [rip + 8], 32 -> mov rax, 0x100000010 + imul rbx, qword ptr [rax], 0x20

    // Extensions
    #[case::adcx_lhs(
        "66480f38f61d08000000",
        0x100000000,
        0,
        "48b8120000000100000066480f38f618"
    )] // adcx rbx, [rip + 8] -> mov rax, 0x100000012 + adcx rbx, [rax]

    // SSE/AVX
    #[case::addpd_rhs("660f580d08000000", 0x100000000, 0, "48b81000000001000000660f5808")] // addpd xmm1, [rip + 8] -> mov rax, 0x100000010 + addpd xmm1, [rax]
    #[case::vaddpd_rhs("c5ed580d08000000", 0x100000000, 0, "48b81000000001000000c5ed5808")] // vaddpd ymm1, ymm2, [rip + 8] -> mov rax, 0x100000010 + vaddpd ymm1, ymm2, [rax]
    #[case::vaddpd_rhs_xmm("c5e9580d08000000", 0x100000000, 0, "48b81000000001000000c5e95808")] // vaddpd xmm1, xmm2, [rip + 8] -> mov rax, 0x100000010 + vaddpd xmm1, xmm2, [rax]
    #[case::vaddpd_rhs_zmm(
        "62f1ed48580d08000000",
        0x100000000,
        0,
        "48b8120000000100000062f1ed485808"
    )] // vaddpd zmm1, zmm2, [rip + 8] -> mov rax, 0x100000012 + vaddpd zmm1, zmm2, [rax]
    fn relocate_64b_rip_rel(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        relocate_64b(instructions, old_address, new_address, expected);
    }

    fn relocate_64b(
        instructions: String,
        old_address: usize,
        new_address: usize,
        expected: String,
    ) {
        // Remove spaces and convert the string to a vector of bytes
        let hex_bytes: Vec<u8> = str_to_vec(instructions);
        let instructions =
            get_stolen_instructions(true, hex_bytes.len(), &hex_bytes, old_address).unwrap();
        let mut result = Vec::new();
        relocate_code(
            true,
            &instructions.0,
            new_address,
            Some(AllRegisters::rax),
            &mut result,
        )
        .unwrap();

        assert_eq!(hex::encode(result), expected);
    }
}
