extern crate alloc;

use crate::jit_conversions_common::{
    convert_to_asm_register32, convert_to_asm_register64, is_allregister_64,
};
use crate::{jit_common::alloc::string::ToString, jit_conversions_common::is_allregister_32};
use iced_x86::code_asm::{dword_ptr, qword_ptr, CodeAssembler};
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
) -> Result<(), JitError<AllRegisters>> {
    match operation {
        Operation::Mov(x) => encode_mov(assembler, x),
        Operation::Push(x) => encode_push(assembler, x),
        Operation::PushStack(x) => encode_push_stack(assembler, x),
        Operation::Sub(x) => encode_sub(assembler, x),
        Operation::Pop(x) => encode_pop(assembler, x),
        Operation::Xchg(x) => encode_xchg(assembler, x),
    }
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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

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
    .map_err(|e| JitError::ThirdPartyAssemblerError(e.to_string()))?;

    Ok(())
}
