use reloaded_hooks_portable::api::traits::register_size::RegisterInfo;

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

impl RegisterInfo for Register {
    fn size_in_bytes(&self) -> usize {
        match self {
            Register::eax
            | Register::ebx
            | Register::ecx
            | Register::edx
            | Register::esi
            | Register::edi
            | Register::ebp
            | Register::esp => 4,

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
            | Register::xmm7 => 16,

            Register::ymm0
            | Register::ymm1
            | Register::ymm2
            | Register::ymm3
            | Register::ymm4
            | Register::ymm5
            | Register::ymm6
            | Register::ymm7 => 32,

            Register::zmm0
            | Register::zmm1
            | Register::zmm2
            | Register::zmm3
            | Register::zmm4
            | Register::zmm5
            | Register::zmm6
            | Register::zmm7 => 64,
        }
    }

    fn is_stack_pointer(&self) -> bool {
        self == &Register::esp
    }
}
