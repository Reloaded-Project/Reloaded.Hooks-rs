use iced_x86::code_asm::{
    AsmRegister32, AsmRegister64, AsmRegisterSt, AsmRegisterXmm, AsmRegisterYmm, AsmRegisterZmm,
};
use reloaded_hooks_portable::api::jit::compiler::JitError;

/// Defines a full-size register, accommodating both x86 and x64 architectures.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AllRegisters {
    // General purpose registers for x86:
    eax,
    ebx,
    ecx,
    edx,
    esi,
    edi,
    ebp,
    esp,

    // General purpose registers for x64:
    rax,
    rbx,
    rcx,
    rdx,
    rsi,
    rdi,
    rbp,
    rsp,
    r8,
    r9,
    r10,
    r11,
    r12,
    r13,
    r14,
    r15,

    // x87 Floating-point stack registers (common to both x86 and x64):
    st0,
    st1,
    st2,
    st3,
    st4,
    st5,
    st6,
    st7,

    // SSE (Streaming SIMD Extensions) 128-bit registers (extended for x64):
    xmm0,
    xmm1,
    xmm2,
    xmm3,
    xmm4,
    xmm5,
    xmm6,
    xmm7,
    xmm8,
    xmm9,
    xmm10,
    xmm11,
    xmm12,
    xmm13,
    xmm14,
    xmm15,

    // AVX (Advanced Vector Extensions) 256-bit registers (extended for x64):
    ymm0,
    ymm1,
    ymm2,
    ymm3,
    ymm4,
    ymm5,
    ymm6,
    ymm7,
    ymm8,
    ymm9,
    ymm10,
    ymm11,
    ymm12,
    ymm13,
    ymm14,
    ymm15,

    // AVX-512 512-bit registers (extended for x64):
    zmm0,
    zmm1,
    zmm2,
    zmm3,
    zmm4,
    zmm5,
    zmm6,
    zmm7,
    zmm8,
    zmm9,
    zmm10,
    zmm11,
    zmm12,
    zmm13,
    zmm14,
    zmm15,
}

impl AllRegisters {
    pub fn size(&self) -> usize {
        match *self {
            // General purpose registers for x86:
            AllRegisters::eax
            | AllRegisters::ebx
            | AllRegisters::ecx
            | AllRegisters::edx
            | AllRegisters::esi
            | AllRegisters::edi
            | AllRegisters::ebp
            | AllRegisters::esp => 4, // 32 bits

            // General purpose registers for x64:
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
            | AllRegisters::r15 => 8, // 64 bits

            // x87 Floating-point stack registers (common to both x86 and x64):
            AllRegisters::st0
            | AllRegisters::st1
            | AllRegisters::st2
            | AllRegisters::st3
            | AllRegisters::st4
            | AllRegisters::st5
            | AllRegisters::st6
            | AllRegisters::st7 => 10, // 80 bits

            // SSE (Streaming SIMD Extensions) 128-bit registers (extended for x64):
            AllRegisters::xmm0
            | AllRegisters::xmm1
            | AllRegisters::xmm2
            | AllRegisters::xmm3
            | AllRegisters::xmm4
            | AllRegisters::xmm5
            | AllRegisters::xmm6
            | AllRegisters::xmm7
            | AllRegisters::xmm8
            | AllRegisters::xmm9
            | AllRegisters::xmm10
            | AllRegisters::xmm11
            | AllRegisters::xmm12
            | AllRegisters::xmm13
            | AllRegisters::xmm14
            | AllRegisters::xmm15 => 16, // 128 bits

            // AVX (Advanced Vector Extensions) 256-bit registers (extended for x64):
            AllRegisters::ymm0
            | AllRegisters::ymm1
            | AllRegisters::ymm2
            | AllRegisters::ymm3
            | AllRegisters::ymm4
            | AllRegisters::ymm5
            | AllRegisters::ymm6
            | AllRegisters::ymm7
            | AllRegisters::ymm8
            | AllRegisters::ymm9
            | AllRegisters::ymm10
            | AllRegisters::ymm11
            | AllRegisters::ymm12
            | AllRegisters::ymm13
            | AllRegisters::ymm14
            | AllRegisters::ymm15 => 32, // 256 bits

            // AVX-512 512-bit registers (extended for x64):
            AllRegisters::zmm0
            | AllRegisters::zmm1
            | AllRegisters::zmm2
            | AllRegisters::zmm3
            | AllRegisters::zmm4
            | AllRegisters::zmm5
            | AllRegisters::zmm6
            | AllRegisters::zmm7
            | AllRegisters::zmm8
            | AllRegisters::zmm9
            | AllRegisters::zmm10
            | AllRegisters::zmm11
            | AllRegisters::zmm12
            | AllRegisters::zmm13
            | AllRegisters::zmm14
            | AllRegisters::zmm15 => 64, // 512 bits
        }
    }

    pub(crate) fn is_32(&self) -> bool {
        matches!(
            *self,
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

    pub(crate) fn is_64(&self) -> bool {
        matches!(
            *self,
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

    pub(crate) fn is_xmm(&self) -> bool {
        matches!(
            *self,
            AllRegisters::xmm0
                | AllRegisters::xmm1
                | AllRegisters::xmm2
                | AllRegisters::xmm3
                | AllRegisters::xmm4
                | AllRegisters::xmm5
                | AllRegisters::xmm6
                | AllRegisters::xmm7
                | AllRegisters::xmm8
                | AllRegisters::xmm9
                | AllRegisters::xmm10
                | AllRegisters::xmm11
                | AllRegisters::xmm12
                | AllRegisters::xmm13
                | AllRegisters::xmm14
                | AllRegisters::xmm15
        )
    }

    pub(crate) fn is_ymm(&self) -> bool {
        matches!(
            *self,
            AllRegisters::ymm0
                | AllRegisters::ymm1
                | AllRegisters::ymm2
                | AllRegisters::ymm3
                | AllRegisters::ymm4
                | AllRegisters::ymm5
                | AllRegisters::ymm6
                | AllRegisters::ymm7
                | AllRegisters::ymm8
                | AllRegisters::ymm9
                | AllRegisters::ymm10
                | AllRegisters::ymm11
                | AllRegisters::ymm12
                | AllRegisters::ymm13
                | AllRegisters::ymm14
                | AllRegisters::ymm15
        )
    }

    pub(crate) fn is_zmm(&self) -> bool {
        matches!(
            *self,
            AllRegisters::zmm0
                | AllRegisters::zmm1
                | AllRegisters::zmm2
                | AllRegisters::zmm3
                | AllRegisters::zmm4
                | AllRegisters::zmm5
                | AllRegisters::zmm6
                | AllRegisters::zmm7
                | AllRegisters::zmm8
                | AllRegisters::zmm9
                | AllRegisters::zmm10
                | AllRegisters::zmm11
                | AllRegisters::zmm12
                | AllRegisters::zmm13
                | AllRegisters::zmm14
                | AllRegisters::zmm15
        )
    }

    pub(crate) fn as_iced_32(&self) -> Result<AsmRegister32, JitError<AllRegisters>> {
        match *self {
            AllRegisters::eax => Ok(iced_x86::code_asm::registers::eax),
            AllRegisters::ebx => Ok(iced_x86::code_asm::registers::ebx),
            AllRegisters::ecx => Ok(iced_x86::code_asm::registers::ecx),
            AllRegisters::edx => Ok(iced_x86::code_asm::registers::edx),
            AllRegisters::esi => Ok(iced_x86::code_asm::registers::esi),
            AllRegisters::edi => Ok(iced_x86::code_asm::registers::edi),
            AllRegisters::ebp => Ok(iced_x86::code_asm::registers::ebp),
            AllRegisters::esp => Ok(iced_x86::code_asm::registers::esp),
            _ => Err(JitError::InvalidRegister(*self)),
        }
    }

    pub(crate) fn as_iced_64(&self) -> Result<AsmRegister64, JitError<AllRegisters>> {
        match *self {
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

            _ => Err(JitError::InvalidRegister(*self)),
        }
    }

    pub(crate) fn as_iced_st(&self) -> Result<AsmRegisterSt, JitError<AllRegisters>> {
        match *self {
            AllRegisters::st0 => Ok(iced_x86::code_asm::registers::st0),
            AllRegisters::st1 => Ok(iced_x86::code_asm::registers::st1),
            AllRegisters::st2 => Ok(iced_x86::code_asm::registers::st2),
            AllRegisters::st3 => Ok(iced_x86::code_asm::registers::st3),
            AllRegisters::st4 => Ok(iced_x86::code_asm::registers::st4),
            AllRegisters::st5 => Ok(iced_x86::code_asm::registers::st5),
            AllRegisters::st6 => Ok(iced_x86::code_asm::registers::st6),
            AllRegisters::st7 => Ok(iced_x86::code_asm::registers::st7),
            _ => Err(JitError::InvalidRegister(*self)),
        }
    }

    pub(crate) fn as_iced_xmm(&self) -> Result<AsmRegisterXmm, JitError<AllRegisters>> {
        match *self {
            AllRegisters::xmm0 => Ok(iced_x86::code_asm::registers::xmm0),
            AllRegisters::xmm1 => Ok(iced_x86::code_asm::registers::xmm1),
            AllRegisters::xmm2 => Ok(iced_x86::code_asm::registers::xmm2),
            AllRegisters::xmm3 => Ok(iced_x86::code_asm::registers::xmm3),
            AllRegisters::xmm4 => Ok(iced_x86::code_asm::registers::xmm4),
            AllRegisters::xmm5 => Ok(iced_x86::code_asm::registers::xmm5),
            AllRegisters::xmm6 => Ok(iced_x86::code_asm::registers::xmm6),
            AllRegisters::xmm7 => Ok(iced_x86::code_asm::registers::xmm7),
            AllRegisters::xmm8 => Ok(iced_x86::code_asm::registers::xmm8),
            AllRegisters::xmm9 => Ok(iced_x86::code_asm::registers::xmm9),
            AllRegisters::xmm10 => Ok(iced_x86::code_asm::registers::xmm10),
            AllRegisters::xmm11 => Ok(iced_x86::code_asm::registers::xmm11),
            AllRegisters::xmm12 => Ok(iced_x86::code_asm::registers::xmm12),
            AllRegisters::xmm13 => Ok(iced_x86::code_asm::registers::xmm13),
            AllRegisters::xmm14 => Ok(iced_x86::code_asm::registers::xmm14),
            AllRegisters::xmm15 => Ok(iced_x86::code_asm::registers::xmm15),
            _ => Err(JitError::InvalidRegister(*self)),
        }
    }

    pub(crate) fn as_iced_ymm(&self) -> Result<AsmRegisterYmm, JitError<AllRegisters>> {
        match *self {
            AllRegisters::ymm0 => Ok(iced_x86::code_asm::registers::ymm0),
            AllRegisters::ymm1 => Ok(iced_x86::code_asm::registers::ymm1),
            AllRegisters::ymm2 => Ok(iced_x86::code_asm::registers::ymm2),
            AllRegisters::ymm3 => Ok(iced_x86::code_asm::registers::ymm3),
            AllRegisters::ymm4 => Ok(iced_x86::code_asm::registers::ymm4),
            AllRegisters::ymm5 => Ok(iced_x86::code_asm::registers::ymm5),
            AllRegisters::ymm6 => Ok(iced_x86::code_asm::registers::ymm6),
            AllRegisters::ymm7 => Ok(iced_x86::code_asm::registers::ymm7),
            AllRegisters::ymm8 => Ok(iced_x86::code_asm::registers::ymm8),
            AllRegisters::ymm9 => Ok(iced_x86::code_asm::registers::ymm9),
            AllRegisters::ymm10 => Ok(iced_x86::code_asm::registers::ymm10),
            AllRegisters::ymm11 => Ok(iced_x86::code_asm::registers::ymm11),
            AllRegisters::ymm12 => Ok(iced_x86::code_asm::registers::ymm12),
            AllRegisters::ymm13 => Ok(iced_x86::code_asm::registers::ymm13),
            AllRegisters::ymm14 => Ok(iced_x86::code_asm::registers::ymm14),
            AllRegisters::ymm15 => Ok(iced_x86::code_asm::registers::ymm15),
            _ => Err(JitError::InvalidRegister(*self)),
        }
    }

    pub(crate) fn as_iced_zmm(&self) -> Result<AsmRegisterZmm, JitError<AllRegisters>> {
        match *self {
            AllRegisters::zmm0 => Ok(iced_x86::code_asm::registers::zmm0),
            AllRegisters::zmm1 => Ok(iced_x86::code_asm::registers::zmm1),
            AllRegisters::zmm2 => Ok(iced_x86::code_asm::registers::zmm2),
            AllRegisters::zmm3 => Ok(iced_x86::code_asm::registers::zmm3),
            AllRegisters::zmm4 => Ok(iced_x86::code_asm::registers::zmm4),
            AllRegisters::zmm5 => Ok(iced_x86::code_asm::registers::zmm5),
            AllRegisters::zmm6 => Ok(iced_x86::code_asm::registers::zmm6),
            AllRegisters::zmm7 => Ok(iced_x86::code_asm::registers::zmm7),
            AllRegisters::zmm8 => Ok(iced_x86::code_asm::registers::zmm8),
            AllRegisters::zmm9 => Ok(iced_x86::code_asm::registers::zmm9),
            AllRegisters::zmm10 => Ok(iced_x86::code_asm::registers::zmm10),
            AllRegisters::zmm11 => Ok(iced_x86::code_asm::registers::zmm11),
            AllRegisters::zmm12 => Ok(iced_x86::code_asm::registers::zmm12),
            AllRegisters::zmm13 => Ok(iced_x86::code_asm::registers::zmm13),
            AllRegisters::zmm14 => Ok(iced_x86::code_asm::registers::zmm14),
            AllRegisters::zmm15 => Ok(iced_x86::code_asm::registers::zmm15),
            _ => Err(JitError::InvalidRegister(*self)),
        }
    }
}
