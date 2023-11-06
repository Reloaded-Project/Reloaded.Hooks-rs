extern crate alloc;

use super::code_rewriter::{
    append_if_can_encode_relative, append_instruction_with_new_pc, can_encode_relative,
};
use crate::all_registers::AllRegisters;
use crate::common::util::invert_branch_condition::invert_branch_condition;
use alloc::string::ToString;
use iced_x86::{Code, Instruction};
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use smallvec::SmallVec;

pub(crate) fn patch_relative_branch(
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
    let mut mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_ins.set_len(10);
    let mut branch_ins = if is_call {
        Instruction::with1(Code::Call_rm64, scratch_reg)
    } else {
        Instruction::with1(Code::Jmp_rm64, scratch_reg)
    }
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    branch_ins.set_len(2);

    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);
    Ok(())
}

pub(crate) fn patch_jump_conditional(
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
    let target = instruction.near_branch_target();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();
    let inverted_condition = invert_branch_condition(instruction.code())?;

    // 12 bytes for mov + branch
    let mut jmp_skip = Instruction::with_branch(inverted_condition, (*current_new_pc + 14) as u64)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jmp_skip.set_len(2);
    let mut mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_ins.set_len(10);
    let mut branch_ins = Instruction::with1(Code::Jmp_rm64, scratch_reg)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    branch_ins.set_len(2);

    append_instruction_with_new_pc(new_isns, current_new_pc, &jmp_skip);
    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);

    Ok(())
}

pub(crate) fn patch_loop(
    scratch_gpr: AllRegisters,
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    if can_encode_relative(current_new_pc, instruction) {
        if instruction.is_loop() {
            // 'loop' has optimized version for imm32, but loope/loopne can't be optimized.
            return patch_loop_imm32(new_isns, current_new_pc, instruction);
        } else {
            append_instruction_with_new_pc(new_isns, current_new_pc, instruction);
            return Ok(());
        }
    }

    /*
        Strategy (for >2GiB):

        0:  loop   4  # jump to '<reg>, OLD_LOOP_INS_JUMP_TARGET' instruction.
        2:  jmp    16 # jump to code after `jmp OLD_LOOP_JUMP_TARGET`
        4:  mov    <reg>, OLD_LOOP_INS_JUMP_TARGET
        14: jmp    <reg>
        16: <other code>

        Note:

        We cannot use same strategy of `jnz` from <2GiB scenario, unfortunately, it wouldn't be as efficient.
    */

    // Jump forward
    let target = instruction.near_branch_target();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();

    let mut loop_over = Instruction::with_branch(instruction.code(), (*current_new_pc + 4) as u64)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    loop_over.set_len(2);

    let mut jmp_skip = Instruction::with_branch(Code::Jmp_rel8_64, (*current_new_pc + 16) as u64)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jmp_skip.set_len(2);

    let mut mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_ins.set_len(10);

    let mut branch_ins = Instruction::with1(Code::Jmp_rm64, scratch_reg)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    branch_ins.set_len(2);

    append_instruction_with_new_pc(new_isns, current_new_pc, &loop_over);
    append_instruction_with_new_pc(new_isns, current_new_pc, &jmp_skip);
    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);

    Ok(())
}

fn patch_loop_imm32(
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    /*
        Strategy (for >2GiB):

        0:  dec   ecx  # jump to '<reg>, OLD_LOOP_INS_JUMP_TARGET' instruction.
        2:  jnz   <original target>
    */

    // Jump forward
    let target = instruction.near_branch_target();

    let mut dec = Instruction::with1(Code::Dec_rm64, iced_x86::Register::RCX)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    dec.set_len(2);

    let mut jnz = Instruction::with_branch(Code::Jne_rel32_64, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jnz.set_len(6);

    append_instruction_with_new_pc(new_isns, current_new_pc, &dec);
    append_instruction_with_new_pc(new_isns, current_new_pc, &jnz);

    Ok(())
}

pub(crate) fn patch_jcx(
    scratch_gpr: AllRegisters,
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    if can_encode_relative(current_new_pc, instruction) {
        return patch_jcx_imm32(new_isns, current_new_pc, instruction);
    }

    /*
        Strategy (for >2GiB):

        0:  jcx    4  # jump to '<reg>, OLD_LOOP_INS_JUMP_TARGET' instruction.
        2:  jmp    16 # jump to code after `jmp OLD_LOOP_JUMP_TARGET`
        4:  mov    <reg>, OLD_LOOP_INS_JUMP_TARGET
        14: jmp    <reg>
        16: <other code>

        Note:

        We cannot use same strategy of `jnz` from <2GiB scenario, unfortunately, it wouldn't be as efficient.
    */

    // Jump forward
    let target = instruction.near_branch_target();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();

    let mut jcx_over = Instruction::with_branch(instruction.code(), (*current_new_pc + 4) as u64)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jcx_over.set_len(2);

    let mut jmp_skip = Instruction::with_branch(Code::Jmp_rel8_64, (*current_new_pc + 16) as u64)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jmp_skip.set_len(2);

    let mut mov_ins = Instruction::with2(Code::Mov_r64_imm64, scratch_reg, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_ins.set_len(10);

    let mut branch_ins = Instruction::with1(Code::Jmp_rm64, scratch_reg)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    branch_ins.set_len(2);

    append_instruction_with_new_pc(new_isns, current_new_pc, &jcx_over);
    append_instruction_with_new_pc(new_isns, current_new_pc, &jmp_skip);
    append_instruction_with_new_pc(new_isns, current_new_pc, &mov_ins);
    append_instruction_with_new_pc(new_isns, current_new_pc, &branch_ins);

    Ok(())
}

fn patch_jcx_imm32(
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    /*
        Strategy (for >2GiB):

        0:  test rcx, rcx  # Test for zero flag.
        2:  jnz   <original target>
    */

    // Jump forward
    let target = instruction.near_branch_target();
    let mut dec = Instruction::with2(
        Code::Test_rm32_r32,
        iced_x86::Register::ECX,
        iced_x86::Register::ECX,
    )
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    dec.set_len(2); // test ECX for portability

    let mut jnz = Instruction::with_branch(Code::Jne_rel32_64, target)
        .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    jnz.set_len(6);

    append_instruction_with_new_pc(new_isns, current_new_pc, &dec);
    append_instruction_with_new_pc(new_isns, current_new_pc, &jnz);
    Ok(())
}
