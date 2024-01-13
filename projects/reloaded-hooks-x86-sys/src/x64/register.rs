use core::mem::transmute;
use derive_enum_all_values::AllValues;
use reloaded_hooks_portable::api::traits::register_info::{KnownRegisterType, RegisterInfo};

/// Defines a full size x64 register, used in specifying custom calling conventions.
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
    // General purpose 64-bit registers (16 registers, up to 64)
    #[default]
    rax = 0b1000000,
    rcx,
    rdx,
    rbx,
    rsp,
    rbp,
    rsi,
    rdi,
    r8,
    r9,
    r10,
    r11,
    r12,
    r13,
    r14,
    r15,

    // SSE 128-bit registers (16 registers, up to 128)
    xmm0 = 0b10000000,
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

    // AVX 256-bit registers (16 registers, up to 256)
    ymm0 = 0b100000000,
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

    // AVX-512 512-bit registers (16 registers, up to 512)
    zmm0 = 0b1000000000,
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

impl Register {
    pub fn is_64(&self) -> bool {
        *self as usize & 0b1000000 != 0
    }

    pub fn is_xmm(&self) -> bool {
        *self as usize & 0b10000000 != 0
    }

    pub fn is_ymm(&self) -> bool {
        *self as usize & 0b100000000 != 0
    }

    pub fn is_zmm(&self) -> bool {
        *self as usize & 0b1000000000 != 0
    }

    pub fn to_zydis(&self) -> zydis::Register {
        let base = *self as u32;

        if base & 0b10000 != 0 {
            unsafe { transmute(zydis::Register::ST0 as u32 + (base - Register::st0 as u32)) }
        } else if base & 0b1000000 != 0 {
            unsafe { transmute(zydis::Register::RAX as u32 + (base - Register::rax as u32)) }
        } else if base & 0b10000000 != 0 {
            unsafe { transmute(zydis::Register::XMM0 as u32 + (base - Register::xmm0 as u32)) }
        } else if base & 0b100000000 != 0 {
            unsafe { transmute(zydis::Register::YMM0 as u32 + (base - Register::ymm0 as u32)) }
        } else if base & 0b1000000000 != 0 {
            unsafe { transmute(zydis::Register::ZMM0 as u32 + (base - Register::zmm0 as u32)) }
        } else {
            zydis::Register::NONE
        }
    }
}

impl RegisterInfo for Register {
    fn size_in_bytes(&self) -> usize {
        let value = *self as usize;
        match value {
            _ if value & 0b10000 != 0 => 10,      // st0 - st7
            _ if value & 0b1000000 != 0 => 8,     // rax - r15
            _ if value & 0b10000000 != 0 => 16,   // xmm0 - xmm15
            _ if value & 0b100000000 != 0 => 32,  // ymm0 - ymm15
            _ if value & 0b1000000000 != 0 => 64, // zmm0 - zmm15
            _ => unreachable!(), // Should never reach here if the enum is well-defined
        }
    }

    fn is_stack_pointer(&self) -> bool {
        self == &Register::rsp
    }

    fn register_type(&self) -> KnownRegisterType {
        let value = *self as usize;
        match value {
            _ if value & 0b10000 != 0 => KnownRegisterType::FloatingPoint, // st0 - st7
            _ if value & 0b1000000 != 0 => KnownRegisterType::GeneralPurpose64, // rax - r15
            _ if value & 0b10000000 != 0 => KnownRegisterType::Vector128,  // xmm0 - xmm15
            _ if value & 0b100000000 != 0 => KnownRegisterType::Vector256, // ymm0 - ymm15
            _ if value & 0b1000000000 != 0 => KnownRegisterType::Vector512, // zmm0 - zmm15
            _ => unreachable!(), // Should never reach here if the enum is well-defined
        }
    }

    fn extend(&self) -> Self {
        let mut value = *self as usize;

        if value >= 0b1000000000 {
            // zmm registers are already the largest in their category.
            return *self;
        }

        if value & 0b100000000 != 0 {
            // If the register is a ymm register, extend it to a zmm register.
            value ^= 0b100000000;
            value |= 0b1000000000;
            return unsafe { transmute(value as u16) };
        }

        if value & 0b10000000 != 0 {
            // If the register is an xmm register, extend it to a zmm register.
            value ^= 0b10000000;
            value |= 0b1000000000;
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

    use crate::x64::Register::*;
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
    #[case(rax, 8)]
    #[case(rbx, 8)]
    #[case(rcx, 8)]
    #[case(rdx, 8)]
    #[case(rsi, 8)]
    #[case(rdi, 8)]
    #[case(rbp, 8)]
    #[case(rsp, 8)]
    #[case(r8, 8)]
    #[case(r9, 8)]
    #[case(r10, 8)]
    #[case(r11, 8)]
    #[case(r12, 8)]
    #[case(r13, 8)]
    #[case(r14, 8)]
    #[case(r15, 8)]
    #[case(xmm0, 16)]
    #[case(xmm1, 16)]
    #[case(xmm2, 16)]
    #[case(xmm3, 16)]
    #[case(xmm4, 16)]
    #[case(xmm5, 16)]
    #[case(xmm6, 16)]
    #[case(xmm7, 16)]
    #[case(xmm8, 16)]
    #[case(xmm9, 16)]
    #[case(xmm10, 16)]
    #[case(xmm11, 16)]
    #[case(xmm12, 16)]
    #[case(xmm13, 16)]
    #[case(xmm14, 16)]
    #[case(xmm15, 16)]
    #[case(ymm0, 32)]
    #[case(ymm1, 32)]
    #[case(ymm2, 32)]
    #[case(ymm3, 32)]
    #[case(ymm4, 32)]
    #[case(ymm5, 32)]
    #[case(ymm6, 32)]
    #[case(ymm7, 32)]
    #[case(ymm8, 32)]
    #[case(ymm9, 32)]
    #[case(ymm10, 32)]
    #[case(ymm11, 32)]
    #[case(ymm12, 32)]
    #[case(ymm13, 32)]
    #[case(ymm14, 32)]
    #[case(ymm15, 32)]
    #[case(zmm0, 64)]
    #[case(zmm1, 64)]
    #[case(zmm2, 64)]
    #[case(zmm3, 64)]
    #[case(zmm4, 64)]
    #[case(zmm5, 64)]
    #[case(zmm6, 64)]
    #[case(zmm7, 64)]
    #[case(zmm8, 64)]
    #[case(zmm9, 64)]
    #[case(zmm10, 64)]
    #[case(zmm11, 64)]
    #[case(zmm12, 64)]
    #[case(zmm13, 64)]
    #[case(zmm14, 64)]
    #[case(zmm15, 64)]
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
    #[case(rax, false)]
    #[case(rbx, false)]
    #[case(rcx, false)]
    #[case(rdx, false)]
    #[case(rsi, false)]
    #[case(rdi, false)]
    #[case(rbp, false)]
    #[case(rsp, true)]
    #[case(r8, false)]
    #[case(r9, false)]
    #[case(r10, false)]
    #[case(r11, false)]
    #[case(r12, false)]
    #[case(r13, false)]
    #[case(r14, false)]
    #[case(r15, false)]
    #[case(xmm0, false)]
    #[case(xmm1, false)]
    #[case(xmm2, false)]
    #[case(xmm3, false)]
    #[case(xmm4, false)]
    #[case(xmm5, false)]
    #[case(xmm6, false)]
    #[case(xmm7, false)]
    #[case(xmm8, false)]
    #[case(xmm9, false)]
    #[case(xmm10, false)]
    #[case(xmm11, false)]
    #[case(xmm12, false)]
    #[case(xmm13, false)]
    #[case(xmm14, false)]
    #[case(xmm15, false)]
    #[case(ymm0, false)]
    #[case(ymm1, false)]
    #[case(ymm2, false)]
    #[case(ymm3, false)]
    #[case(ymm4, false)]
    #[case(ymm5, false)]
    #[case(ymm6, false)]
    #[case(ymm7, false)]
    #[case(ymm8, false)]
    #[case(ymm9, false)]
    #[case(ymm10, false)]
    #[case(ymm11, false)]
    #[case(ymm12, false)]
    #[case(ymm13, false)]
    #[case(ymm14, false)]
    #[case(ymm15, false)]
    #[case(zmm0, false)]
    #[case(zmm1, false)]
    #[case(zmm2, false)]
    #[case(zmm3, false)]
    #[case(zmm4, false)]
    #[case(zmm5, false)]
    #[case(zmm6, false)]
    #[case(zmm7, false)]
    #[case(zmm8, false)]
    #[case(zmm9, false)]
    #[case(zmm10, false)]
    #[case(zmm11, false)]
    #[case(zmm12, false)]
    #[case(zmm13, false)]
    #[case(zmm14, false)]
    #[case(zmm15, false)]
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
    #[case(rax, GeneralPurpose64)]
    #[case(rbx, GeneralPurpose64)]
    #[case(rcx, GeneralPurpose64)]
    #[case(rdx, GeneralPurpose64)]
    #[case(rsi, GeneralPurpose64)]
    #[case(rdi, GeneralPurpose64)]
    #[case(rbp, GeneralPurpose64)]
    #[case(rsp, GeneralPurpose64)]
    #[case(r8, GeneralPurpose64)]
    #[case(r9, GeneralPurpose64)]
    #[case(r10, GeneralPurpose64)]
    #[case(r11, GeneralPurpose64)]
    #[case(r12, GeneralPurpose64)]
    #[case(r13, GeneralPurpose64)]
    #[case(r14, GeneralPurpose64)]
    #[case(r15, GeneralPurpose64)]
    #[case(xmm0, Vector128)]
    #[case(xmm1, Vector128)]
    #[case(xmm2, Vector128)]
    #[case(xmm3, Vector128)]
    #[case(xmm4, Vector128)]
    #[case(xmm5, Vector128)]
    #[case(xmm6, Vector128)]
    #[case(xmm7, Vector128)]
    #[case(xmm8, Vector128)]
    #[case(xmm9, Vector128)]
    #[case(xmm10, Vector128)]
    #[case(xmm11, Vector128)]
    #[case(xmm12, Vector128)]
    #[case(xmm13, Vector128)]
    #[case(xmm14, Vector128)]
    #[case(xmm15, Vector128)]
    #[case(ymm0, Vector256)]
    #[case(ymm1, Vector256)]
    #[case(ymm2, Vector256)]
    #[case(ymm3, Vector256)]
    #[case(ymm4, Vector256)]
    #[case(ymm5, Vector256)]
    #[case(ymm6, Vector256)]
    #[case(ymm7, Vector256)]
    #[case(ymm8, Vector256)]
    #[case(ymm9, Vector256)]
    #[case(ymm10, Vector256)]
    #[case(ymm11, Vector256)]
    #[case(ymm12, Vector256)]
    #[case(ymm13, Vector256)]
    #[case(ymm14, Vector256)]
    #[case(ymm15, Vector256)]
    #[case(zmm0, Vector512)]
    #[case(zmm1, Vector512)]
    #[case(zmm2, Vector512)]
    #[case(zmm3, Vector512)]
    #[case(zmm4, Vector512)]
    #[case(zmm5, Vector512)]
    #[case(zmm6, Vector512)]
    #[case(zmm7, Vector512)]
    #[case(zmm8, Vector512)]
    #[case(zmm9, Vector512)]
    #[case(zmm10, Vector512)]
    #[case(zmm11, Vector512)]
    #[case(zmm12, Vector512)]
    #[case(zmm13, Vector512)]
    #[case(zmm14, Vector512)]
    #[case(zmm15, Vector512)]
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
    #[case(rax, rax)]
    #[case(rbx, rbx)]
    #[case(rcx, rcx)]
    #[case(rdx, rdx)]
    #[case(rsi, rsi)]
    #[case(rdi, rdi)]
    #[case(rbp, rbp)]
    #[case(rsp, rsp)]
    #[case(r8, r8)]
    #[case(r9, r9)]
    #[case(r10, r10)]
    #[case(r11, r11)]
    #[case(r12, r12)]
    #[case(r13, r13)]
    #[case(r14, r14)]
    #[case(r15, r15)]
    #[case(xmm0, zmm0)]
    #[case(xmm1, zmm1)]
    #[case(xmm2, zmm2)]
    #[case(xmm3, zmm3)]
    #[case(xmm4, zmm4)]
    #[case(xmm5, zmm5)]
    #[case(xmm6, zmm6)]
    #[case(xmm7, zmm7)]
    #[case(xmm8, zmm8)]
    #[case(xmm9, zmm9)]
    #[case(xmm10, zmm10)]
    #[case(xmm11, zmm11)]
    #[case(xmm12, zmm12)]
    #[case(xmm13, zmm13)]
    #[case(xmm14, zmm14)]
    #[case(xmm15, zmm15)]
    #[case(ymm0, zmm0)]
    #[case(ymm1, zmm1)]
    #[case(ymm2, zmm2)]
    #[case(ymm3, zmm3)]
    #[case(ymm4, zmm4)]
    #[case(ymm5, zmm5)]
    #[case(ymm6, zmm6)]
    #[case(ymm7, zmm7)]
    #[case(ymm8, zmm8)]
    #[case(ymm9, zmm9)]
    #[case(ymm10, zmm10)]
    #[case(ymm11, zmm11)]
    #[case(ymm12, zmm12)]
    #[case(ymm13, zmm13)]
    #[case(ymm14, zmm14)]
    #[case(ymm15, zmm15)]
    #[case(zmm0, zmm0)]
    #[case(zmm1, zmm1)]
    #[case(zmm2, zmm2)]
    #[case(zmm3, zmm3)]
    #[case(zmm4, zmm4)]
    #[case(zmm5, zmm5)]
    #[case(zmm6, zmm6)]
    #[case(zmm7, zmm7)]
    #[case(zmm8, zmm8)]
    #[case(zmm9, zmm9)]
    #[case(zmm10, zmm10)]
    #[case(zmm11, zmm11)]
    #[case(zmm12, zmm12)]
    #[case(zmm13, zmm13)]
    #[case(zmm14, zmm14)]
    #[case(zmm15, zmm15)]
    fn extend_test(#[case] input: Register, #[case] expected: Register) {
        assert_eq!(input.extend(), expected);
    }

    #[rstest]
    #[case(Register::st0, zydis::Register::ST0)]
    #[case(Register::st1, zydis::Register::ST1)]
    #[case(Register::st2, zydis::Register::ST2)]
    #[case(Register::st3, zydis::Register::ST3)]
    #[case(Register::st4, zydis::Register::ST4)]
    #[case(Register::st5, zydis::Register::ST5)]
    #[case(Register::st6, zydis::Register::ST6)]
    #[case(Register::st7, zydis::Register::ST7)]
    #[case(Register::rax, zydis::Register::RAX)]
    #[case(Register::rbx, zydis::Register::RBX)]
    #[case(Register::rcx, zydis::Register::RCX)]
    #[case(Register::rdx, zydis::Register::RDX)]
    #[case(Register::rsi, zydis::Register::RSI)]
    #[case(Register::rdi, zydis::Register::RDI)]
    #[case(Register::rbp, zydis::Register::RBP)]
    #[case(Register::rsp, zydis::Register::RSP)]
    #[case(Register::r8, zydis::Register::R8)]
    #[case(Register::r9, zydis::Register::R9)]
    #[case(Register::r10, zydis::Register::R10)]
    #[case(Register::r11, zydis::Register::R11)]
    #[case(Register::r12, zydis::Register::R12)]
    #[case(Register::r13, zydis::Register::R13)]
    #[case(Register::r14, zydis::Register::R14)]
    #[case(Register::r15, zydis::Register::R15)]
    #[case(Register::xmm0, zydis::Register::XMM0)]
    #[case(Register::xmm1, zydis::Register::XMM1)]
    #[case(Register::xmm2, zydis::Register::XMM2)]
    #[case(Register::xmm3, zydis::Register::XMM3)]
    #[case(Register::xmm4, zydis::Register::XMM4)]
    #[case(Register::xmm5, zydis::Register::XMM5)]
    #[case(Register::xmm6, zydis::Register::XMM6)]
    #[case(Register::xmm7, zydis::Register::XMM7)]
    #[case(Register::xmm8, zydis::Register::XMM8)]
    #[case(Register::xmm9, zydis::Register::XMM9)]
    #[case(Register::xmm10, zydis::Register::XMM10)]
    #[case(Register::xmm11, zydis::Register::XMM11)]
    #[case(Register::xmm12, zydis::Register::XMM12)]
    #[case(Register::xmm13, zydis::Register::XMM13)]
    #[case(Register::xmm14, zydis::Register::XMM14)]
    #[case(Register::xmm15, zydis::Register::XMM15)]
    #[case(Register::ymm0, zydis::Register::YMM0)]
    #[case(Register::ymm1, zydis::Register::YMM1)]
    #[case(Register::ymm2, zydis::Register::YMM2)]
    #[case(Register::ymm3, zydis::Register::YMM3)]
    #[case(Register::ymm4, zydis::Register::YMM4)]
    #[case(Register::ymm5, zydis::Register::YMM5)]
    #[case(Register::ymm6, zydis::Register::YMM6)]
    #[case(Register::ymm7, zydis::Register::YMM7)]
    #[case(Register::ymm8, zydis::Register::YMM8)]
    #[case(Register::ymm9, zydis::Register::YMM9)]
    #[case(Register::ymm10, zydis::Register::YMM10)]
    #[case(Register::ymm11, zydis::Register::YMM11)]
    #[case(Register::ymm12, zydis::Register::YMM12)]
    #[case(Register::ymm13, zydis::Register::YMM13)]
    #[case(Register::ymm14, zydis::Register::YMM14)]
    #[case(Register::ymm15, zydis::Register::YMM15)]
    #[case(Register::zmm0, zydis::Register::ZMM0)]
    #[case(Register::zmm1, zydis::Register::ZMM1)]
    #[case(Register::zmm2, zydis::Register::ZMM2)]
    #[case(Register::zmm3, zydis::Register::ZMM3)]
    #[case(Register::zmm4, zydis::Register::ZMM4)]
    #[case(Register::zmm5, zydis::Register::ZMM5)]
    #[case(Register::zmm6, zydis::Register::ZMM6)]
    #[case(Register::zmm7, zydis::Register::ZMM7)]
    #[case(Register::zmm8, zydis::Register::ZMM8)]
    #[case(Register::zmm9, zydis::Register::ZMM9)]
    #[case(Register::zmm10, zydis::Register::ZMM10)]
    #[case(Register::zmm11, zydis::Register::ZMM11)]
    #[case(Register::zmm12, zydis::Register::ZMM12)]
    #[case(Register::zmm13, zydis::Register::ZMM13)]
    #[case(Register::zmm14, zydis::Register::ZMM14)]
    #[case(Register::zmm15, zydis::Register::ZMM15)]
    fn to_zydis_test(#[case] register: Register, #[case] expected_zydis_reg: zydis::Register) {
        assert_eq!(register.to_zydis(), expected_zydis_reg);
    }
}
