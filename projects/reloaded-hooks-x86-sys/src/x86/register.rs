use core::mem::transmute;
use derive_enum_all_values::AllValues;
use reloaded_hooks_portable::api::traits::register_info::{KnownRegisterType, RegisterInfo};

/// Defines a full size x86 register, used in specifying custom calling conventions.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, AllValues)]
pub enum Register {
    // 0b10000 - 0b11111
    // x87 Floating-point stack registers (8 registers, 8 reserved)
    st0 = 0b10000,
    st1,
    st2,
    st3,
    st4,
    st5,
    st6,
    st7,

    // 0b100000 - 0b111111
    // General purpose 64-bit registers (16 registers, 16 reserved)
    #[default]
    eax = 0b100000,
    ebx,
    ecx,
    edx,
    esi,
    edi,
    ebp,
    esp,

    // SSE 128-bit registers (16 registers, 48 reserved)
    xmm0 = 0b1000000,
    xmm1,
    xmm2,
    xmm3,
    xmm4,
    xmm5,
    xmm6,
    xmm7,

    // AVX 256-bit registers (16 registers, 240 reserved)
    ymm0 = 0b10000000,
    ymm1,
    ymm2,
    ymm3,
    ymm4,
    ymm5,
    ymm6,
    ymm7,

    // AVX-512 512-bit registers (16 registers, 496 reserved)
    zmm0 = 0b100000000,
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
        let value = *self as usize;
        match value {
            _ if value & 0b10000 != 0 => 10,     // st0 - st7
            _ if value & 0b100000 != 0 => 4,     // eax - esp
            _ if value & 0b1000000 != 0 => 16,   // xmm0 - xmm7
            _ if value & 0b10000000 != 0 => 32,  // ymm0 - ymm7
            _ if value & 0b100000000 != 0 => 64, // zmm0 - zmm7
            _ => unreachable!(), // Should never reach here if the enum is well-defined
        }
    }

    fn is_stack_pointer(&self) -> bool {
        self == &Register::esp
    }

    fn register_type(&self) -> KnownRegisterType {
        let value = *self as usize;
        match value {
            _ if value & 0b10000 != 0 => KnownRegisterType::FloatingPoint, // st0 - st7
            _ if value & 0b100000 != 0 => KnownRegisterType::GeneralPurpose32, // eax - esp
            _ if value & 0b1000000 != 0 => KnownRegisterType::Vector128,   // xmm0 - xmm7
            _ if value & 0b10000000 != 0 => KnownRegisterType::Vector256,  // ymm0 - ymm7
            _ if value & 0b100000000 != 0 => KnownRegisterType::Vector512, // zmm0 - zmm7
            _ => unreachable!(), // Should never reach here if the enum is well-defined
        }
    }

    fn extend(&self) -> Self {
        let mut value = *self as usize;

        if value >= 0b100000000 {
            // zmm registers are already the largest in their category.
            return *self;
        }

        if value & 0b10000000 != 0 {
            // If the register is a ymm register, extend it to a zmm register.
            value ^= 0b10000000;
            value |= 0b100000000;
            return unsafe { transmute(value as u16) };
        }

        if value & 0b1000000 != 0 {
            // If the register is an xmm register, extend it to a zmm register.
            value ^= 0b1000000;
            value |= 0b100000000;
            return unsafe { transmute(value as u16) };
        }

        // General-purpose and Floating-point registers do not have a larger counterpart in this enum.
        // Return the same register.
        *self
    }

    fn all_registers() -> &'static [Self]
    where
        Self: Sized,
    {
        Self::all_values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x86::Register::*;
    use reloaded_hooks_portable::api::traits::register_info::KnownRegisterType::*;
    use rstest::rstest;

    #[rstest]
    #[case(st0, 10)]
    #[case(st1, 10)]
    #[case(st2, 10)]
    #[case(st3, 10)]
    #[case(st4, 10)]
    #[case(st5, 10)]
    #[case(st6, 10)]
    #[case(st7, 10)]
    #[case(eax, 4)]
    #[case(ebx, 4)]
    #[case(ecx, 4)]
    #[case(edx, 4)]
    #[case(esi, 4)]
    #[case(edi, 4)]
    #[case(ebp, 4)]
    #[case(esp, 4)]
    #[case(xmm0, 16)]
    #[case(xmm1, 16)]
    #[case(xmm2, 16)]
    #[case(xmm3, 16)]
    #[case(xmm4, 16)]
    #[case(xmm5, 16)]
    #[case(xmm6, 16)]
    #[case(xmm7, 16)]
    #[case(ymm0, 32)]
    #[case(ymm1, 32)]
    #[case(ymm2, 32)]
    #[case(ymm3, 32)]
    #[case(ymm4, 32)]
    #[case(ymm5, 32)]
    #[case(ymm6, 32)]
    #[case(ymm7, 32)]
    #[case(zmm0, 64)]
    #[case(zmm1, 64)]
    #[case(zmm2, 64)]
    #[case(zmm3, 64)]
    #[case(zmm4, 64)]
    #[case(zmm5, 64)]
    #[case(zmm6, 64)]
    #[case(zmm7, 64)]
    fn size_in_bytes_test(#[case] register: Register, #[case] expected_size: usize) {
        assert_eq!(register.size_in_bytes(), expected_size);
    }

    #[rstest]
    #[case(st0, false)]
    #[case(st1, false)]
    #[case(st2, false)]
    #[case(st3, false)]
    #[case(st4, false)]
    #[case(st5, false)]
    #[case(st6, false)]
    #[case(st7, false)]
    #[case(eax, false)]
    #[case(ebx, false)]
    #[case(ecx, false)]
    #[case(edx, false)]
    #[case(esi, false)]
    #[case(edi, false)]
    #[case(ebp, false)]
    #[case(esp, true)]
    #[case(xmm0, false)]
    #[case(xmm1, false)]
    #[case(xmm2, false)]
    #[case(xmm3, false)]
    #[case(xmm4, false)]
    #[case(xmm5, false)]
    #[case(xmm6, false)]
    #[case(xmm7, false)]
    #[case(ymm0, false)]
    #[case(ymm1, false)]
    #[case(ymm2, false)]
    #[case(ymm3, false)]
    #[case(ymm4, false)]
    #[case(ymm5, false)]
    #[case(ymm6, false)]
    #[case(ymm7, false)]
    #[case(zmm0, false)]
    #[case(zmm1, false)]
    #[case(zmm2, false)]
    #[case(zmm3, false)]
    #[case(zmm4, false)]
    #[case(zmm5, false)]
    #[case(zmm6, false)]
    #[case(zmm7, false)]
    fn is_stack_pointer_test(#[case] register: Register, #[case] is_sp: bool) {
        assert_eq!(register.is_stack_pointer(), is_sp);
    }

    #[rstest]
    #[case(st0, FloatingPoint)]
    #[case(st1, FloatingPoint)]
    #[case(st2, FloatingPoint)]
    #[case(st3, FloatingPoint)]
    #[case(st4, FloatingPoint)]
    #[case(st5, FloatingPoint)]
    #[case(st6, FloatingPoint)]
    #[case(st7, FloatingPoint)]
    #[case(eax, GeneralPurpose32)]
    #[case(ebx, GeneralPurpose32)]
    #[case(ecx, GeneralPurpose32)]
    #[case(edx, GeneralPurpose32)]
    #[case(esi, GeneralPurpose32)]
    #[case(edi, GeneralPurpose32)]
    #[case(ebp, GeneralPurpose32)]
    #[case(esp, GeneralPurpose32)]
    #[case(xmm0, Vector128)]
    #[case(xmm1, Vector128)]
    #[case(xmm2, Vector128)]
    #[case(xmm3, Vector128)]
    #[case(xmm4, Vector128)]
    #[case(xmm5, Vector128)]
    #[case(xmm6, Vector128)]
    #[case(xmm7, Vector128)]
    #[case(ymm0, Vector256)]
    #[case(ymm1, Vector256)]
    #[case(ymm2, Vector256)]
    #[case(ymm3, Vector256)]
    #[case(ymm4, Vector256)]
    #[case(ymm5, Vector256)]
    #[case(ymm6, Vector256)]
    #[case(ymm7, Vector256)]
    #[case(zmm0, Vector512)]
    #[case(zmm1, Vector512)]
    #[case(zmm2, Vector512)]
    #[case(zmm3, Vector512)]
    #[case(zmm4, Vector512)]
    #[case(zmm5, Vector512)]
    #[case(zmm6, Vector512)]
    #[case(zmm7, Vector512)]
    fn register_type_test(#[case] register: Register, #[case] expected_type: KnownRegisterType) {
        assert_eq!(register.register_type(), expected_type);
    }

    #[rstest]
    #[case(st0, st0)]
    #[case(st1, st1)]
    #[case(st2, st2)]
    #[case(st3, st3)]
    #[case(st4, st4)]
    #[case(st5, st5)]
    #[case(st6, st6)]
    #[case(st7, st7)]
    #[case(eax, eax)]
    #[case(ebx, ebx)]
    #[case(ecx, ecx)]
    #[case(edx, edx)]
    #[case(esi, esi)]
    #[case(edi, edi)]
    #[case(ebp, ebp)]
    #[case(esp, esp)]
    #[case(xmm0, zmm0)]
    #[case(xmm1, zmm1)]
    #[case(xmm2, zmm2)]
    #[case(xmm3, zmm3)]
    #[case(xmm4, zmm4)]
    #[case(xmm5, zmm5)]
    #[case(xmm6, zmm6)]
    #[case(xmm7, zmm7)]
    #[case(ymm0, zmm0)]
    #[case(ymm1, zmm1)]
    #[case(ymm2, zmm2)]
    #[case(ymm3, zmm3)]
    #[case(ymm4, zmm4)]
    #[case(ymm5, zmm5)]
    #[case(ymm6, zmm6)]
    #[case(ymm7, zmm7)]
    #[case(zmm0, zmm0)]
    #[case(zmm1, zmm1)]
    #[case(zmm2, zmm2)]
    #[case(zmm3, zmm3)]
    #[case(zmm4, zmm4)]
    #[case(zmm5, zmm5)]
    #[case(zmm6, zmm6)]
    #[case(zmm7, zmm7)]
    fn extend_test(#[case] input: Register, #[case] expected: Register) {
        assert_eq!(input.extend(), expected);
    }
}
