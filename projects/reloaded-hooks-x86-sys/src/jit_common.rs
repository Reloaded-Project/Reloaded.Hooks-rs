extern crate alloc;

use crate::jit_conversions_common::{
    convert_to_asm_register32, convert_to_asm_register64, is_allregister_64,
};
use crate::{jit_common::alloc::string::ToString, jit_conversions_common::is_allregister_32};
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
use iced_x86::IcedError;
use reloaded_hooks_portable::api::jit::call_absolute_operation::CallAbsoluteOperation;
use reloaded_hooks_portable::api::jit::call_relative_operation::CallRelativeOperation;
use reloaded_hooks_portable::api::jit::call_rip_relative_operation::CallIpRelativeOperation;
use reloaded_hooks_portable::api::jit::jump_absolute_operation::JumpAbsoluteOperation;
use reloaded_hooks_portable::api::jit::jump_relative_operation::JumpRelativeOperation;
use reloaded_hooks_portable::api::jit::jump_rip_relative_operation::JumpIpRelativeOperation;
use reloaded_hooks_portable::api::jit::{
    compiler::JitError, mov_operation::MovOperation, operation::Operation,
    pop_operation::PopOperation, push_operation::PushOperation,
    push_stack_operation::PushStackOperation, sub_operation::SubOperation,
    xchg_operation::XChgOperation,
};

use crate::all_registers::AllRegisters;

pub(crate) fn encode_instruction(
    assembler: &mut CodeAssembler,
    operation: &Operation<AllRegisters>,
    address: usize,
) -> Result<(), JitError<AllRegisters>> {
    match operation {
        Operation::Mov(x) => encode_mov(assembler, x),
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
    }
}

fn convert_error(e: IcedError) -> JitError<AllRegisters> {
    JitError::ThirdPartyAssemblerError(e.to_string())
}

fn encode_xchg(
    a: &mut CodeAssembler,
    xchg: &XChgOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&xchg.register1) && is_allregister_32(&xchg.register2) {
        a.xchg(
            convert_to_asm_register32(xchg.register1)?,
            convert_to_asm_register32(xchg.register2)?,
        )
    } else if is_allregister_64(&xchg.register1) && is_allregister_64(&xchg.register2) {
        a.xchg(
            convert_to_asm_register64(xchg.register1)?,
            convert_to_asm_register64(xchg.register2)?,
        )
    } else {
        return Err(JitError::InvalidRegisterCombination(
            xchg.register1,
            xchg.register2,
        ));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_pop(
    a: &mut CodeAssembler,
    pop: &PopOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&pop.register) {
        a.pop(convert_to_asm_register32(pop.register)?)
    } else if is_allregister_64(&pop.register) {
        a.pop(convert_to_asm_register64(pop.register)?)
    } else {
        return Err(JitError::InvalidRegister(pop.register));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_sub(
    a: &mut CodeAssembler,
    sub: &SubOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&sub.register) {
        a.sub(convert_to_asm_register32(sub.register)?, sub.operand)
    } else if is_allregister_64(&sub.register) {
        a.sub(convert_to_asm_register64(sub.register)?, sub.operand)
    } else {
        return Err(JitError::InvalidRegister(sub.register));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_push_stack(
    a: &mut CodeAssembler,
    push: &PushStackOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&push.base_register) {
        let ptr = dword_ptr(convert_to_asm_register32(push.base_register)? + push.offset as i32);
        a.push(ptr)
    } else if is_allregister_64(&push.base_register) {
        let ptr = qword_ptr(convert_to_asm_register64(push.base_register)? + push.offset as i32);
        a.push(ptr)
    } else {
        return Err(JitError::InvalidRegister(push.base_register));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_push(
    a: &mut CodeAssembler,
    push: &PushOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&push.register) {
        a.push(convert_to_asm_register32(push.register)?)
    } else if is_allregister_64(&push.register) {
        a.push(convert_to_asm_register64(push.register)?)
    } else {
        return Err(JitError::InvalidRegister(push.register));
    }
    .map_err(convert_error)?;

    Ok(())
}

fn encode_mov(
    a: &mut CodeAssembler,
    mov: &MovOperation<AllRegisters>,
) -> Result<(), JitError<AllRegisters>> {
    if is_allregister_32(&mov.target) && is_allregister_32(&mov.source) {
        a.mov(
            convert_to_asm_register32(mov.target)?,
            convert_to_asm_register32(mov.source)?,
        )
    } else if is_allregister_64(&mov.target) && is_allregister_64(&mov.source) {
        a.mov(
            convert_to_asm_register64(mov.target)?,
            convert_to_asm_register64(mov.source)?,
        )
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
        let target_reg = convert_to_asm_register64(x.scratch_register)?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = convert_to_asm_register32(x.scratch_register)?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.jmp(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            "Non 32/64bit architectures are not supported".to_string(),
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
        let target_reg = convert_to_asm_register64(x.scratch_register)?;
        a.mov(target_reg, x.target_address as u64)
            .map_err(convert_error)?;
        a.call(target_reg).map_err(convert_error)?;
    } else if a.bitness() == 32 {
        let target_reg = convert_to_asm_register32(x.scratch_register)?;
        a.mov(target_reg, x.target_address as u32)
            .map_err(convert_error)?;
        a.call(target_reg).map_err(convert_error)?;
    } else {
        return Err(JitError::ThirdPartyAssemblerError(
            "Non 32/64bit architectures are not supported".to_string(),
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
