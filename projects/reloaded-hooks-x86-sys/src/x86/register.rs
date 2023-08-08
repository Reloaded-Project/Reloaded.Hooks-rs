/// Defines a full size x86 register, used in specifying custom calling conventions.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Register {
    /// Accumulator register, used in arithmetic operations.
    #[default]
    eax,
    /// Base register, used as a pointer to data.
    ebx,
    /// Counter register, used in loops and shifts.
    ecx,
    /// Data register, used in arithmetic operations and I/O.
    edx,
    /// Source Index, used in string operations.
    esi,
    /// Destination Index, used in string operations.
    edi,
    /// Base Pointer, points to data on the stack.
    ebp,
    /// Stack Pointer, points to the top of the stack.
    esp,

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

    /// AVX (Advanced Vector Extensions) 256-bit registers:
    ymm0,
    ymm1,
    ymm2,
    ymm3,
    ymm4,
    ymm5,
    ymm6,
    ymm7,

    /// AVX-512 512-bit registers:
    zmm0,
    zmm1,
    zmm2,
    zmm3,
    zmm4,
    zmm5,
    zmm6,
    zmm7,
}
