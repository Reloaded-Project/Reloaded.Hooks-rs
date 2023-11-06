extern crate alloc;

use crate::all_registers::AllRegisters;
use crate::common::util::invert_branch_condition::invert_branch_condition;
use alloc::{string::ToString, vec::Vec};
use iced_x86::Instruction;
use iced_x86::{BlockEncoder, BlockEncoderOptions, Code, FlowControl, InstructionBlock};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::{smallvec, SmallVec};

use super::patches::{patch_jump_conditional, patch_loop, patch_relative_branch};

/// Relocates the code to a new location.
///
/// # Parameters
/// `is_64bit`: Whether the code is 64bit or not.
/// `instructions`: The instructions to relocate.
/// `new_pc`: The new program counter (RIP/EIP).
/// `scratch_gpr`: A scratch general purpose register that can be used for operations.
/// `scratch_xmm`: A scratch general xmm register that can be used for operations.
#[allow(dead_code)]
pub(crate) fn relocate_code(
    is_64bit: bool,
    instructions: &SmallVec<[Instruction; 4]>,
    new_pc: usize,
    scratch_gpr: AllRegisters,
    scratch_xmm: AllRegisters,
) -> Result<Vec<u8>, CodeRewriterError> {
    let mut new_isns: SmallVec<[Instruction; 4]> = smallvec![];
    let mut current_new_pc = new_pc;

    // Note: These translations can only happen in x64, because in x86, branches will always be reachable.
    // Otherwise we need to translate the jmp/call to an absolute address.
    for instruction in instructions {
        let flow_control = instruction.flow_control();
        match flow_control {
            FlowControl::UnconditionalBranch | FlowControl::Call => {
                // Note: Check docs for `UnconditionalBranch` and `Call` above for instructions accepted into this branch.
                // If this is not a near call or jump, copy the instruction straight up.
                let is_call_near = instruction.is_call_near();
                let is_jmp_near = instruction.is_jmp_short_or_near();
                if !is_call_near && !is_jmp_near {
                    append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
                    continue;
                }

                patch_relative_branch(
                    scratch_gpr,
                    &mut new_isns,
                    &mut current_new_pc,
                    instruction,
                    is_call_near,
                )?;
            }
            FlowControl::IndirectBranch | FlowControl::IndirectCall => {
                // Unless this is RIP relative, we leave this alone.
            }
            FlowControl::ConditionalBranch => {
                // Conditional Branch

                /*
                    Jcc: https://www.felixcloutier.com/x86/jcc
                    LOOP: https://www.felixcloutier.com/x86/loop:loopcc
                    JRCXZ: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_counter_register_is_zero
                    JKccD: ??
                */

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
                }

                // Otherwise if unhandled, we copy.
                append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
            }
            FlowControl::Next => {
                // Detect RIP Relative Read
                append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
            }
            _ => {
                append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
            }
        }
    }

    // Relocate the code to some new location. It can fix short/near branches and
    // convert them to short/near/long forms if needed. This also works even if it's a
    // jrcxz/loop/loopcc instruction which only have short forms.
    //
    // It can currently only fix RIP relative operands if the new location is within 2GB
    // of the target data location.
    //
    // Note that a block is not the same thing as a basic block. A block can contain any
    // number of instructions, including any number of branch instructions. One block
    // should be enough unless you must relocate different blocks to different locations.

    let block = InstructionBlock::new(&new_isns, new_pc as u64);
    // This method can also encode more than one block but that's rarely needed, see above comment.
    let result = match BlockEncoder::encode(
        if is_64bit { 64 } else { 32 },
        block,
        BlockEncoderOptions::NONE,
    ) {
        Err(err) => panic!("{}", err),
        Ok(result) => result,
    };

    let new_code = result.code_buffer;

    Ok(new_code)
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
    if (-0x80000000..0x7FFFFFFF).contains(&delta) {
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
    if (-0x80000000..0x7FFFFFFF).contains(&delta) {
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
    use rstest::rstest;

    // TODO: Library is borked with code in 4GiB addresses.

    #[rstest]
    #[case::simple_branch_pad("50eb02", 4096, 0, "50e9ff0f0000")] // push + jmp +2 -> push + jmp +4098
    #[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_absolute_jmp_i8("eb02", 0x80000000, 0, "48b80400008000000000ffe0")] // jmp +2 -> mov rax, 0x80000004 + jmp rax
    #[case::to_absolute_jmp_i32("e9fb0f0000", 0x80000000, 0, "48b80010008000000000ffe0")] // jmp +4091 -> mov rax, 0x80001000 + jmp rax
    #[case::to_absolute_call("e8ffffffff", 0x80000000, 0, "48b80400008000000000ffd0")] // call -1 -> call rax, 0x80000004 + call rax
    #[case::to_abs_cond_jmp_jo("7002", 0x80000000, 0, "710c48b80400008000000000ffe0")] // jo +2 -> jno +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_pad("507002", 0x80000000, 0, "50710c48b80500008000000000ffe0")] // push + jo +2 -> push + jno +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jb("7202", 0x80000000, 0, "730c48b80400008000000000ffe0")] // jb +2 -> jnb +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jz("7402", 0x80000000, 0, "750c48b80400008000000000ffe0")] // jz +2 -> jnz +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jbe("7602", 0x80000000, 0, "770c48b80400008000000000ffe0")] // jbe +2 -> jnbe +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_js("7802", 0x80000000, 0, "790c48b80400008000000000ffe0")] // js +2 -> jns +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jp("7a02", 0x80000000, 0, "7b0c48b80400008000000000ffe0")] // jp +2 -> jnp +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jl("7c02", 0x80000000, 0, "7d0c48b80400008000000000ffe0")] // jl +2 -> jnl +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jle("7e02", 0x80000000, 0, "7f0c48b80400008000000000ffe0")] // jle +2 -> jnle +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jno("7102", 0x80000000, 0, "700c48b80400008000000000ffe0")] // jno +2 -> jo +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnb("7302", 0x80000000, 0, "720c48b80400008000000000ffe0")] // jnb +2 -> jb +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnz("7502", 0x80000000, 0, "740c48b80400008000000000ffe0")] // jnz +2 -> jz +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnbe("7702", 0x80000000, 0, "760c48b80400008000000000ffe0")] // jnbe +2 -> jbe +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jns("7902", 0x80000000, 0, "780c48b80400008000000000ffe0")] // jns +2 -> js +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnp("7b02", 0x80000000, 0, "7a0c48b80400008000000000ffe0")] // jnp +2 -> jp +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnl("7d02", 0x80000000, 0, "7c0c48b80400008000000000ffe0")] // jnl +2 -> jl +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnle("7f02", 0x80000000, 0, "7e0c48b80400008000000000ffe0")] // jnle +2 -> jle +12 <skip> + mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jo_i32("0f80fa0f0000", 0x80000000, 0, "710c48b80010008000000000ffe0")] // jo 4096 -> jno +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jb_i32("0f82fa0f0000", 0x80000000, 0, "730c48b80010008000000000ffe0")] // jb 4096 -> jnb +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jz_i32("0f84fa0f0000", 0x80000000, 0, "750c48b80010008000000000ffe0")] // jz 4096 -> jnz +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jbe_i32("0f86fa0f0000", 0x80000000, 0, "770c48b80010008000000000ffe0")] // jbe 4096 -> jnbe +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_js_i32("0f88fa0f0000", 0x80000000, 0, "790c48b80010008000000000ffe0")] // js 4096 -> jns +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jp_i32("0f8afa0f0000", 0x80000000, 0, "7b0c48b80010008000000000ffe0")] // jp 4096 -> jnp +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jl_i32("0f8cfa0f0000", 0x80000000, 0, "7d0c48b80010008000000000ffe0")] // jl 4096 -> jnl +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jle_i32("0f8efa0f0000", 0x80000000, 0, "7f0c48b80010008000000000ffe0")] // jle 4096 -> jnle +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jno_i32("0f81fa0f0000", 0x80000000, 0, "700c48b80010008000000000ffe0")] // jno 4096 -> jo +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnb_i32("0f83fa0f0000", 0x80000000, 0, "720c48b80010008000000000ffe0")] // jnb 4096 -> jb +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnz_i32("0f85fa0f0000", 0x80000000, 0, "740c48b80010008000000000ffe0")] // jnz 4096 -> jz +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnbe_i32("0f87fa0f0000", 0x80000000, 0, "760c48b80010008000000000ffe0")] // jnbe 4096 -> jbe +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jns_i32("0f89fa0f0000", 0x80000000, 0, "780c48b80010008000000000ffe0")] // jns 4096 -> js +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnp_i32("0f8bfa0f0000", 0x80000000, 0, "7a0c48b80010008000000000ffe0")] // jnp 4096 -> jp +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnl_i32("0f8dfa0f0000", 0x80000000, 0, "7c0c48b80010008000000000ffe0")] // jnl 4096 -> jl +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnle_i32("0f8ffa0f0000", 0x80000000, 0, "7e0c48b80010008000000000ffe0")] // jnle 4096 -> jle +12 <skip> + mov rax, 0x80001000 + jmp rax
    #[case::loop_backward_abs("50e2fa", 0x80001000, 0, "50e202eb0c48b8fd0f008000000000ffe0")] // push rax + loop -3 -> push rax + loop +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loope_backward_abs("50e1fa", 0x80001000, 0, "50e102eb0c48b8fd0f008000000000ffe0")] // push rax + loope -3 -> push rax + loope +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loopne_backward_abs("50e0fa", 0x80001000, 0, "50e002eb0c48b8fd0f008000000000ffe0")] // push rax + loopne -3 -> push rax + loopne +2 + jmp 0x11 + movabs rax, 0x80000ffd + jmp rax
    #[case::loop_backward_i8("50e2fa", 4096, 0, "5048ffc90f85f30f0000")] // push rax + loop -3 -> push rax + dec ecx + jnz 0xffd
    #[case::loop_backward_i32("50e2fa", 0x8000000, 0, "5048ffc90f85f3ffff07")] // push rax + loop -3 -> push rax + dec ecx + jnz 0x7fffffd
    #[case::loope_backward_i8("50e1fa", 4096, 0, "50e102eb05e9f30f0000")] // push rax + loope -3 -> push rax + loope 5 + jmp 0xa + jmp 0xffd
    #[case::loope_backward_i32("50e1fa", 0x8000000, 0, "50e102eb05e9f3ffff07")] // push rax + loope -3 -> push rax + loope 5 + jmp 0xa + jmp 0x7fffffd
    #[case::loopne_backward_i8("50e0fa", 4096, 0, "50e002eb05e9f30f0000")] // push rax + loopne -3 -> push rax + loopne 5 + jmp 0xa + jmp 0xffd
    #[case::loopne_backward_i32("50e0fa", 0x8000000, 0, "50e002eb05e9f3ffff07")]
    // push rax + loopne -3 -> push rax + loopne 5 + jmp 0xa + jmp 0x7fffffd

    // Some tests when in upper bytes
    #[case::simple_branch_upper64("eb02", 0x8000000000001000, 0x8000000000000000, "e9ff0f0000")] // jmp +2 -> jmp +4098
    #[case::to_absolute_jmp_8b_upper64(
        "eb02",
        0x8000000080000000,
        0x8000000000000000,
        "48b80400008000000080ffe0"
    )] // jmp +2 -> mov rax, 0x8000000000000004 + jmp rax
    #[case::to_absolute_jmp_i32_upper64(
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
    #[case::to_abs_cond_jmp_jo_upper64(
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

    //#[case::rip_relative_beyond_2gib("488B0500000000", 0, 0x100000000, "488b0500000000")]
    fn relocate_64b(
        #[case] instructions: String,
        #[case] old_address: usize,
        #[case] new_address: usize,
        #[case] expected: String,
    ) {
        // Remove spaces and convert the string to a vector of bytes
        let hex_bytes: Vec<u8> = as_vec(instructions);
        let instructions =
            get_stolen_instructions(true, hex_bytes.len() as u8, &hex_bytes, old_address).unwrap();
        let result = relocate_code(
            true,
            &instructions.0,
            new_address,
            AllRegisters::rax,
            AllRegisters::xmm0,
        );

        assert_eq!(hex::encode(result.unwrap()), expected);
    }

    fn as_vec(hex: String) -> Vec<u8> {
        hex.as_bytes()
            .chunks(2)
            .map(|chunk| {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(hex_str, 16).unwrap()
            })
            .collect()
    }
}
