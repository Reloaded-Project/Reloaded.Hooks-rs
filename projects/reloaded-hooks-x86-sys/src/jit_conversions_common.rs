use iced_x86::code_asm::{
    AsmRegister32, AsmRegister64, AsmRegisterSt, AsmRegisterXmm, AsmRegisterYmm, AsmRegisterZmm,
};
use reloaded_hooks_portable::api::jit::compiler::JitError;

use crate::all_registers::AllRegisters;

pub(crate) fn is_allregister_32(reg: &AllRegisters) -> bool {
    matches!(
        reg,
        AllRegisters::eax
            | AllRegisters::ebx
            | AllRegisters::ecx
            | AllRegisters::edx
            | AllRegisters::esi
            | AllRegisters::edi
            | AllRegisters::ebp
            | AllRegisters::esp
    )
}

pub(crate) fn is_allregister_64(reg: &AllRegisters) -> bool {
    matches!(
        reg,
        AllRegisters::rax
            | AllRegisters::rbx
            | AllRegisters::rcx
            | AllRegisters::rdx
            | AllRegisters::rsi
            | AllRegisters::rdi
            | AllRegisters::rbp
            | AllRegisters::rsp
            | AllRegisters::r8
            | AllRegisters::r9
            | AllRegisters::r10
            | AllRegisters::r11
            | AllRegisters::r12
            | AllRegisters::r13
            | AllRegisters::r14
            | AllRegisters::r15
    )
}

pub(crate) fn convert_to_asm_register32(
    reg: AllRegisters,
) -> Result<AsmRegister32, JitError<AllRegisters>> {
    match reg {
        AllRegisters::eax => Ok(iced_x86::code_asm::registers::eax),
        AllRegisters::ebx => Ok(iced_x86::code_asm::registers::ebx),
        AllRegisters::ecx => Ok(iced_x86::code_asm::registers::ecx),
        AllRegisters::edx => Ok(iced_x86::code_asm::registers::edx),
        AllRegisters::esi => Ok(iced_x86::code_asm::registers::esi),
        AllRegisters::edi => Ok(iced_x86::code_asm::registers::edi),
        AllRegisters::ebp => Ok(iced_x86::code_asm::registers::ebp),
        AllRegisters::esp => Ok(iced_x86::code_asm::registers::esp),
        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn convert_to_asm_register64(
    reg: AllRegisters,
) -> Result<AsmRegister64, JitError<AllRegisters>> {
    match reg {
        AllRegisters::rax => Ok(iced_x86::code_asm::registers::rax),
        AllRegisters::rbx => Ok(iced_x86::code_asm::registers::rbx),
        AllRegisters::rcx => Ok(iced_x86::code_asm::registers::rcx),
        AllRegisters::rdx => Ok(iced_x86::code_asm::registers::rdx),
        AllRegisters::rsi => Ok(iced_x86::code_asm::registers::rsi),
        AllRegisters::rdi => Ok(iced_x86::code_asm::registers::rdi),
        AllRegisters::rbp => Ok(iced_x86::code_asm::registers::rbp),
        AllRegisters::rsp => Ok(iced_x86::code_asm::registers::rsp),
        AllRegisters::r8 => Ok(iced_x86::code_asm::registers::r8),
        AllRegisters::r9 => Ok(iced_x86::code_asm::registers::r9),
        AllRegisters::r10 => Ok(iced_x86::code_asm::registers::r10),
        AllRegisters::r11 => Ok(iced_x86::code_asm::registers::r11),
        AllRegisters::r12 => Ok(iced_x86::code_asm::registers::r12),
        AllRegisters::r13 => Ok(iced_x86::code_asm::registers::r13),
        AllRegisters::r14 => Ok(iced_x86::code_asm::registers::r14),
        AllRegisters::r15 => Ok(iced_x86::code_asm::registers::r15),

        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn convert_to_asm_register_st(
    reg: AllRegisters,
) -> Result<AsmRegisterSt, JitError<AllRegisters>> {
    match reg {
        AllRegisters::st0 => Ok(iced_x86::code_asm::registers::st0),
        AllRegisters::st1 => Ok(iced_x86::code_asm::registers::st1),
        AllRegisters::st2 => Ok(iced_x86::code_asm::registers::st2),
        AllRegisters::st3 => Ok(iced_x86::code_asm::registers::st3),
        AllRegisters::st4 => Ok(iced_x86::code_asm::registers::st4),
        AllRegisters::st5 => Ok(iced_x86::code_asm::registers::st5),
        AllRegisters::st6 => Ok(iced_x86::code_asm::registers::st6),
        AllRegisters::st7 => Ok(iced_x86::code_asm::registers::st7),
        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn convert_to_asm_register_xmm(
    reg: AllRegisters,
) -> Result<AsmRegisterXmm, JitError<AllRegisters>> {
    match reg {
        AllRegisters::xmm0 => Ok(iced_x86::code_asm::registers::xmm0),
        AllRegisters::xmm1 => Ok(iced_x86::code_asm::registers::xmm1),
        AllRegisters::xmm2 => Ok(iced_x86::code_asm::registers::xmm2),
        AllRegisters::xmm3 => Ok(iced_x86::code_asm::registers::xmm3),
        AllRegisters::xmm4 => Ok(iced_x86::code_asm::registers::xmm4),
        AllRegisters::xmm5 => Ok(iced_x86::code_asm::registers::xmm5),
        AllRegisters::xmm6 => Ok(iced_x86::code_asm::registers::xmm6),
        AllRegisters::xmm7 => Ok(iced_x86::code_asm::registers::xmm7),
        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn convert_to_asm_register_ymm(
    reg: AllRegisters,
) -> Result<AsmRegisterYmm, JitError<AllRegisters>> {
    match reg {
        AllRegisters::ymm0 => Ok(iced_x86::code_asm::registers::ymm0),
        AllRegisters::ymm1 => Ok(iced_x86::code_asm::registers::ymm1),
        AllRegisters::ymm2 => Ok(iced_x86::code_asm::registers::ymm2),
        AllRegisters::ymm3 => Ok(iced_x86::code_asm::registers::ymm3),
        AllRegisters::ymm4 => Ok(iced_x86::code_asm::registers::ymm4),
        AllRegisters::ymm5 => Ok(iced_x86::code_asm::registers::ymm5),
        AllRegisters::ymm6 => Ok(iced_x86::code_asm::registers::ymm6),
        AllRegisters::ymm7 => Ok(iced_x86::code_asm::registers::ymm7),
        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn convert_to_asm_register_zmm(
    reg: AllRegisters,
) -> Result<AsmRegisterZmm, JitError<AllRegisters>> {
    match reg {
        AllRegisters::zmm0 => Ok(iced_x86::code_asm::registers::zmm0),
        AllRegisters::zmm1 => Ok(iced_x86::code_asm::registers::zmm1),
        AllRegisters::zmm2 => Ok(iced_x86::code_asm::registers::zmm2),
        AllRegisters::zmm3 => Ok(iced_x86::code_asm::registers::zmm3),
        AllRegisters::zmm4 => Ok(iced_x86::code_asm::registers::zmm4),
        AllRegisters::zmm5 => Ok(iced_x86::code_asm::registers::zmm5),
        AllRegisters::zmm6 => Ok(iced_x86::code_asm::registers::zmm6),
        AllRegisters::zmm7 => Ok(iced_x86::code_asm::registers::zmm7),
        _ => Err(JitError::InvalidRegister(reg)),
    }
}

pub(crate) fn map_register_x86_to_allregisters(reg: crate::x86::Register) -> AllRegisters {
    match reg {
        crate::x86::Register::eax => AllRegisters::eax,
        crate::x86::Register::ebx => AllRegisters::ebx,
        crate::x86::Register::ecx => AllRegisters::ecx,
        crate::x86::Register::edx => AllRegisters::edx,
        crate::x86::Register::esi => AllRegisters::esi,
        crate::x86::Register::edi => AllRegisters::edi,
        crate::x86::Register::ebp => AllRegisters::ebp,
        crate::x86::Register::esp => AllRegisters::esp,

        crate::x86::Register::st0 => AllRegisters::st0,
        crate::x86::Register::st1 => AllRegisters::st1,
        crate::x86::Register::st2 => AllRegisters::st2,
        crate::x86::Register::st3 => AllRegisters::st3,
        crate::x86::Register::st4 => AllRegisters::st4,
        crate::x86::Register::st5 => AllRegisters::st5,
        crate::x86::Register::st6 => AllRegisters::st6,
        crate::x86::Register::st7 => AllRegisters::st7,

        crate::x86::Register::xmm0 => AllRegisters::xmm0,
        crate::x86::Register::xmm1 => AllRegisters::xmm1,
        crate::x86::Register::xmm2 => AllRegisters::xmm2,
        crate::x86::Register::xmm3 => AllRegisters::xmm3,
        crate::x86::Register::xmm4 => AllRegisters::xmm4,
        crate::x86::Register::xmm5 => AllRegisters::xmm5,
        crate::x86::Register::xmm6 => AllRegisters::xmm6,
        crate::x86::Register::xmm7 => AllRegisters::xmm7,

        crate::x86::Register::ymm0 => AllRegisters::ymm0,
        crate::x86::Register::ymm1 => AllRegisters::ymm1,
        crate::x86::Register::ymm2 => AllRegisters::ymm2,
        crate::x86::Register::ymm3 => AllRegisters::ymm3,
        crate::x86::Register::ymm4 => AllRegisters::ymm4,
        crate::x86::Register::ymm5 => AllRegisters::ymm5,
        crate::x86::Register::ymm6 => AllRegisters::ymm6,
        crate::x86::Register::ymm7 => AllRegisters::ymm7,

        crate::x86::Register::zmm0 => AllRegisters::zmm0,
        crate::x86::Register::zmm1 => AllRegisters::zmm1,
        crate::x86::Register::zmm2 => AllRegisters::zmm2,
        crate::x86::Register::zmm3 => AllRegisters::zmm3,
        crate::x86::Register::zmm4 => AllRegisters::zmm4,
        crate::x86::Register::zmm5 => AllRegisters::zmm5,
        crate::x86::Register::zmm6 => AllRegisters::zmm6,
        crate::x86::Register::zmm7 => AllRegisters::zmm7,
    }
}

pub(crate) fn map_register_x64_to_allregisters(reg: crate::x64::Register) -> AllRegisters {
    match reg {
        crate::x64::Register::rax => AllRegisters::rax,
        crate::x64::Register::rbx => AllRegisters::rbx,
        crate::x64::Register::rcx => AllRegisters::rcx,
        crate::x64::Register::rdx => AllRegisters::rdx,
        crate::x64::Register::rsi => AllRegisters::rsi,
        crate::x64::Register::rdi => AllRegisters::rdi,
        crate::x64::Register::rbp => AllRegisters::rbp,
        crate::x64::Register::rsp => AllRegisters::rsp,

        crate::x64::Register::r8 => AllRegisters::r8,
        crate::x64::Register::r9 => AllRegisters::r9,
        crate::x64::Register::r10 => AllRegisters::r10,
        crate::x64::Register::r11 => AllRegisters::r11,
        crate::x64::Register::r12 => AllRegisters::r12,
        crate::x64::Register::r13 => AllRegisters::r13,
        crate::x64::Register::r14 => AllRegisters::r14,
        crate::x64::Register::r15 => AllRegisters::r15,

        crate::x64::Register::st0 => AllRegisters::st0,
        crate::x64::Register::st1 => AllRegisters::st1,
        crate::x64::Register::st2 => AllRegisters::st2,
        crate::x64::Register::st3 => AllRegisters::st3,
        crate::x64::Register::st4 => AllRegisters::st4,
        crate::x64::Register::st5 => AllRegisters::st5,
        crate::x64::Register::st6 => AllRegisters::st6,
        crate::x64::Register::st7 => AllRegisters::st7,

        crate::x64::Register::xmm0 => AllRegisters::xmm0,
        crate::x64::Register::xmm1 => AllRegisters::xmm1,
        crate::x64::Register::xmm2 => AllRegisters::xmm2,
        crate::x64::Register::xmm3 => AllRegisters::xmm3,
        crate::x64::Register::xmm4 => AllRegisters::xmm4,
        crate::x64::Register::xmm5 => AllRegisters::xmm5,
        crate::x64::Register::xmm6 => AllRegisters::xmm6,
        crate::x64::Register::xmm7 => AllRegisters::xmm7,
        crate::x64::Register::xmm8 => AllRegisters::xmm8,
        crate::x64::Register::xmm9 => AllRegisters::xmm9,
        crate::x64::Register::xmm10 => AllRegisters::xmm10,
        crate::x64::Register::xmm11 => AllRegisters::xmm11,
        crate::x64::Register::xmm12 => AllRegisters::xmm12,
        crate::x64::Register::xmm13 => AllRegisters::xmm13,
        crate::x64::Register::xmm14 => AllRegisters::xmm14,
        crate::x64::Register::xmm15 => AllRegisters::xmm15,

        crate::x64::Register::ymm0 => AllRegisters::ymm0,
        crate::x64::Register::ymm1 => AllRegisters::ymm1,
        crate::x64::Register::ymm2 => AllRegisters::ymm2,
        crate::x64::Register::ymm3 => AllRegisters::ymm3,
        crate::x64::Register::ymm4 => AllRegisters::ymm4,
        crate::x64::Register::ymm5 => AllRegisters::ymm5,
        crate::x64::Register::ymm6 => AllRegisters::ymm6,
        crate::x64::Register::ymm7 => AllRegisters::ymm7,
        crate::x64::Register::ymm8 => AllRegisters::ymm8,
        crate::x64::Register::ymm9 => AllRegisters::ymm9,
        crate::x64::Register::ymm10 => AllRegisters::ymm10,
        crate::x64::Register::ymm11 => AllRegisters::ymm11,
        crate::x64::Register::ymm12 => AllRegisters::ymm12,
        crate::x64::Register::ymm13 => AllRegisters::ymm13,
        crate::x64::Register::ymm14 => AllRegisters::ymm14,
        crate::x64::Register::ymm15 => AllRegisters::ymm15,

        crate::x64::Register::zmm0 => AllRegisters::zmm0,
        crate::x64::Register::zmm1 => AllRegisters::zmm1,
        crate::x64::Register::zmm2 => AllRegisters::zmm2,
        crate::x64::Register::zmm3 => AllRegisters::zmm3,
        crate::x64::Register::zmm4 => AllRegisters::zmm4,
        crate::x64::Register::zmm5 => AllRegisters::zmm5,
        crate::x64::Register::zmm6 => AllRegisters::zmm6,
        crate::x64::Register::zmm7 => AllRegisters::zmm7,
        crate::x64::Register::zmm8 => AllRegisters::zmm8,
        crate::x64::Register::zmm9 => AllRegisters::zmm9,
        crate::x64::Register::zmm10 => AllRegisters::zmm10,
        crate::x64::Register::zmm11 => AllRegisters::zmm11,
        crate::x64::Register::zmm12 => AllRegisters::zmm12,
        crate::x64::Register::zmm13 => AllRegisters::zmm13,
        crate::x64::Register::zmm14 => AllRegisters::zmm14,
        crate::x64::Register::zmm15 => AllRegisters::zmm15,
    }
}

pub(crate) fn map_allregisters_to_x86(reg: AllRegisters) -> crate::x86::Register {
    match reg {
        AllRegisters::eax => crate::x86::Register::eax,
        AllRegisters::ebx => crate::x86::Register::ebx,
        AllRegisters::ecx => crate::x86::Register::ecx,
        AllRegisters::edx => crate::x86::Register::edx,
        AllRegisters::esi => crate::x86::Register::esi,
        AllRegisters::edi => crate::x86::Register::edi,
        AllRegisters::ebp => crate::x86::Register::ebp,
        AllRegisters::esp => crate::x86::Register::esp,
        AllRegisters::st0 => crate::x86::Register::st0,
        AllRegisters::st1 => crate::x86::Register::st1,
        AllRegisters::st2 => crate::x86::Register::st2,
        AllRegisters::st3 => crate::x86::Register::st3,
        AllRegisters::st4 => crate::x86::Register::st4,
        AllRegisters::st5 => crate::x86::Register::st5,
        AllRegisters::st6 => crate::x86::Register::st6,
        AllRegisters::st7 => crate::x86::Register::st7,
        AllRegisters::xmm0 => crate::x86::Register::xmm0,
        AllRegisters::xmm1 => crate::x86::Register::xmm1,
        AllRegisters::xmm2 => crate::x86::Register::xmm2,
        AllRegisters::xmm3 => crate::x86::Register::xmm3,
        AllRegisters::xmm4 => crate::x86::Register::xmm4,
        AllRegisters::xmm5 => crate::x86::Register::xmm5,
        AllRegisters::xmm6 => crate::x86::Register::xmm6,
        AllRegisters::xmm7 => crate::x86::Register::xmm7,
        AllRegisters::ymm0 => crate::x86::Register::ymm0,
        AllRegisters::ymm1 => crate::x86::Register::ymm1,
        AllRegisters::ymm2 => crate::x86::Register::ymm2,
        AllRegisters::ymm3 => crate::x86::Register::ymm3,
        AllRegisters::ymm4 => crate::x86::Register::ymm4,
        AllRegisters::ymm5 => crate::x86::Register::ymm5,
        AllRegisters::ymm6 => crate::x86::Register::ymm6,
        AllRegisters::ymm7 => crate::x86::Register::ymm7,
        AllRegisters::zmm0 => crate::x86::Register::zmm0,
        AllRegisters::zmm1 => crate::x86::Register::zmm1,
        AllRegisters::zmm2 => crate::x86::Register::zmm2,
        AllRegisters::zmm3 => crate::x86::Register::zmm3,
        AllRegisters::zmm4 => crate::x86::Register::zmm4,
        AllRegisters::zmm5 => crate::x86::Register::zmm5,
        AllRegisters::zmm6 => crate::x86::Register::zmm6,
        AllRegisters::zmm7 => crate::x86::Register::zmm7,
        _ => todo!(),
    }
}

pub(crate) fn map_allregisters_to_x64(reg: AllRegisters) -> crate::x64::Register {
    match reg {
        AllRegisters::rax => crate::x64::Register::rax,
        AllRegisters::rbx => crate::x64::Register::rbx,
        AllRegisters::rcx => crate::x64::Register::rcx,
        AllRegisters::rdx => crate::x64::Register::rdx,
        AllRegisters::rsi => crate::x64::Register::rsi,
        AllRegisters::rdi => crate::x64::Register::rdi,
        AllRegisters::rbp => crate::x64::Register::rbp,
        AllRegisters::rsp => crate::x64::Register::rsp,
        AllRegisters::r8 => crate::x64::Register::r8,
        AllRegisters::r9 => crate::x64::Register::r9,
        AllRegisters::r10 => crate::x64::Register::r10,
        AllRegisters::r11 => crate::x64::Register::r11,
        AllRegisters::r12 => crate::x64::Register::r12,
        AllRegisters::r13 => crate::x64::Register::r13,
        AllRegisters::r14 => crate::x64::Register::r14,
        AllRegisters::r15 => crate::x64::Register::r15,
        AllRegisters::st0 => crate::x64::Register::st0,
        AllRegisters::st1 => crate::x64::Register::st1,
        AllRegisters::st2 => crate::x64::Register::st2,
        AllRegisters::st3 => crate::x64::Register::st3,
        AllRegisters::st4 => crate::x64::Register::st4,
        AllRegisters::st5 => crate::x64::Register::st5,
        AllRegisters::st6 => crate::x64::Register::st6,
        AllRegisters::st7 => crate::x64::Register::st7,
        AllRegisters::xmm0 => crate::x64::Register::xmm0,
        AllRegisters::xmm1 => crate::x64::Register::xmm1,
        AllRegisters::xmm2 => crate::x64::Register::xmm2,
        AllRegisters::xmm3 => crate::x64::Register::xmm3,
        AllRegisters::xmm4 => crate::x64::Register::xmm4,
        AllRegisters::xmm5 => crate::x64::Register::xmm5,
        AllRegisters::xmm6 => crate::x64::Register::xmm6,
        AllRegisters::xmm7 => crate::x64::Register::xmm7,
        AllRegisters::xmm8 => crate::x64::Register::xmm8,
        AllRegisters::xmm9 => crate::x64::Register::xmm9,
        AllRegisters::xmm10 => crate::x64::Register::xmm10,
        AllRegisters::xmm11 => crate::x64::Register::xmm11,
        AllRegisters::xmm12 => crate::x64::Register::xmm12,
        AllRegisters::xmm13 => crate::x64::Register::xmm13,
        AllRegisters::xmm14 => crate::x64::Register::xmm14,
        AllRegisters::xmm15 => crate::x64::Register::xmm15,
        AllRegisters::ymm0 => crate::x64::Register::ymm0,
        AllRegisters::ymm1 => crate::x64::Register::ymm1,
        AllRegisters::ymm2 => crate::x64::Register::ymm2,
        AllRegisters::ymm3 => crate::x64::Register::ymm3,
        AllRegisters::ymm4 => crate::x64::Register::ymm4,
        AllRegisters::ymm5 => crate::x64::Register::ymm5,
        AllRegisters::ymm6 => crate::x64::Register::ymm6,
        AllRegisters::ymm7 => crate::x64::Register::ymm7,
        AllRegisters::ymm8 => crate::x64::Register::ymm8,
        AllRegisters::ymm9 => crate::x64::Register::ymm9,
        AllRegisters::ymm10 => crate::x64::Register::ymm10,
        AllRegisters::ymm11 => crate::x64::Register::ymm11,
        AllRegisters::ymm12 => crate::x64::Register::ymm12,
        AllRegisters::ymm13 => crate::x64::Register::ymm13,
        AllRegisters::ymm14 => crate::x64::Register::ymm14,
        AllRegisters::ymm15 => crate::x64::Register::ymm15,
        AllRegisters::zmm0 => crate::x64::Register::zmm0,
        AllRegisters::zmm1 => crate::x64::Register::zmm1,
        AllRegisters::zmm2 => crate::x64::Register::zmm2,
        AllRegisters::zmm3 => crate::x64::Register::zmm3,
        AllRegisters::zmm4 => crate::x64::Register::zmm4,
        AllRegisters::zmm5 => crate::x64::Register::zmm5,
        AllRegisters::zmm6 => crate::x64::Register::zmm6,
        AllRegisters::zmm7 => crate::x64::Register::zmm7,
        AllRegisters::zmm8 => crate::x64::Register::zmm8,
        AllRegisters::zmm9 => crate::x64::Register::zmm9,
        AllRegisters::zmm10 => crate::x64::Register::zmm10,
        AllRegisters::zmm11 => crate::x64::Register::zmm11,
        AllRegisters::zmm12 => crate::x64::Register::zmm12,
        AllRegisters::zmm13 => crate::x64::Register::zmm13,
        AllRegisters::zmm14 => crate::x64::Register::zmm14,
        AllRegisters::zmm15 => crate::x64::Register::zmm15,
        _ => todo!(),
    }
}
