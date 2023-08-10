use reloaded_hooks_portable::api::traits::register_size::RegisterInfo;

/// Defines a full size x64 register, used in specifying custom calling conventions.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Register {
    /// Accumulator register, used in arithmetic operations.
    #[default]
    rax,
    /// Base register, used as a pointer to data.
    rbx,
    /// Counter register, used in loops and shifts.
    rcx,
    /// Data register, used in arithmetic operations and I/O.
    rdx,
    /// Source Index, used in string operations.
    rsi,
    /// Destination Index, used in string operations.
    rdi,
    /// Base Pointer, points to data on the stack.
    rbp,
    /// Stack Pointer, points to the top of the stack.
    rsp,

    /// Extended 64-bit registers:
    r8,
    r9,
    r10,
    r11,
    r12,
    r13,
    r14,
    r15,

    /// x87 Floating-point stack registers:
    /// unsupported in this library; just here for completion
    st0,
    st1,
    st2,
    st3,
    st4,
    st5,
    st6,
    st7,

    /// SSE (Streaming SIMD Extensions) 128-bit registers:
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

    /// AVX (Advanced Vector Extensions) 256-bit registers:
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

    /// AVX-512 512-bit registers:
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

impl RegisterInfo for Register {
    fn size_in_bytes(&self) -> usize {
        match self {
            Register::rax
            | Register::rbx
            | Register::rcx
            | Register::rdx
            | Register::rsi
            | Register::rdi
            | Register::rbp
            | Register::rsp
            | Register::r8
            | Register::r9
            | Register::r10
            | Register::r11
            | Register::r12
            | Register::r13
            | Register::r14
            | Register::r15 => 8,

            Register::st0
            | Register::st1
            | Register::st2
            | Register::st3
            | Register::st4
            | Register::st5
            | Register::st6
            | Register::st7 => 10,

            Register::xmm0
            | Register::xmm1
            | Register::xmm2
            | Register::xmm3
            | Register::xmm4
            | Register::xmm5
            | Register::xmm6
            | Register::xmm7
            | Register::xmm8
            | Register::xmm9
            | Register::xmm10
            | Register::xmm11
            | Register::xmm12
            | Register::xmm13
            | Register::xmm14
            | Register::xmm15 => 16,

            Register::ymm0
            | Register::ymm1
            | Register::ymm2
            | Register::ymm3
            | Register::ymm4
            | Register::ymm5
            | Register::ymm6
            | Register::ymm7
            | Register::ymm8
            | Register::ymm9
            | Register::ymm10
            | Register::ymm11
            | Register::ymm12
            | Register::ymm13
            | Register::ymm14
            | Register::ymm15 => 32,

            Register::zmm0
            | Register::zmm1
            | Register::zmm2
            | Register::zmm3
            | Register::zmm4
            | Register::zmm5
            | Register::zmm6
            | Register::zmm7
            | Register::zmm8
            | Register::zmm9
            | Register::zmm10
            | Register::zmm11
            | Register::zmm12
            | Register::zmm13
            | Register::zmm14
            | Register::zmm15 => 64,
        }
    }

    fn is_stack_pointer(&self) -> bool {
        self == &Register::rsp
    }

    fn register_type(&self) -> usize {
        match self {
            Register::rax
            | Register::rbx
            | Register::rcx
            | Register::rdx
            | Register::rsi
            | Register::rdi
            | Register::rbp
            | Register::rsp
            | Register::r8
            | Register::r9
            | Register::r10
            | Register::r11
            | Register::r12
            | Register::r13
            | Register::r14
            | Register::r15 => 0,
            Register::st0
            | Register::st1
            | Register::st2
            | Register::st3
            | Register::st4
            | Register::st5
            | Register::st6
            | Register::st7 => 1,
            Register::xmm0
            | Register::xmm1
            | Register::xmm2
            | Register::xmm3
            | Register::xmm4
            | Register::xmm5
            | Register::xmm6
            | Register::xmm7
            | Register::xmm8
            | Register::xmm9
            | Register::xmm10
            | Register::xmm11
            | Register::xmm12
            | Register::xmm13
            | Register::xmm14
            | Register::xmm15
            | Register::ymm0
            | Register::ymm1
            | Register::ymm2
            | Register::ymm3
            | Register::ymm4
            | Register::ymm5
            | Register::ymm6
            | Register::ymm7
            | Register::ymm8
            | Register::ymm9
            | Register::ymm10
            | Register::ymm11
            | Register::ymm12
            | Register::ymm13
            | Register::ymm14
            | Register::ymm15
            | Register::zmm0
            | Register::zmm1
            | Register::zmm2
            | Register::zmm3
            | Register::zmm4
            | Register::zmm5
            | Register::zmm6
            | Register::zmm7
            | Register::zmm8
            | Register::zmm9
            | Register::zmm10
            | Register::zmm11
            | Register::zmm12
            | Register::zmm13
            | Register::zmm14
            | Register::zmm15 => 2,
        }
    }
}
