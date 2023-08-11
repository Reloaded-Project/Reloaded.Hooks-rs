extern crate alloc;

use crate::jit_common::alloc::string::ToString;
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use iced_x86::IcedError;
use reloaded_hooks_portable::api::jit::call_absolute_operation::CallAbsoluteOperation;
use reloaded_hooks_portable::api::jit::call_relative_operation::CallRelativeOperation;
use reloaded_hooks_portable::api::jit::call_rip_relative_operation::CallIpRelativeOperation;
use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
use reloaded_hooks_portable::api::jit::jump_rip_relative_operation::JumpIpRelativeOperation;
use reloaded_hooks_portable::api::jit::mov_from_stack_operation::MovFromStackOperation;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, mov_operation::MovOperation, operation::Operation,
    pop_operation::PopOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, sub_operation::SubOperation,
    xchg_operation::XChgOperation,
};

use crate::all_registers::AllRegisters;
use iced_x86::code_asm::registers as iced_regs;

const ARCH_NOT_SUPPORTED: &str = "Non 32/64bit architectures are not supported";

pub(crate) fn encode_instruction(
    assembler: &mut CodeAssembler,
    operation: &Operation<AllRegisters>,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    match operation {
        Operation::Mov(x) => encode_mov(assembler, x),
        Operation::MovFromStack(x) => encode_mov_from_stack(assembler, x),
        Operation::Push(x) => encode_push(assembler, x),
        Operation::PushStack(x) => encode_push_stack(assembler, x),
        Operation::Sub(x) => encode_sub(assembler, x),
        Operation::Pop(x) => encode_pop(assembler, x),
        Operation::Xchg(x) => encode_xchg(assembler, x),
        Operation::CallRelative(x) => encode_call_relative(assembler, x),
        Operation::CallAbsolute(x) => encode_call_absolute(assembler, x),
        Operation::JumpRelative(x) => encode_jump_relative(assembler, x),
        Operation::JumpAbsolute(x) => encode_jump_absolute(assembler, x),

        // x64 only
        Operation::CallIpRelative(x) => encode_call_ip_relative(assembler, x, address),
        Operation::JumpIpRelative(x) => encode_jump_ip_relative(assembler, x, address),

        // Optimised Functions
        Operation::MultiPush(x) => encode_multi_push(assembler, x),
        Operation::MultiPop(x) => encode_multi_pop(assembler, x),
    }
}

macro_rules! multi_push_item {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        match $a.bitness() {
            32 => {
                $a.$op(dword_ptr(iced_regs::esp) + $offset, $reg.$convert_method()?)
                    .map_err(convert_error)?;
            }
            64 => {
                $a.$op(dword_ptr(iced_regs::rsp) + $offset, $reg.$convert_method()?)
                    .map_err(convert_error)?;
            }
            _ => {
                return Err(JitError::ThirdPartyAssemblerError(
                    ARCH_NOT_SUPPORTED.to_string(),
                ));
            }
        }
    };
}

fn encode_multi_push(
    a: &mut CodeAssembler,
    ops: &[PushOperation<AllRegisters>],
) -> Result<(), JitError<AllRegisters>> {
    // Calculate space to reserve.
    let mut space_needed = 0;

    for x in ops {
        space_needed += x.register.size();
    }

    // Reserve the space.
    a.sub(iced_regs::esp, space_needed as i32)
        .map_err(convert_error)?;

    // Push the items.
    let mut current_offset = 0;
    for x in ops.iter().rev() {
        // Loop through the operations in reverse
        if x.register.is_32() {
            multi_push_item!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() {
            multi_push_item!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            multi_push_item!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            multi_push_item!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            multi_push_item!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    Ok(())
}

macro_rules! multi_pop_item {
    ($a:expr, $reg:expr, $offset:expr, $convert_method:ident, $op:ident) => {
        match $a.bitness() {
            32 => {
                $a.$op($reg.$convert_method()?, dword_ptr(iced_regs::esp) + $offset)
                    .map_err(convert_error)?;
            }
            64 => {
                $a.$op($reg.$convert_method()?, dword_ptr(iced_regs::rsp) + $offset)
                    .map_err(convert_error)?;
            }
            _ => {
                return Err(JitError::ThirdPartyAssemblerError(
                    ARCH_NOT_SUPPORTED.to_string(),
                ));
            }
        }
    };
}

fn encode_multi_pop(
    a: &mut CodeAssembler,
    ops: &[PopOperation<AllRegisters>],
) -> Result<(), JitError<AllRegisters>> {
    // Note: It is important that we do MOV in ascending address order, to help CPU caching :wink:

    // Start from the top of the reserved space.
    let mut current_offset = 0;
    for x in ops {
        if x.register.is_32() {
            multi_pop_item!(a, x.register, current_offset, as_iced_32, mov);
        } else if x.register.is_64() {
            multi_pop_item!(a, x.register, current_offset, as_iced_64, mov);
        } else if x.register.is_xmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_xmm, movdqu);
        } else if x.register.is_ymm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_ymm, vmovdqu);
        } else if x.register.is_zmm() {
            multi_pop_item!(a, x.register, current_offset, as_iced_zmm, vmovdqu8);
        } else {
            return Err(JitError::InvalidRegister(x.register));
        }

        // Move to the next offset.
        current_offset += x.register.size();
    }

    // Release the space.
    let total_space = ops.iter().map(|x| x.register.size()).sum::<usize>();
    a.add(iced_regs::esp, total_space as i32)
        .map_err(convert_error)?;

    Ok(())
}

fn convert_error(e: IcedError) -> JitError<AllRegisters> {
    JitError::ThirdPartyAssemblerError(e.to_string())
}

fn encode_xchg(
    a: &mut CodeAssembler,
    xchg: &XChgOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if xchg.register1.is_32() && xchg.register2.is_32() {
        a.xchg(xchg.register1.as_iced_32()?, xchg.register2.as_iced_32()?)
    } else if xchg.register1.is_64() && xchg.register2.is_64() {
        a.xchg(xchg.register1.as_iced_64()?, xchg.register2.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(
            xchg.register1,
            xchg.register2,
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}

macro_rules! encode_xmm_pop {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 {
            $a.$op($reg.$reg_type()?, dword_ptr(iced_regs::esp))
                .map_err(convert_error)?;
            $a.add(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 {
            $a.$op($reg.$reg_type()?, dword_ptr(iced_regs::rsp))
                .map_err(convert_error)?;
            $a.add(iced_regs::rsp, $reg.size() as i32)
                .map_err(convert_error)?;
        } else {
            return Err(JitError::ThirdPartyAssemblerError(
                ARCH_NOT_SUPPORTED.to_string(),
            ));
        }
    };
}

fn encode_pop(
    a: &mut CodeAssembler,
    pop: &PopOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if pop.register.is_32() {
        a.pop(pop.register.as_iced_32()?).map_err(convert_error)?;
    } else if pop.register.is_64() {
        a.pop(pop.register.as_iced_64()?).map_err(convert_error)?;
    } else if pop.register.is_xmm() {
        encode_xmm_pop!(a, pop.register, as_iced_xmm, movdqu);
    } else if pop.register.is_ymm() {
        encode_xmm_pop!(a, pop.register, as_iced_ymm, vmovdqu);
    } else if pop.register.is_zmm() {
        encode_xmm_pop!(a, pop.register, as_iced_zmm, vmovdqu8);
    } else {
        return Err(JitError::InvalidRegister(pop.register));
    }

    Ok(())
}

fn encode_sub(
    a: &mut CodeAssembler,
    sub: &SubOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if sub.register.is_32() {
        a.sub(sub.register.as_iced_32()?, sub.operand)
    } else if sub.register.is_64() {
        a.sub(sub.register.as_iced_64()?, sub.operand)
    } else {
        return Err(JitError::InvalidRegister(sub.register));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_mov_from_stack(
    a: &mut CodeAssembler,
    x: &MovFromStackOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        let ptr = dword_ptr(iced_x86::Register::ESP) + x.stack_offset;
        a.mov(x.target.as_iced_32()?, ptr)
    } else if a.bitness() == 64 {
        let ptr = qword_ptr(iced_x86::Register::RSP) + x.stack_offset;
        a.mov(x.target.as_iced_64()?, ptr)
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_push_stack(
    a: &mut CodeAssembler,
    push: &PushStackOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if push.base_register.is_32() {
        if push.item_size != 4 {
            return Err(JitError::ThirdPartyAssemblerError(
                "Pushing float registers not implemented right now.".to_string(),
            ));
        }

        let ptr = dword_ptr(push.base_register.as_iced_32()?) + push.offset as i32;
        a.push(ptr)
    } else if push.base_register.is_64() {
        if push.item_size != 8 {
            return Err(JitError::ThirdPartyAssemblerError(
                "Pushing float registers not implemented right now.".to_string(),
            ));
        }

        let ptr = qword_ptr(push.base_register.as_iced_64()?) + push.offset as i32;
        a.push(ptr)
    } else {
        return Err(JitError::InvalidRegister(push.base_register));
    }
    .map_err(convert_error)?;

    Ok(())
}

macro_rules! encode_xmm_push {
    ($a:expr, $reg:expr, $reg_type:ident, $op:ident) => {
        if $a.bitness() == 32 {
            $a.sub(iced_regs::esp, $reg.size() as i32)
                .map_err(convert_error)?;
            $a.$op(dword_ptr(iced_regs::esp), $reg.$reg_type()?)
                .map_err(convert_error)?;
        } else if $a.bitness() == 64 {
            $a.sub(iced_regs::rsp, $reg.size() as i32)
                .map_err(convert_error)?;
            $a.$op(dword_ptr(iced_regs::rsp), $reg.$reg_type()?)
                .map_err(convert_error)?;
        } else {
            return Err(JitError::ThirdPartyAssemblerError(
                ARCH_NOT_SUPPORTED.to_string(),
            ));
        }
    };
}

fn encode_push(
    a: &mut CodeAssembler,
    push: &PushOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if push.register.is_32() {
        a.push(push.register.as_iced_32()?).map_err(convert_error)?;
    } else if push.register.is_64() {
        a.push(push.register.as_iced_64()?).map_err(convert_error)?;
    } else if push.register.is_xmm() {
        encode_xmm_push!(a, push.register, as_iced_xmm, movdqu);
    } else if push.register.is_ymm() {
        encode_xmm_push!(a, push.register, as_iced_ymm, vmovdqu);
    } else if push.register.is_zmm() {
        encode_xmm_push!(a, push.register, as_iced_zmm, vmovdqu8);
    } else {
        return Err(JitError::InvalidRegister(push.register));
    }

    Ok(())
}

fn encode_mov(
    a: &mut CodeAssembler,
    mov: &MovOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if mov.target.is_32() && mov.source.is_32() {
        a.mov(mov.target.as_iced_32()?, mov.source.as_iced_32()?)
    } else if mov.target.is_64() && mov.source.is_64() {
        a.mov(mov.target.as_iced_64()?, mov.source.as_iced_64()?)
    } else {
        return Err(JitError::InvalidRegisterCombination(mov.source, mov.target));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_jump_relative(
    a: &mut CodeAssembler,
    x: &JumpRelativeOperation,
) -> Result<(), JitError<AllRegisters>> {
    a.jmp(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}

fn encode_jump_absolute(
    a: &mut CodeAssembler,
    x: &JumpAbsoluteOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 64 {
        let target_reg = x.scratch_register.as_iced_64()?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = x.scratch_register.as_iced_32()?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }

    Ok(())
}

fn encode_call_relative(
    a: &mut CodeAssembler,
    x: &CallRelativeOperation,
) -> Result<(), JitError<AllRegisters>> {
    a.call(x.target_address as u64).map_err(convert_error)?;
    Ok(())
}

fn encode_call_absolute(
    a: &mut CodeAssembler,
    x: &CallAbsoluteOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 64 {
        let target_reg = x.scratch_register.as_iced_64()?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.call(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = x.scratch_register.as_iced_32()?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.call(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            ARCH_NOT_SUPPORTED.to_string(),
        ));
    }

    Ok(())
}

fn encode_jump_ip_relative(
    a: &mut CodeAssembler,
    x: &JumpIpRelativeOperation,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        return Err(JitError::ThirdPartyAssemblerError(
            "Jump IP Relative is only Supported on 64-bit!".to_string(),
        ));
    }

    let isns = a.instructions();
    let current_ip = if !isns.is_empty() {
        isns.last().unwrap().next_ip()
    } else {
        address as u64
    };

    let relative_offset = x.target_address.wrapping_sub(current_ip as usize);
    a.jmp(qword_ptr(iced_x86::Register::RIP) + relative_offset as i32)
        .map_err(convert_error)?;
    Ok(())
}

fn encode_call_ip_relative(
    a: &mut CodeAssembler,
    x: &CallIpRelativeOperation,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    if a.bitness() == 32 {
        return Err(JitError::ThirdPartyAssemblerError(
            "Call IP Relative is only Supported on 64-bit!".to_string(),
        ));
    }

    let isns = a.instructions();
    let current_ip = if !isns.is_empty() {
        isns.last().unwrap().next_ip()
    } else {
        address as u64
    };

    let relative_offset = x.target_address.wrapping_sub(current_ip as usize);
    a.call(qword_ptr(iced_x86::Register::RIP) + relative_offset as i32)
        .map_err(convert_error)?;
    Ok(())
}
