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
    #[cfg(feature = "x64")]
    rax,
    #[cfg(feature = "x64")]
    rbx,
    #[cfg(feature = "x64")]
    rcx,
    #[cfg(feature = "x64")]
    rdx,
    #[cfg(feature = "x64")]
    rsi,
    #[cfg(feature = "x64")]
    rdi,
    #[cfg(feature = "x64")]
    rbp,
    #[cfg(feature = "x64")]
    rsp,
    #[cfg(feature = "x64")]
    r8,
    #[cfg(feature = "x64")]
    r9,
    #[cfg(feature = "x64")]
    r10,
    #[cfg(feature = "x64")]
    r11,
    #[cfg(feature = "x64")]
    r12,
    #[cfg(feature = "x64")]
    r13,
    #[cfg(feature = "x64")]
    r14,
    #[cfg(feature = "x64")]
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
    #[cfg(feature = "x64")]
    xmm8,
    #[cfg(feature = "x64")]
    xmm9,
    #[cfg(feature = "x64")]
    xmm10,
    #[cfg(feature = "x64")]
    xmm11,
    #[cfg(feature = "x64")]
    xmm12,
    #[cfg(feature = "x64")]
    xmm13,
    #[cfg(feature = "x64")]
    xmm14,
    #[cfg(feature = "x64")]
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
    #[cfg(feature = "x64")]
    ymm8,
    #[cfg(feature = "x64")]
    ymm9,
    #[cfg(feature = "x64")]
    ymm10,
    #[cfg(feature = "x64")]
    ymm11,
    #[cfg(feature = "x64")]
    ymm12,
    #[cfg(feature = "x64")]
    ymm13,
    #[cfg(feature = "x64")]
    ymm14,
    #[cfg(feature = "x64")]
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
    #[cfg(feature = "x64")]
    zmm8,
    #[cfg(feature = "x64")]
    zmm9,
    #[cfg(feature = "x64")]
    zmm10,
    #[cfg(feature = "x64")]
    zmm11,
    #[cfg(feature = "x64")]
    zmm12,
    #[cfg(feature = "x64")]
    zmm13,
    #[cfg(feature = "x64")]
    zmm14,
    #[cfg(feature = "x64")]
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
            #[cfg(feature = "x64")]
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
            | AllRegisters::xmm7 => 16, // 128 bits

            #[cfg(feature = "x64")]
            AllRegisters::xmm8
            | AllRegisters::xmm9
            | AllRegisters::xmm10
            | AllRegisters::xmm11
            | AllRegisters::xmm12
            | AllRegisters::xmm13
            | AllRegisters::xmm14
            | AllRegisters::xmm15 => 16, // 128 bits (x64)

            // AVX (Advanced Vector Extensions) 256-bit registers (extended for x64):
            AllRegisters::ymm0
            | AllRegisters::ymm1
            | AllRegisters::ymm2
            | AllRegisters::ymm3
            | AllRegisters::ymm4
            | AllRegisters::ymm5
            | AllRegisters::ymm6
            | AllRegisters::ymm7 => 32, // 256 bits

            #[cfg(feature = "x64")]
            AllRegisters::ymm8
            | AllRegisters::ymm9
            | AllRegisters::ymm10
            | AllRegisters::ymm11
            | AllRegisters::ymm12
            | AllRegisters::ymm13
            | AllRegisters::ymm14
            | AllRegisters::ymm15 => 32, // 256 bits (x64)

            // AVX-512 512-bit registers (extended for x64):
            AllRegisters::zmm0
            | AllRegisters::zmm1
            | AllRegisters::zmm2
            | AllRegisters::zmm3
            | AllRegisters::zmm4
            | AllRegisters::zmm5
            | AllRegisters::zmm6
            | AllRegisters::zmm7 => 64, // 512 bits

            #[cfg(feature = "x64")]
            AllRegisters::zmm8
            | AllRegisters::zmm9
            | AllRegisters::zmm10
            | AllRegisters::zmm11
            | AllRegisters::zmm12
            | AllRegisters::zmm13
            | AllRegisters::zmm14
            | AllRegisters::zmm15 => 64, // 512 bits
        }
    }

    pub(crate) fn as_iced_allregister(&self) -> Result<iced_x86::Register, JitError<AllRegisters>> {
        match *self {
            // x86
            AllRegisters::eax => Ok(iced_x86::Register::EAX),
            AllRegisters::ebx => Ok(iced_x86::Register::EBX),
            AllRegisters::ecx => Ok(iced_x86::Register::ECX),
            AllRegisters::edx => Ok(iced_x86::Register::EDX),
            AllRegisters::esi => Ok(iced_x86::Register::ESI),
            AllRegisters::edi => Ok(iced_x86::Register::EDI),
            AllRegisters::ebp => Ok(iced_x86::Register::EBP),
            AllRegisters::esp => Ok(iced_x86::Register::ESP),

            // x64
            #[cfg(feature = "x64")]
            AllRegisters::rax => Ok(iced_x86::Register::RAX),
            #[cfg(feature = "x64")]
            AllRegisters::rbx => Ok(iced_x86::Register::RBX),
            #[cfg(feature = "x64")]
            AllRegisters::rcx => Ok(iced_x86::Register::RCX),
            #[cfg(feature = "x64")]
            AllRegisters::rdx => Ok(iced_x86::Register::RDX),
            #[cfg(feature = "x64")]
            AllRegisters::rsi => Ok(iced_x86::Register::RSI),
            #[cfg(feature = "x64")]
            AllRegisters::rdi => Ok(iced_x86::Register::RDI),
            #[cfg(feature = "x64")]
            AllRegisters::rbp => Ok(iced_x86::Register::RBP),
            #[cfg(feature = "x64")]
            AllRegisters::rsp => Ok(iced_x86::Register::RSP),
            #[cfg(feature = "x64")]
            AllRegisters::r8 => Ok(iced_x86::Register::R8),
            #[cfg(feature = "x64")]
            AllRegisters::r9 => Ok(iced_x86::Register::R9),
            #[cfg(feature = "x64")]
            AllRegisters::r10 => Ok(iced_x86::Register::R10),
            #[cfg(feature = "x64")]
            AllRegisters::r11 => Ok(iced_x86::Register::R11),
            #[cfg(feature = "x64")]
            AllRegisters::r12 => Ok(iced_x86::Register::R12),
            #[cfg(feature = "x64")]
            AllRegisters::r13 => Ok(iced_x86::Register::R13),
            #[cfg(feature = "x64")]
            AllRegisters::r14 => Ok(iced_x86::Register::R14),
            #[cfg(feature = "x64")]
            AllRegisters::r15 => Ok(iced_x86::Register::R15),

            // x87 Floating-point stack registers
            AllRegisters::st0 => Ok(iced_x86::Register::ST0),
            AllRegisters::st1 => Ok(iced_x86::Register::ST1),
            AllRegisters::st2 => Ok(iced_x86::Register::ST2),
            AllRegisters::st3 => Ok(iced_x86::Register::ST3),
            AllRegisters::st4 => Ok(iced_x86::Register::ST4),
            AllRegisters::st5 => Ok(iced_x86::Register::ST5),
            AllRegisters::st6 => Ok(iced_x86::Register::ST6),
            AllRegisters::st7 => Ok(iced_x86::Register::ST7),

            // SSE (Streaming SIMD Extensions) 128-bit registers
            AllRegisters::xmm0 => Ok(iced_x86::Register::XMM0),
            AllRegisters::xmm1 => Ok(iced_x86::Register::XMM1),
            AllRegisters::xmm2 => Ok(iced_x86::Register::XMM2),
            AllRegisters::xmm3 => Ok(iced_x86::Register::XMM3),
            AllRegisters::xmm4 => Ok(iced_x86::Register::XMM4),
            AllRegisters::xmm5 => Ok(iced_x86::Register::XMM5),
            AllRegisters::xmm6 => Ok(iced_x86::Register::XMM6),
            AllRegisters::xmm7 => Ok(iced_x86::Register::XMM7),

            #[cfg(feature = "x64")]
            AllRegisters::xmm8 => Ok(iced_x86::Register::XMM8),
            #[cfg(feature = "x64")]
            AllRegisters::xmm9 => Ok(iced_x86::Register::XMM9),
            #[cfg(feature = "x64")]
            AllRegisters::xmm10 => Ok(iced_x86::Register::XMM10),
            #[cfg(feature = "x64")]
            AllRegisters::xmm11 => Ok(iced_x86::Register::XMM11),
            #[cfg(feature = "x64")]
            AllRegisters::xmm12 => Ok(iced_x86::Register::XMM12),
            #[cfg(feature = "x64")]
            AllRegisters::xmm13 => Ok(iced_x86::Register::XMM13),
            #[cfg(feature = "x64")]
            AllRegisters::xmm14 => Ok(iced_x86::Register::XMM14),
            #[cfg(feature = "x64")]
            AllRegisters::xmm15 => Ok(iced_x86::Register::XMM15),

            // AVX (Advanced Vector Extensions) 256-bit registers
            AllRegisters::ymm0 => Ok(iced_x86::Register::YMM0),
            AllRegisters::ymm1 => Ok(iced_x86::Register::YMM1),
            AllRegisters::ymm2 => Ok(iced_x86::Register::YMM2),
            AllRegisters::ymm3 => Ok(iced_x86::Register::YMM3),
            AllRegisters::ymm4 => Ok(iced_x86::Register::YMM4),
            AllRegisters::ymm5 => Ok(iced_x86::Register::YMM5),
            AllRegisters::ymm6 => Ok(iced_x86::Register::YMM6),
            AllRegisters::ymm7 => Ok(iced_x86::Register::YMM7),

            #[cfg(feature = "x64")]
            AllRegisters::ymm8 => Ok(iced_x86::Register::YMM8),
            #[cfg(feature = "x64")]
            AllRegisters::ymm9 => Ok(iced_x86::Register::YMM9),
            #[cfg(feature = "x64")]
            AllRegisters::ymm10 => Ok(iced_x86::Register::YMM10),
            #[cfg(feature = "x64")]
            AllRegisters::ymm11 => Ok(iced_x86::Register::YMM11),
            #[cfg(feature = "x64")]
            AllRegisters::ymm12 => Ok(iced_x86::Register::YMM12),
            #[cfg(feature = "x64")]
            AllRegisters::ymm13 => Ok(iced_x86::Register::YMM13),
            #[cfg(feature = "x64")]
            AllRegisters::ymm14 => Ok(iced_x86::Register::YMM14),
            #[cfg(feature = "x64")]
            AllRegisters::ymm15 => Ok(iced_x86::Register::YMM15),

            // AVX-512 512-bit registers
            AllRegisters::zmm0 => Ok(iced_x86::Register::ZMM0),
            AllRegisters::zmm1 => Ok(iced_x86::Register::ZMM1),
            AllRegisters::zmm2 => Ok(iced_x86::Register::ZMM2),
            AllRegisters::zmm3 => Ok(iced_x86::Register::ZMM3),
            AllRegisters::zmm4 => Ok(iced_x86::Register::ZMM4),
            AllRegisters::zmm5 => Ok(iced_x86::Register::ZMM5),
            AllRegisters::zmm6 => Ok(iced_x86::Register::ZMM6),
            AllRegisters::zmm7 => Ok(iced_x86::Register::ZMM7),

            #[cfg(feature = "x64")]
            AllRegisters::zmm8 => Ok(iced_x86::Register::ZMM8),
            #[cfg(feature = "x64")]
            AllRegisters::zmm9 => Ok(iced_x86::Register::ZMM9),
            #[cfg(feature = "x64")]
            AllRegisters::zmm10 => Ok(iced_x86::Register::ZMM10),
            #[cfg(feature = "x64")]
            AllRegisters::zmm11 => Ok(iced_x86::Register::ZMM11),
            #[cfg(feature = "x64")]
            AllRegisters::zmm12 => Ok(iced_x86::Register::ZMM12),
            #[cfg(feature = "x64")]
            AllRegisters::zmm13 => Ok(iced_x86::Register::ZMM13),
            #[cfg(feature = "x64")]
            AllRegisters::zmm14 => Ok(iced_x86::Register::ZMM14),
            #[cfg(feature = "x64")]
            AllRegisters::zmm15 => Ok(iced_x86::Register::ZMM15),
        }
    }
}
