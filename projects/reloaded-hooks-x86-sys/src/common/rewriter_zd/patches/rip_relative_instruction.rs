extern crate alloc;

use crate::common::traits::ToZydis;
use crate::common::util::get_stolen_instructions::ZydisInstruction;
use crate::x64;
use alloc::vec::Vec;
use iced_x86::Register;
use reloaded_hooks_portable::api::rewriter::code_rewriter::CodeRewriterError;
use zydis::ffi::{DecodedOperand, DecodedOperandKind, ImmediateInfo};
use zydis::Mnemonic::MOV;
use zydis::Register::RIP;
use zydis::{EncoderOperand, EncoderRequest, Status};

/// Patch a relative branch instruction from an older address to a new address.
/// [Docs](https://reloaded-project.github.io/Reloaded.Hooks-rs/dev/arch/x86/code_relocation/#jump-conditional)
#[cfg(feature = "x64")]
pub(crate) fn patch_rip_relative_64(
    instruction: &ZydisInstruction,
    instruction_bytes: &[u8],
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: Option<x64::Register>,
    buf: &mut Vec<u8>,
) -> Result<(), CodeRewriterError> {
    // TODO: 'Fast relocate RIP relative operands' (no scratch register)

    // For encoding, encoder allows us to specify immediates as 32-bit, even if they are imm8 etc. let's abuse this.
    // In any case, we have 3 main types of operands covered:
    //  - reg (note: this includes vector registers)
    //  - mem
    //  - imm (8/16/32)

    // We can have 0-5 operands, where any 1 operand can be RIP relative thus need to cover all permutations.
    // According to our script projects/code-generators/x86/generate_enum_ins_combos.py, the following combinations exist
    //
    // (Note: `rip == mem`)
    // (Note 2: These instructions were hand-checked by human, I may have done errors)
    //
    // 1 Operands:        [function name]
    // - rip              patch_riprel
    //
    // 2 Operands:
    // - rip, imm         patch_riprel_op
    // - rip, reg         patch_riprel_op
    // - reg, rip         patch_op_riprel
    //
    // 3 Operands:
    // - reg, reg, rip    patch_op_op_riprel
    // - reg, rip, imm    patch_op_riprel_op
    // - rip, reg, imm    patch_riprel_op_op
    // - rip, reg, reg    patch_riprel_op_op
    // - reg, rip, reg    patch_op_riprel_op
    //
    // 4 Operands:
    // - reg, reg, rip, imm patch_op_op_riprel_op [XOP Instructions, Deprecated in modern CPUS]
    // - reg, reg, reg, rip patch_op_op_op_riprel [XOP Instructions, Deprecated in modern CPUS]
    //
    // 5 Operands:
    // - reg, reg, reg, rip, imm patch_op_op_op_riprel_op [XOP Instructions, Deprecated in modern CPUS]
    // - reg, reg, rip, reg, imm patch_op_op_riprel_op_op [XOP Instructions, Deprecated in modern CPUS]

    // common code: mov [scratch], address
    let scratch = scratch_register.ok_or_else(|| {
        CodeRewriterError::NoScratchRegister("Required to rewrite RIP relative operand".into())
    });

    if instruction.operand_count_visible == 2 {
        // 2 Operands:
        // - rip, imm         patch_riprel_op
        // - rip, reg         patch_riprel_op
        // - reg, rip         patch_op_riprel
        let op_0 = &instruction.operands()[0];
        let op_1 = &instruction.operands()[1];
        let is_first_rip = is_rip_operand(op_0);

        if is_first_rip {
            match &op_1.kind {
                // rip, reg
                DecodedOperandKind::Reg(reg) => {
                    patch_riprel_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_0,
                        *reg,
                    )?;
                    return Ok(());
                }
                // rip, imm
                DecodedOperandKind::Imm(imm) => {
                    patch_riprel_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_0,
                        imm.clone().value,
                    )?;
                    return Ok(());
                }
                _ => {}
            }
        }

        let is_second_rip = is_rip_operand(op_0);
        if is_second_rip {
            // reg, rip
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                patch_op_riprel(
                    instruction,
                    dest_address,
                    source_address,
                    scratch?,
                    buf,
                    op_1,
                    *reg,
                )?;
                return Ok(());
            }
        }
    } else if instruction.operand_count_visible == 3 {
        let op_0 = &instruction.operands()[0];
        let op_1 = &instruction.operands()[1];
        let op_2 = &instruction.operands()[2];

        // 3 Operands:
        // - rip, reg, imm    patch_riprel_op_op
        // - rip, reg, reg    patch_riprel_op_op
        // - reg, rip, imm    patch_op_riprel_op
        // - reg, rip, reg    patch_op_riprel_op
        // - reg, reg, rip    patch_op_op_riprel

        let is_first_rip = is_rip_operand(op_0);
        if is_first_rip {
            if let DecodedOperandKind::Reg(reg) = &op_1.kind {
                // rip, reg, imm
                if let DecodedOperandKind::Imm(imm) = &op_2.kind {
                    patch_riprel_op_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_0,
                        *reg,
                        imm.clone().value,
                    )?;
                    return Ok(());
                }

                // rip, reg, reg
                if let DecodedOperandKind::Reg(reg2) = &op_2.kind {
                    patch_riprel_op_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_0,
                        *reg,
                        *reg2,
                    )?;
                    return Ok(());
                }
            }
        }

        let is_second_rip = is_rip_operand(op_1);
        if is_second_rip {
            // reg, rip, imm
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Imm(imm) = &op_2.kind {
                    patch_op_riprel_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_1,
                        *reg,
                        imm.clone().value,
                    )?;
                    return Ok(());
                }
            }

            // reg, rip, reg
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_2.kind {
                    patch_op_riprel_op(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_1,
                        *reg,
                        *reg2,
                    )?;
                    return Ok(());
                }
            }
        }

        let is_third_rip = is_rip_operand(op_2);
        // reg, reg, rip
        if is_third_rip {
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_1.kind {
                    patch_op_op_riprel(
                        instruction,
                        dest_address,
                        source_address,
                        scratch?,
                        buf,
                        op_2,
                        *reg,
                        *reg2,
                    )?;
                    return Ok(());
                }
            }
        }
    } else if instruction.operand_count_visible == 4 {
        // 4 Operands:
        // - reg, reg, rip, imm patch_op_op_riprel_op
        // - reg, reg, reg, rip patch_op_op_op_riprel

        let op_0 = &instruction.operands()[0];
        let op_1 = &instruction.operands()[1];
        let op_2 = &instruction.operands()[2];
        let op_3 = &instruction.operands()[3];

        let is_third_rip = is_rip_operand(op_2);
        if is_third_rip {
            // reg, reg, rip, imm
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_1.kind {
                    if let DecodedOperandKind::Imm(imm) = &op_3.kind {
                        patch_op_op_riprel_op(
                            instruction,
                            dest_address,
                            source_address,
                            scratch?,
                            buf,
                            op_2,
                            *reg,
                            *reg2,
                            imm.value,
                        )?;
                        return Ok(());
                    }
                }
            }
        }

        let is_fourth_rip = is_rip_operand(op_3);
        if is_fourth_rip {
            // reg, reg, reg, rip
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_1.kind {
                    if let DecodedOperandKind::Reg(reg3) = &op_2.kind {
                        patch_op_op_op_riprel(
                            instruction,
                            dest_address,
                            source_address,
                            scratch?,
                            buf,
                            op_3,
                            *reg,
                            *reg2,
                            *reg3,
                        )?;
                        return Ok(());
                    }
                }
            }
        }
    } else if instruction.operand_count_visible == 5 {
        // 5 Operands:
        // - reg, reg, reg, rip, imm patch_op_op_op_riprel_op
        // - reg, reg, rip, reg, imm patch_op_op_riprel_op_op

        let op_0 = &instruction.operands()[0];
        let op_1 = &instruction.operands()[1];
        let op_2 = &instruction.operands()[2];
        let op_3 = &instruction.operands()[3];
        let op_4 = &instruction.operands()[4];

        // e.g. `VPERMIL2PS xmm1, xmm2, xmm3, xmm4/m128, imm4`
        // e.g. `VPERMIL2PS xmm1, xmm2, xmm3/m128, xmm4, imm4`

        let is_fourth_rip = is_rip_operand(op_3);
        if is_fourth_rip {
            // reg, reg, reg, rip, imm
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_1.kind {
                    if let DecodedOperandKind::Reg(reg3) = &op_2.kind {
                        if let DecodedOperandKind::Imm(imm) = &op_4.kind {
                            patch_op_op_op_riprel_op(
                                instruction,
                                dest_address,
                                source_address,
                                scratch?,
                                buf,
                                op_3,
                                *reg,
                                *reg2,
                                *reg3,
                                imm.value,
                            )?;
                            return Ok(());
                        }
                    }
                }
            }
        }

        let is_third_rip = is_rip_operand(op_2);
        if is_third_rip {
            // reg, reg, rip, reg, imm
            if let DecodedOperandKind::Reg(reg) = &op_0.kind {
                if let DecodedOperandKind::Reg(reg2) = &op_1.kind {
                    if let DecodedOperandKind::Reg(reg3) = &op_3.kind {
                        if let DecodedOperandKind::Imm(imm) = &op_4.kind {
                            patch_op_op_riprel_op_op(
                                instruction,
                                dest_address,
                                source_address,
                                scratch?,
                                buf,
                                op_2,
                                *reg,
                                *reg2,
                                *reg3,
                                imm.value,
                            )?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    } else if instruction.operand_count_visible == 1 && is_rip_operand(&instruction.operands()[0]) {
        patch_riprel(
            instruction,
            dest_address,
            source_address,
            scratch?,
            buf,
            &instruction.operands()[0],
        )?;
        return Ok(());
    }

    *source_address += instruction_bytes.len();
    *dest_address += instruction_bytes.len();
    buf.extend_from_slice(instruction_bytes);

    Ok(())
}

fn is_rip_operand(op: &DecodedOperand) -> bool {
    match &op.kind {
        DecodedOperandKind::Mem(x) => x.base == RIP,
        _ => false,
    }
}

/// Example patch:
///
/// from:
///     inc dword ptr [rip+ 8]
///
/// to:
///     mov rax, 0x000000000000000E
///     inc DWORD PTR [rax]
fn patch_riprel(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax]
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .encode_extend(buf)?;

    Ok(())
}

/// Example patch:
///
/// from:
///     mov [rip + 8], rbx
///
/// to:
///     mov rax, 0x10000000f
///     mov [rax], rbx
fn patch_riprel_op<TOperand: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(op)
        .encode_extend(buf)?;

    Ok(())
}

/// Example patch:
///
/// from:
///     shld [rip + 8], ebx, CL
///
/// to:
///     mov rax, 0x10000000f
///     shld [rax], ebx, CL
#[allow(clippy::too_many_arguments)]
fn patch_riprel_op_op<TOperand: Into<EncoderOperand>, TOperand2: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op, op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(op)
        .add_operand(op_2)
        .encode_extend(buf)?;

    Ok(())
}

/// Example patch:
///
/// from:
///     shld [rip + 8], ebx, CL
///
/// to:
///     mov rax, 0x10000000f
///     shld [rax], ebx, CL
#[allow(clippy::too_many_arguments)]
fn patch_op_riprel_op<TOperand: Into<EncoderOperand>, TOperand2: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op, op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(op_2)
        .encode_extend(buf)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn patch_op_op_riprel<TOperand: Into<EncoderOperand>, TOperand2: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op, op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(op_2)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .encode_extend(buf)?;

    Ok(())
}

/// Example patch:
///
/// from:
///     xchg rbx, [rip + 8]
///
/// to:
///     mov rax, 0x10000000f
///     xchg rbx, [rax]
fn patch_op_riprel<TOperand: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> op, [rax]
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .encode_extend(buf)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn patch_op_op_riprel_op<
    TOperand: Into<EncoderOperand>,
    TOperand2: Into<EncoderOperand>,
    TOperand3: Into<EncoderOperand>,
>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
    op_3: TOperand3,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op, op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(op_2)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(op_3)
        .encode_extend(buf)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn patch_op_op_op_riprel<TOperand: Into<EncoderOperand>, TOperand2: Into<EncoderOperand>>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
    op_3: zydis::Register,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> op, op, op, [rax]
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(op_2)
        .add_operand(EncoderOperand::reg_is4(op_3, true))
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .encode_extend(buf)?;

    Ok(())
}

/// Example patch:
///
/// from:
///     vpermil2ps xmm1, xmm2, xmm3, xmmword ptr [rip + 8], 0xa
///
/// to:
///     movabs rax, 0x100000012
///     vpermil2ps xmm1, xmm2, xmm3, xmmword ptr [rax], 0xa
#[allow(clippy::too_many_arguments)]
fn patch_op_op_op_riprel_op<
    TOperand: Into<EncoderOperand>,
    TOperand2: Into<EncoderOperand>,
    TOperand4: Into<EncoderOperand>,
>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
    op_3: zydis::Register,
    op_4: TOperand4,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    // <orig_opcode> [rax], op
    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(op_2)
        .add_operand(EncoderOperand::reg_is4(op_3, true))
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(op_4)
        .encode_extend(buf)?;
    Ok(())
}

/// Example patch:
///
/// from:
///     vpermil2ps xmm1,xmm2,xmmword ptr [rip + 8],xmm4,0xa
///
/// to:
///     movabs rax, 0x100000012
///     vpermil2ps xmm1, xmm2, xmmword ptr [rax], xmm4, 0xa
#[allow(clippy::too_many_arguments)]
fn patch_op_op_riprel_op_op<
    TOperand: Into<EncoderOperand>,
    TOperand2: Into<EncoderOperand>,
    TOperand4: Into<EncoderOperand>,
>(
    instruction: &ZydisInstruction,
    dest_address: &mut usize,
    source_address: &mut usize, // pc (eip/rip)
    scratch_register: x64::Register,
    buf: &mut Vec<u8>,
    riprel: &DecodedOperand,
    op: TOperand,
    op_2: TOperand2,
    op_3: zydis::Register,
    op_4: TOperand4,
) -> Result<(), ZydisRewriterError> {
    let zydis_reg = patch_common(
        instruction,
        source_address,
        riprel,
        scratch_register,
        dest_address,
        buf,
    )?;

    *dest_address += EncoderRequest::new64(instruction.mnemonic)
        .add_operand(op)
        .add_operand(op_2)
        .add_operand(make_mem_operand(zydis_reg, riprel))
        .add_operand(EncoderOperand::reg_is4(op_3, true))
        .add_operand(op_4)
        .encode_extend(buf)?;
    Ok(())
}

/// Encodes the common prologue of `mov <scratch>, <address>` instruction,
/// which is common between all patches.
fn patch_common(
    instruction: &zydis::Instruction<zydis::OperandArrayVec<5>>,
    source_address: &mut usize,
    riprel: &DecodedOperand,
    scratch_register: x64::Register,
    dest_address: &mut usize,
    buf: &mut Vec<u8>,
) -> Result<zydis::Register, ZydisRewriterError> {
    let target_addr = instruction
        .calc_absolute_address(*source_address as u64, riprel)
        .unwrap();

    *source_address += instruction.length as usize;

    let zydis_reg = scratch_register.to_zydis();
    *dest_address += EncoderRequest::new64(MOV)
        .add_operand(zydis_reg)
        .add_operand(target_addr)
        .encode_extend(buf)?;

    Ok(zydis_reg)
}

fn make_mem_operand(zydis_reg: zydis::Register, op: &DecodedOperand) -> EncoderOperand {
    let mut reg = EncoderOperand::ZERO_MEM.clone();
    reg.base = zydis_reg;
    reg.size = op.size / 8;
    EncoderOperand::mem_custom(reg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::util::test_utilities::test_relocate_instruction;
    use crate::x64;
    use rstest::rstest;

    #[rstest]
    // 1 Operands:        [function name]
    // - rip              patch_riprel
    #[case::inc_rip_rel_abs("ff0508000000", 0x100000000, 0, "48b80e00000001000000ff00")]
    // inc dword ptr [rip + 8] -> mov rax, 0x10000000e + inc dword ptr [rax]

    // 2 Operands:       [function name]
    // - rip, reg        patch_riprel_op
    #[case::mov_lhs("48891d08000000", 0x100000000, 0, "48b80f00000001000000488918")]
    // mov [rip + 8], rbx -> mov rax, 0x10000000f + mov [rax], rbx
    // - rip, imm        patch_riprel_op
    #[case::bt_imm8("480fba250800000008", 0x100000000, 0, "48b81100000001000000480fba2008")]
    // bt qword ptr [rip + 8], 8 -> mov rax, 0x100000011 + bt [rax], 8
    // - reg, rip        patch_op_riprel
    #[case::xchg_src("48871d08000000", 0x100000000, 0, "48b80f00000001000000488718")]
    // xchg rbx, [rip + 8] -> mov rax, 0x10000000f + xchg [rax], rbx

    // 3 Operands:       [function name]
    // - rip, reg, imm    patch_riprel_op_op
    #[case::shld_imm8("0fa41d0800000005", 0x100000000, 0, "48b810000000010000000fa41805")]
    // shld [rip + 8], ebx, 5 -> mov rax, 0x100000010 + shld [rax], ebx, 5

    // - rip, reg, reg    patch_riprel_op_op
    #[case::shld_cl("0fa51d08000000", 0x100000000, 0, "48b80f000000010000000fa518")]
    // shld [rip + 8], ebx, CL -> mov rax, 0x10000000f + shld [rax], ebx, CL

    // - reg, rip, imm    patch_op_riprel_op
    #[case::imul_reg_mem_imm("486b1d0800000020", 0x100000000, 0, "48b81000000001000000486b1820")]
    // imul rbx, [rip + 8], 32 -> mov rax, 0x100000010 + imul rbx, qword ptr [rax], 0x20

    // - reg, rip, reg    patch_op_riprel_op
    // Not tested, because covered by already tested patch_op_riprel_op.

    // - reg, reg, rip    patch_op_op_riprel
    #[case::vaddpd_rhs("c5ed580d08000000", 0x100000000, 0, "48b81000000001000000c5ed5808")]
    // vaddpd ymm1, ymm2, [rip + 8] -> mov rax, 0x100000010 + vaddpd ymm1, ymm2, [rax]

    // 4 Operands:
    // - reg, reg, rip, imm patch_op_op_riprel_op
    #[case::vcmpps("c5ecc20d0800000004", 0x100000000, 0, "48b81100000001000000c5ecc20804")]
    // vcmpps ymm1, ymm2, ymmword ptr [rip+0x08], 0x04 -> mov rax, 0x100000011 + vcmpps ymm1, ymm2, ymmword ptr ds:[rax], 0x04
    // - reg, reg, reg, rip patch_op_op_op_riprel
    #[case::vfnmaddpd(
        "c4e3e95c0d0800000030",
        0x100000000,
        0,
        "48b81200000001000000c4e3e95c0830"
    )]
    // vfmaddsubps xmm1, xmm2, xmm3, xmmword ptr [rip + 8] -> movabs rax, 0x100000012 + vfmaddsubps xmm1, xmm2, xmm3, xmmword ptr [rax]

    // 5 Operands:
    #[case::vpermil2ps_2(
        "c4e3e9480d080000003a",
        0x100000000,
        0,
        "48b81200000001000000c4e3e948083a"
    )]
    // vpermil2ps xmm1,xmm2,xmm3,xmmword ptr [rip + 8],0xa -> movabs rax, 0x100000012 + vpermil2ps xmm1,xmm2,xmm3,xmmword ptr [rax],0xa
    #[case::vpermil2ps_1(
        "c4e369480d080000004a",
        0x100000000,
        0,
        "48b81200000001000000c4e36948084a"
    )]
    // vpermil2ps xmm1,xmm2,xmmword ptr [rip + 8],xmm4,0xa -> movabs rax, 0x100000012 + vpermil2ps xmm1, xmm2, xmmword ptr [rax], xmm4, 0xa
    fn relocate_rip_rel_64(
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
            patch_rip_relative_64, // the function being tested
        );
    }
}

#[derive(Debug)]
pub struct ZydisRewriterError {
    err: Status,
}

impl From<Status> for ZydisRewriterError {
    fn from(e: Status) -> Self {
        Self { err: e }
    }
}

impl From<ZydisRewriterError> for CodeRewriterError {
    fn from(e: ZydisRewriterError) -> Self {
        CodeRewriterError::ThirdPartyAssemblerError(e.err.description().into())
    }
}
