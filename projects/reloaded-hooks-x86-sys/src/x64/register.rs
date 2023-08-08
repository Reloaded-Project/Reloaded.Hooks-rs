/// Defines a full size x64 register, used in specifying custom calling conventions.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    /// Accumulator register, used in arithmetic operations.
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
