extern crate alloc;

use crate::all_registers::AllRegisters;
use alloc::{string::ToString, vec::Vec};
use iced_x86::Instruction;
use iced_x86::{BlockEncoder, BlockEncoderOptions, Code, FlowControl, InstructionBlock};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::{smallvec, SmallVec};

use super::util::invert_branch_condition::invert_branch_condition;

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
                }
                // Otherwise if unhandled, we copy.
                append_instruction_with_new_pc(&mut new_isns, &mut current_new_pc, instruction);
            }
            FlowControl::Next => {
                // Can be a memory read
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

fn append_if_can_encode_relative(
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

fn patch_relative_branch(
    scratch_gpr: AllRegisters,
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
    is_call: bool,
) -> Result<(), CodeRewriterError> {
    if append_if_can_encode_relative(new_isns, current_new_pc, instruction) {
        return Ok(());
    }

    let target = instruction.near_branch_target();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();
    let mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    let branch_ins = if is_call {
        Instruction::with1(Code::Call_rm64, scratch_reg)
    } else {
        Instruction::with1(Code::Jmp_rm64, scratch_reg)
    }
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;

    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);
    Ok(())
}

fn patch_jump_conditional(
    scratch_gpr: AllRegisters,
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    if append_if_can_encode_relative(new_isns, current_new_pc, instruction) {
        return Ok(());
    }

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

        Unfortunately, there's no way to do this with Iced :(
    */

    // Invert branch condition and make jump absolute.
    // Note: The Iced encoder fixes up the IP during the optimize step, therefore we can reuse `current_new_pc`.
    //       to insert a label, if we create pseudo instruction with IP set to current_new_pc + 1. But that's
    //       unfortunately no good, as we need to waste a byte with nop. We're not gonna do that.
    let target = instruction.near_branch_target();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();
    let inverted_condition = invert_branch_condition(instruction.code())?;

    // 12 bytes for mov + branch
    let jmp_skip = Instruction::with_branch(inverted_condition, 12)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;

    let mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    let branch_ins = Instruction::with1(Code::Jmp_rm64, scratch_reg)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;

    append_instruction_with_new_pc(new_isns, current_new_pc, &jmp_skip);
    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);

    Ok(())
}

fn append_instruction_with_new_pc(
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
    use crate::common::disasm::relocate_code;
    use crate::common::util::get_stolen_instructions::get_stolen_instructions;
    use rstest::rstest;

    #[rstest]
    #[case::simple_branch("eb02", 4096, 0, "e9ff0f0000")] // jmp 4 -> jmp 4100
    #[case::to_absolute_jmp_8b("eb02", 0x80000000, 0, "48b80400008000000000ffe0")] // jmp 4 -> mov rax, 0x80000004 + jmp rax
    #[case::to_absolute_jmp_32b("e9fb0f0000", 0x80000000, 0, "48b80010008000000000ffe0")] // jmp 4096 -> mov rax, 0x80001000 + jmp rax
    #[case::to_absolute_call("e8ffffffff", 0x80000000, 0, "48b80400008000000000ffd0")] // call 4 -> call rax, 0x80000004 + call rax
    #[case::to_abs_cond_jmp_jo("7002", 0x80000000, 0, "710a48b80400008000000000ffe0")] // jo 0x4 -> jno 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jb("7202", 0x80000000, 0, "730a48b80400008000000000ffe0")] // jb 0x4 -> jnb 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jz("7402", 0x80000000, 0, "750a48b80400008000000000ffe0")] // jz 0x4 -> jnz 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jbe("7602", 0x80000000, 0, "770a48b80400008000000000ffe0")] // jbe 0x4 -> jnbe 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_js("7802", 0x80000000, 0, "790a48b80400008000000000ffe0")] // js 0x4 -> jns 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jp("7a02", 0x80000000, 0, "7b0a48b80400008000000000ffe0")] // jp 0x4 -> jnp 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jl("7c02", 0x80000000, 0, "7d0a48b80400008000000000ffe0")] // jl 0x4 -> jnl 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jle("7e02", 0x80000000, 0, "7f0a48b80400008000000000ffe0")] // jle 0x4 -> jnle 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jno("7102", 0x80000000, 0, "700a48b80400008000000000ffe0")] // jno 0x4 -> jo 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnb("7302", 0x80000000, 0, "720a48b80400008000000000ffe0")] // jnb 0x4 -> jb 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnz("7502", 0x80000000, 0, "740a48b80400008000000000ffe0")] // jnz 0x4 -> jz 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnbe("7702", 0x80000000, 0, "760a48b80400008000000000ffe0")] // jnbe 0x4 -> jbe 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jns("7902", 0x80000000, 0, "780a48b80400008000000000ffe0")] // jns 0x4 -> js 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnp("7b02", 0x80000000, 0, "7a0a48b80400008000000000ffe0")] // jnp 0x4 -> jp 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnl("7d02", 0x80000000, 0, "7c0a48b80400008000000000ffe0")] // jnl 0x4 -> jl 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jnle("7f02", 0x80000000, 0, "7e0a48b80400008000000000ffe0")] // jnle 0x4 -> jle 12 <skip> -> mov rax, 0x80000004 + jmp rax
    #[case::to_abs_cond_jmp_jo_i32("0f80fa0f0000", 0x80000000, 0, "710a48b80010008000000000ffe0")] // jo 4096 -> jno 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jb_i32("0f82fa0f0000", 0x80000000, 0, "730a48b80010008000000000ffe0")] // jb 4096 -> jnb 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jz_i32("0f84fa0f0000", 0x80000000, 0, "750a48b80010008000000000ffe0")] // jz 4096 -> jnz 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jbe_i32("0f86fa0f0000", 0x80000000, 0, "770a48b80010008000000000ffe0")] // jbe 4096 -> jnbe 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_js_i32("0f88fa0f0000", 0x80000000, 0, "790a48b80010008000000000ffe0")] // js 4096 -> jns 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jp_i32("0f8afa0f0000", 0x80000000, 0, "7b0a48b80010008000000000ffe0")] // jp 4096 -> jnp 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jl_i32("0f8cfa0f0000", 0x80000000, 0, "7d0a48b80010008000000000ffe0")] // jl 4096 -> jnl 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jle_i32("0f8efa0f0000", 0x80000000, 0, "7f0a48b80010008000000000ffe0")] // jle 4096 -> jnle 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jno_i32("0f81fa0f0000", 0x80000000, 0, "700a48b80010008000000000ffe0")] // jno 4096 -> jo 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnb_i32("0f83fa0f0000", 0x80000000, 0, "720a48b80010008000000000ffe0")] // jnb 4096 -> jb 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnz_i32("0f85fa0f0000", 0x80000000, 0, "740a48b80010008000000000ffe0")] // jnz 4096 -> jz 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnbe_i32("0f87fa0f0000", 0x80000000, 0, "760a48b80010008000000000ffe0")] // jnbe 4096 -> jbe 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jns_i32("0f89fa0f0000", 0x80000000, 0, "780a48b80010008000000000ffe0")] // jns 4096 -> js 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnp_i32("0f8bfa0f0000", 0x80000000, 0, "7a0a48b80010008000000000ffe0")] // jnp 4096 -> jp 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnl_i32("0f8dfa0f0000", 0x80000000, 0, "7c0a48b80010008000000000ffe0")] // jnl 4096 -> jl 12 <skip> -> mov rax, 0x80001000 + jmp rax
    #[case::to_abs_cond_jmp_jnle_i32("0f8ffa0f0000", 0x80000000, 0, "7e0a48b80010008000000000ffe0")] // jnle 4096 -> jle 12 <skip> -> mov rax, 0x80001000 + jmp rax

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
