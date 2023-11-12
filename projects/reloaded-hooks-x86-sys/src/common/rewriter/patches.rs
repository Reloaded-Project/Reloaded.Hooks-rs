extern crate alloc;

use super::code_rewriter::{
    append_if_can_encode_relative, append_instruction_with_new_pc, can_encode_relative,
};
use crate::all_registers::AllRegisters;
use crate::common::util::invert_branch_condition::invert_branch_condition;
use alloc::string::ToString;
use iced_x86::{Code, Instruction, MemoryOperand, OpKind, Register};
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

pub(crate) fn append_if_can_encode_relative_rip(
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> bool {
    // If the branch offset is within 2GiB, do no action
    // because Iced will handle it for us on re-encode.
    let target = instruction.memory_displacement64();
    let end_of_new_inst = *current_new_pc + instruction.len();
    let delta = (target - end_of_new_inst as u64) as i64;
    if (-0x80000000..=0x7FFFFFFF).contains(&delta) {
        append_instruction_with_new_pc(new_isns, current_new_pc, instruction);
        return true;
    }

    false
}

pub(crate) fn patch_rip_relative_operand(
    scratch_gpr: AllRegisters,
    scratch_xmm: AllRegisters,
    new_isns: &mut SmallVec<[Instruction; 4]>,
    current_new_pc: &mut usize,
    instruction: &Instruction,
) -> Result<(), CodeRewriterError> {
    if append_if_can_encode_relative_rip(new_isns, current_new_pc, instruction) {
        return Ok(());
    }

    let target = instruction.memory_displacement64();
    let scratch_reg = scratch_gpr.as_iced_allregister().unwrap();

    let mut params = PatchInstructionParams {
        instruction,
        scratch_reg,
        target,
        new_isns,
        current_new_pc,
    };

    /*
    if instruction.op0_kind() == OpKind::Memory && instruction.op1_kind() == OpKind::Register {
        patch_rip_rel_reg(&mut params, instruction.code(), OpType::RegToMem, 3);
    }
    */

    // Every single instruction marked as `rModRM:r/m (r` in
    // https://cdrdv2-public.intel.com/789583/325462-sdm-vol-1-2abcd-3abcd-4.pdf
    // Intel® 64 and IA-32 Architectures Software Developer’s Manual
    match instruction.code() {
        // +1 for '66' or '48' or '88' prefix
        // Core x86 Instructions
        Code::Mov_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Mov_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Mov_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Mov_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Mov_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Mov_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Mov_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Mov_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Lea_r64_m => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Lea_r32_m => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Lea_r16_m => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Xchg_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xchg_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Xchg_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xchg_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),

        Code::Add_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Add_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Add_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Add_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Add_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Add_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Add_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Add_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Adc_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Adc_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Adc_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Adc_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Adc_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Adc_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Adc_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Adc_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Or_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Or_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Or_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Or_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Or_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Or_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Or_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Or_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Sbb_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sbb_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Sbb_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Sbb_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Sbb_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sbb_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Sbb_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sbb_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::And_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::And_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::And_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::And_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::And_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::And_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::And_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::And_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Sub_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sub_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Sub_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Sub_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Sub_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sub_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Sub_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Sub_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Xor_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xor_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Xor_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Xor_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Xor_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xor_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Xor_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xor_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Cmp_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Cmp_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmp_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Cmp_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 2),
        Code::Cmp_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Cmp_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmp_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Cmp_r8_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),

        Code::Test_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Test_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Test_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Test_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),

        // Multi-byte extensions (0F)
        Code::Imul_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Imul_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Imul_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Crc32_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 5),
        Code::Crc32_r64_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 5),
        Code::Crc32_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Crc32_r32_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 5),
        Code::Crc32_r32_rm8 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 5),

        Code::Cmovo_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovo_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovo_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Cmovno_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovno_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovno_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Cmovb_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovb_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovb_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNB
        Code::Cmovae_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovae_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovae_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVZ (Zero / Equal) / CMOVE
        Code::Cmove_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmove_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmove_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNZ (Not Zero / Not Equal)
        Code::Cmovne_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovne_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovne_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVBE (Below or Equal / Not Above)
        Code::Cmovbe_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovbe_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovbe_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNBE (Not Below or Equal / Above)
        Code::Cmova_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmova_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmova_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVS (Sign)
        Code::Cmovs_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovs_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovs_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNS (Not Sign)
        Code::Cmovns_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovns_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovns_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVP (Parity / Parity Even)
        Code::Cmovp_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovp_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovp_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNP (Not Parity / Parity Odd)
        Code::Cmovnp_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovnp_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovnp_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVL (Less / Not Greater)
        Code::Cmovl_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovl_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovl_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNL (Not Less / Greater or Equal)
        Code::Cmovge_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovge_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovge_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVLE (Less or Equal / Not Greater)
        Code::Cmovle_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovle_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovle_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        // CMOVNLE (Not Less or Equal / Greater)
        Code::Cmovg_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Cmovg_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Cmovg_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Bt_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),
        Code::Bt_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Bt_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),

        Code::Bts_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),
        Code::Bts_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Bts_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),

        Code::Shld_rm64_r64_CL => patch_rip_rel_op_shrd(&mut params, true, 3),
        Code::Shld_rm32_r32_CL => patch_rip_rel_op_shrd(&mut params, true, 4),
        Code::Shld_rm16_r16_CL => patch_rip_rel_op_shrd(&mut params, true, 4),

        Code::Shld_rm64_r64_imm8 => patch_rip_rel_op_shrd(&mut params, false, 4),
        Code::Shld_rm32_r32_imm8 => patch_rip_rel_op_shrd(&mut params, false, 5),
        Code::Shld_rm16_r16_imm8 => patch_rip_rel_op_shrd(&mut params, false, 5),

        Code::Shrd_rm64_r64_CL => patch_rip_rel_op_shrd(&mut params, true, 3),
        Code::Shrd_rm32_r32_CL => patch_rip_rel_op_shrd(&mut params, true, 4),
        Code::Shrd_rm16_r16_CL => patch_rip_rel_op_shrd(&mut params, true, 4),

        Code::Shrd_rm64_r64_imm8 => patch_rip_rel_op_shrd(&mut params, false, 4),
        Code::Shrd_rm32_r32_imm8 => patch_rip_rel_op_shrd(&mut params, false, 5),
        Code::Shrd_rm16_r16_imm8 => patch_rip_rel_op_shrd(&mut params, false, 5),

        Code::Cmpxchg_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Cmpxchg_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 2),
        Code::Cmpxchg_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Cmpxchg_rm8_r8 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),

        Code::Btr_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),
        Code::Btr_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Btr_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),

        Code::Popcnt_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Popcnt_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Popcnt_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Btc_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),
        Code::Btc_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Btc_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),

        Code::Bt_rm64_imm8 => patch_rip_rel_imm(&mut params, 5),
        Code::Bt_rm32_imm8 => patch_rip_rel_imm(&mut params, 6),
        Code::Bt_rm16_imm8 => patch_rip_rel_imm(&mut params, 6),

        Code::Bts_rm64_imm8 => patch_rip_rel_imm(&mut params, 5),
        Code::Bts_rm32_imm8 => patch_rip_rel_imm(&mut params, 6),
        Code::Bts_rm16_imm8 => patch_rip_rel_imm(&mut params, 6),

        Code::Btr_rm64_imm8 => patch_rip_rel_imm(&mut params, 5),
        Code::Btr_rm32_imm8 => patch_rip_rel_imm(&mut params, 6),
        Code::Btr_rm16_imm8 => patch_rip_rel_imm(&mut params, 6),

        Code::Btc_rm64_imm8 => patch_rip_rel_imm(&mut params, 5),
        Code::Btc_rm32_imm8 => patch_rip_rel_imm(&mut params, 6),
        Code::Btc_rm16_imm8 => patch_rip_rel_imm(&mut params, 6),

        Code::Bsf_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Bsf_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Bsf_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Bsr_r64_rm64 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),
        Code::Bsr_r32_rm32 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 3),
        Code::Bsr_r16_rm16 => patch_rip_rel_reg(&mut params, OpType::MemToReg, 4),

        Code::Xadd_rm64_r64 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),
        Code::Xadd_rm32_r32 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 3),
        Code::Xadd_rm16_r16 => patch_rip_rel_reg(&mut params, OpType::RegToMem, 4),

        // Fallback
        _ => {
            append_instruction_with_new_pc(new_isns, current_new_pc, instruction);
            Ok(())
        }
    }
}

struct PatchInstructionParams<'a> {
    instruction: &'a Instruction,
    scratch_reg: Register,
    target: u64,
    new_isns: &'a mut SmallVec<[Instruction; 4]>,
    current_new_pc: &'a mut usize,
}

enum OpType {
    RegToMem, // e.g., `add [rip + 8], rbx`
    MemToReg, // e.g., `add rbx, [rip + 8]`
}

fn patch_rip_rel_reg(
    params: &mut PatchInstructionParams,
    operand_type: OpType,
    patched_ins_len: usize,
) -> Result<(), CodeRewriterError> {
    let mut mov_address_ins =
        Instruction::with2(Code::Mov_r64_imm64, params.scratch_reg, params.target)
            .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_address_ins.set_len(10);

    let mut patched_ins = match operand_type {
        OpType::RegToMem => Instruction::with2(
            params.instruction.code(),
            MemoryOperand::with_base(params.scratch_reg),
            params.instruction.op1_register(),
        ),
        OpType::MemToReg => Instruction::with2(
            params.instruction.code(),
            params.instruction.op0_register(),
            MemoryOperand::with_base(params.scratch_reg),
        ),
    }
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    patched_ins.set_len(patched_ins_len);

    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &mov_address_ins);
    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &patched_ins);
    Ok(())
}

fn patch_rip_rel_imm(
    params: &mut PatchInstructionParams,
    patched_ins_len: usize,
) -> Result<(), CodeRewriterError> {
    let mut mov_address_ins =
        Instruction::with2(Code::Mov_r64_imm64, params.scratch_reg, params.target)
            .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_address_ins.set_len(10);

    let mut patched_ins = Instruction::with2(
        params.instruction.code(),
        MemoryOperand::with_base(params.scratch_reg),
        params.instruction.immediate32(),
    )
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    patched_ins.set_len(patched_ins_len);

    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &mov_address_ins);
    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &patched_ins);
    Ok(())
}

fn patch_rip_rel_op_shrd(
    params: &mut PatchInstructionParams,
    is_cl: bool,
    patched_ins_len: usize,
) -> Result<(), CodeRewriterError> {
    let mut mov_address_ins =
        Instruction::with2(Code::Mov_r64_imm64, params.scratch_reg, params.target)
            .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    mov_address_ins.set_len(10);

    let mut patched_ins = match is_cl {
        // Special handling for SHLD
        true => Instruction::with3(
            params.instruction.code(),
            MemoryOperand::with_base(params.scratch_reg),
            params.instruction.op1_register(),
            params.instruction.op2_register(),
        ),
        // Special handling for SHLD with immediate
        false => Instruction::with3(
            params.instruction.code(),
            MemoryOperand::with_base(params.scratch_reg),
            params.instruction.op1_register(),
            params.instruction.immediate32(),
        ),

        // Handling other instructions
        _ => unreachable!(),
    }
    .map_err(|x| CodeRewriterError::ThirdPartyAssemblerError(x.to_string()))?;
    patched_ins.set_len(patched_ins_len);

    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &mov_address_ins);
    append_instruction_with_new_pc(params.new_isns, params.current_new_pc, &patched_ins);
    Ok(())
}
