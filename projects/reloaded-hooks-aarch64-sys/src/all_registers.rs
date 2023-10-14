use core::mem::transmute;

use derive_enum_all_values::AllValues;
use reloaded_hooks_portable::api::traits::register_info::{
    KnownRegisterType, KnownRegisterType::*, RegisterInfo,
};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, AllValues, Default)]
pub enum AllRegisters {
    // Range 0b00000 - 0b11111 (0-31)
    // 32 bit general purpose registers
    w0, // 0b00000
    w1,
    w2,
    w3,
    w4,
    w5,
    w6,
    w7,
    w8,
    w9,
    w10,
    w11,
    w12,
    w13,
    w14,
    w15,
    w16,
    w17,
    w18,
    w19,
    w20,
    w21,
    w22,
    w23,
    w24,
    w25,
    w26,
    w27,
    w28,
    w29,
    w30,
    w31, // 0b11111

    // Range 0b100000 - 0b111111 (0-31)
    // XOR bit 0b100000 to toggle between w and x registers.
    // 64 bit general purpose registers
    #[default]
    x0, // 0b100000
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15,
    x16,
    x17,
    x18,
    x19,
    x20,
    x21,
    x22,
    x23,
    x24,
    x25,
    x26,
    x27,
    x28,
    x29,
    LR,
    SP, // 0b111111

    // 128 bit SIMD registers
    v0, // 0b1000000
    v1,
    v2,
    v3,
    v4,
    v5,
    v6,
    v7,
    v8,
    v9,
    v10,
    v11,
    v12,
    v13,
    v14,
    v15,
    v16,
    v17,
    v18,
    v19,
    v20,
    v21,
    v22,
    v23,
    v24,
    v25,
    v26,
    v27,
    v28,
    v29,
    v30,
    v31, // 0b1011111
}

impl AllRegisters {
    pub fn register_number(&self) -> u32 {
        // Mask the lower 5 bits to get the register number
        (*self as u32) & 0b11111
    }

    // Implement size(), is_32(), is_64() etc. functions
    pub fn size(&self) -> usize {
        let register_value = *self as u32;

        if register_value & 0b1000000 != 0 {
            16
        } else if register_value & 0b100000 != 0 {
            8
        } else {
            4
        }
    }

    pub fn is_32(&self) -> bool {
        *self as u32 <= 0b11111
    }

    pub fn is_64(&self) -> bool {
        *self as u32 & 0b100000 != 0
    }

    pub fn is_128(&self) -> bool {
        *self as u32 & 0b1000000 != 0
    }

    /// Shrinks a 64-bit general purpose register to a 32-bit general purpose register.
    /// Note: This will shrink to 32-bit GPR with matching number even if the register is not a 64-bit GPR.
    pub fn shrink_to32(&self) -> Self {
        let value = *self as u32;
        if self.is_64() {
            return unsafe { transmute((value as u8) & 0b11111) };
        }

        *self
    }
}

impl RegisterInfo for AllRegisters {
    fn size_in_bytes(&self) -> usize {
        self.size()
    }

    fn is_stack_pointer(&self) -> bool {
        self == &AllRegisters::SP
    }

    fn register_type(&self) -> KnownRegisterType {
        if self.is_64() {
            GeneralPurpose64
        } else if self.is_32() {
            GeneralPurpose32
        } else if self.is_128() {
            Vector128
        } else {
            Unknown
        }
    }

    fn extend(&self) -> Self {
        let value = *self as u32;
        if self.is_32() {
            return unsafe { transmute((value + 0b100000) as u8) };
        }

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
    use crate::all_registers::AllRegisters;
    use crate::all_registers::AllRegisters::*;
    use crate::all_registers::KnownRegisterType::*;
    use reloaded_hooks_portable::api::traits::register_info::KnownRegisterType;
    use reloaded_hooks_portable::api::traits::register_info::RegisterInfo;
    use rstest::rstest;

    #[rstest]
    #[case(w0, 0)]
    #[case(w1, 1)]
    #[case(w2, 2)]
    #[case(w3, 3)]
    #[case(w4, 4)]
    #[case(w5, 5)]
    #[case(w6, 6)]
    #[case(w7, 7)]
    #[case(w8, 8)]
    #[case(w9, 9)]
    #[case(w10, 10)]
    #[case(w11, 11)]
    #[case(w12, 12)]
    #[case(w13, 13)]
    #[case(w14, 14)]
    #[case(w15, 15)]
    #[case(w16, 16)]
    #[case(w17, 17)]
    #[case(w18, 18)]
    #[case(w19, 19)]
    #[case(w20, 20)]
    #[case(w21, 21)]
    #[case(w22, 22)]
    #[case(w23, 23)]
    #[case(w24, 24)]
    #[case(w25, 25)]
    #[case(w26, 26)]
    #[case(w27, 27)]
    #[case(w28, 28)]
    #[case(w29, 29)]
    #[case(w30, 30)]
    #[case(w31, 31)]
    #[case(x0, 0)]
    #[case(x1, 1)]
    #[case(x2, 2)]
    #[case(x3, 3)]
    #[case(x4, 4)]
    #[case(x5, 5)]
    #[case(x6, 6)]
    #[case(x7, 7)]
    #[case(x8, 8)]
    #[case(x9, 9)]
    #[case(x10, 10)]
    #[case(x11, 11)]
    #[case(x12, 12)]
    #[case(x13, 13)]
    #[case(x14, 14)]
    #[case(x15, 15)]
    #[case(x16, 16)]
    #[case(x17, 17)]
    #[case(x18, 18)]
    #[case(x19, 19)]
    #[case(x20, 20)]
    #[case(x21, 21)]
    #[case(x22, 22)]
    #[case(x23, 23)]
    #[case(x24, 24)]
    #[case(x25, 25)]
    #[case(x26, 26)]
    #[case(x27, 27)]
    #[case(x28, 28)]
    #[case(x29, 29)]
    #[case(LR, 30)]
    #[case(SP, 31)]
    #[case(v0, 0)]
    #[case(v1, 1)]
    #[case(v2, 2)]
    #[case(v3, 3)]
    #[case(v4, 4)]
    #[case(v5, 5)]
    #[case(v6, 6)]
    #[case(v7, 7)]
    #[case(v8, 8)]
    #[case(v9, 9)]
    #[case(v10, 10)]
    #[case(v11, 11)]
    #[case(v12, 12)]
    #[case(v13, 13)]
    #[case(v14, 14)]
    #[case(v15, 15)]
    #[case(v16, 16)]
    #[case(v17, 17)]
    #[case(v18, 18)]
    #[case(v19, 19)]
    #[case(v20, 20)]
    #[case(v21, 21)]
    #[case(v22, 22)]
    #[case(v23, 23)]
    #[case(v24, 24)]
    #[case(v25, 25)]
    #[case(v26, 26)]
    #[case(v27, 27)]
    #[case(v28, 28)]
    #[case(v29, 29)]
    #[case(v30, 30)]
    #[case(v31, 31)]
    fn register_number(#[case] register: AllRegisters, #[case] expected_number: u32) {
        assert_eq!(register.register_number(), expected_number);
    }

    #[rstest]
    #[case(x1, 8)]
    #[case(x2, 8)]
    #[case(x3, 8)]
    #[case(x4, 8)]
    #[case(x5, 8)]
    #[case(x6, 8)]
    #[case(x7, 8)]
    #[case(x8, 8)]
    #[case(x9, 8)]
    #[case(x10, 8)]
    #[case(x11, 8)]
    #[case(x12, 8)]
    #[case(x13, 8)]
    #[case(x14, 8)]
    #[case(x15, 8)]
    #[case(x16, 8)]
    #[case(x17, 8)]
    #[case(x18, 8)]
    #[case(x19, 8)]
    #[case(x20, 8)]
    #[case(x21, 8)]
    #[case(x22, 8)]
    #[case(x23, 8)]
    #[case(x24, 8)]
    #[case(x25, 8)]
    #[case(x26, 8)]
    #[case(x27, 8)]
    #[case(x28, 8)]
    #[case(x29, 8)]
    #[case(LR, 8)]
    #[case(SP, 8)]
    #[case(w1, 4)]
    #[case(w2, 4)]
    #[case(w3, 4)]
    #[case(w4, 4)]
    #[case(w5, 4)]
    #[case(w6, 4)]
    #[case(w7, 4)]
    #[case(w8, 4)]
    #[case(w9, 4)]
    #[case(w10, 4)]
    #[case(w11, 4)]
    #[case(w12, 4)]
    #[case(w13, 4)]
    #[case(w14, 4)]
    #[case(w15, 4)]
    #[case(w16, 4)]
    #[case(w17, 4)]
    #[case(w18, 4)]
    #[case(w19, 4)]
    #[case(w20, 4)]
    #[case(w21, 4)]
    #[case(w22, 4)]
    #[case(w23, 4)]
    #[case(w24, 4)]
    #[case(w25, 4)]
    #[case(w26, 4)]
    #[case(w27, 4)]
    #[case(w28, 4)]
    #[case(w29, 4)]
    #[case(w30, 4)]
    #[case(w31, 4)]
    #[case(v1, 16)]
    #[case(v2, 16)]
    #[case(v3, 16)]
    #[case(v4, 16)]
    #[case(v5, 16)]
    #[case(v6, 16)]
    #[case(v7, 16)]
    #[case(v8, 16)]
    #[case(v9, 16)]
    #[case(v10, 16)]
    #[case(v11, 16)]
    #[case(v12, 16)]
    #[case(v13, 16)]
    #[case(v14, 16)]
    #[case(v15, 16)]
    #[case(v16, 16)]
    #[case(v17, 16)]
    #[case(v18, 16)]
    #[case(v19, 16)]
    #[case(v20, 16)]
    #[case(v21, 16)]
    #[case(v22, 16)]
    #[case(v23, 16)]
    #[case(v24, 16)]
    #[case(v25, 16)]
    #[case(v26, 16)]
    #[case(v27, 16)]
    #[case(v28, 16)]
    #[case(v29, 16)]
    #[case(v30, 16)]
    #[case(v31, 16)]
    fn register_size(#[case] register: AllRegisters, #[case] expected_size: usize) {
        assert_eq!(register.size(), expected_size);
    }

    #[rstest]
    #[case(w0, true, false, false)]
    #[case(w1, true, false, false)]
    #[case(w2, true, false, false)]
    #[case(w3, true, false, false)]
    #[case(w4, true, false, false)]
    #[case(w5, true, false, false)]
    #[case(w6, true, false, false)]
    #[case(w7, true, false, false)]
    #[case(w8, true, false, false)]
    #[case(w9, true, false, false)]
    #[case(w10, true, false, false)]
    #[case(w11, true, false, false)]
    #[case(w12, true, false, false)]
    #[case(w13, true, false, false)]
    #[case(w14, true, false, false)]
    #[case(w15, true, false, false)]
    #[case(w16, true, false, false)]
    #[case(w17, true, false, false)]
    #[case(w18, true, false, false)]
    #[case(w19, true, false, false)]
    #[case(w20, true, false, false)]
    #[case(w21, true, false, false)]
    #[case(w22, true, false, false)]
    #[case(w23, true, false, false)]
    #[case(w24, true, false, false)]
    #[case(w25, true, false, false)]
    #[case(w26, true, false, false)]
    #[case(w27, true, false, false)]
    #[case(w28, true, false, false)]
    #[case(w29, true, false, false)]
    #[case(w30, true, false, false)]
    #[case(w31, true, false, false)]
    #[case(x0, false, true, false)]
    #[case(x1, false, true, false)]
    #[case(x2, false, true, false)]
    #[case(x3, false, true, false)]
    #[case(x4, false, true, false)]
    #[case(x5, false, true, false)]
    #[case(x6, false, true, false)]
    #[case(x7, false, true, false)]
    #[case(x8, false, true, false)]
    #[case(x9, false, true, false)]
    #[case(x10, false, true, false)]
    #[case(x11, false, true, false)]
    #[case(x12, false, true, false)]
    #[case(x13, false, true, false)]
    #[case(x14, false, true, false)]
    #[case(x15, false, true, false)]
    #[case(x16, false, true, false)]
    #[case(x17, false, true, false)]
    #[case(x18, false, true, false)]
    #[case(x19, false, true, false)]
    #[case(x20, false, true, false)]
    #[case(x21, false, true, false)]
    #[case(x22, false, true, false)]
    #[case(x23, false, true, false)]
    #[case(x24, false, true, false)]
    #[case(x25, false, true, false)]
    #[case(x26, false, true, false)]
    #[case(x27, false, true, false)]
    #[case(x28, false, true, false)]
    #[case(x29, false, true, false)]
    #[case(LR, false, true, false)]
    #[case(SP, false, true, false)]
    #[case(v0, false, false, true)]
    #[case(v1, false, false, true)]
    #[case(v2, false, false, true)]
    #[case(v3, false, false, true)]
    #[case(v4, false, false, true)]
    #[case(v5, false, false, true)]
    #[case(v6, false, false, true)]
    #[case(v7, false, false, true)]
    #[case(v8, false, false, true)]
    #[case(v9, false, false, true)]
    #[case(v10, false, false, true)]
    #[case(v11, false, false, true)]
    #[case(v12, false, false, true)]
    #[case(v13, false, false, true)]
    #[case(v14, false, false, true)]
    #[case(v15, false, false, true)]
    #[case(v16, false, false, true)]
    #[case(v17, false, false, true)]
    #[case(v18, false, false, true)]
    #[case(v19, false, false, true)]
    #[case(v20, false, false, true)]
    #[case(v21, false, false, true)]
    #[case(v22, false, false, true)]
    #[case(v23, false, false, true)]
    #[case(v24, false, false, true)]
    #[case(v25, false, false, true)]
    #[case(v26, false, false, true)]
    #[case(v27, false, false, true)]
    #[case(v28, false, false, true)]
    #[case(v29, false, false, true)]
    #[case(v30, false, false, true)]
    #[case(v31, false, false, true)]
    fn register_bit_width(
        #[case] register: AllRegisters,
        #[case] is_32: bool,
        #[case] is_64: bool,
        #[case] is_128: bool,
    ) {
        assert_eq!(register.is_32(), is_32);
        assert_eq!(register.is_64(), is_64);
        assert_eq!(register.is_128(), is_128);
    }

    #[rstest]
    #[case(w0, GeneralPurpose32)]
    #[case(w1, GeneralPurpose32)]
    #[case(w2, GeneralPurpose32)]
    #[case(w3, GeneralPurpose32)]
    #[case(w4, GeneralPurpose32)]
    #[case(w5, GeneralPurpose32)]
    #[case(w6, GeneralPurpose32)]
    #[case(w7, GeneralPurpose32)]
    #[case(w8, GeneralPurpose32)]
    #[case(w9, GeneralPurpose32)]
    #[case(w10, GeneralPurpose32)]
    #[case(w11, GeneralPurpose32)]
    #[case(w12, GeneralPurpose32)]
    #[case(w13, GeneralPurpose32)]
    #[case(w14, GeneralPurpose32)]
    #[case(w15, GeneralPurpose32)]
    #[case(w16, GeneralPurpose32)]
    #[case(w17, GeneralPurpose32)]
    #[case(w18, GeneralPurpose32)]
    #[case(w19, GeneralPurpose32)]
    #[case(w20, GeneralPurpose32)]
    #[case(w21, GeneralPurpose32)]
    #[case(w22, GeneralPurpose32)]
    #[case(w23, GeneralPurpose32)]
    #[case(w24, GeneralPurpose32)]
    #[case(w25, GeneralPurpose32)]
    #[case(w26, GeneralPurpose32)]
    #[case(w27, GeneralPurpose32)]
    #[case(w28, GeneralPurpose32)]
    #[case(w29, GeneralPurpose32)]
    #[case(w30, GeneralPurpose32)]
    #[case(w31, GeneralPurpose32)]
    #[case(x0, GeneralPurpose64)]
    #[case(x1, GeneralPurpose64)]
    #[case(x2, GeneralPurpose64)]
    #[case(x3, GeneralPurpose64)]
    #[case(x4, GeneralPurpose64)]
    #[case(x5, GeneralPurpose64)]
    #[case(x6, GeneralPurpose64)]
    #[case(x7, GeneralPurpose64)]
    #[case(x8, GeneralPurpose64)]
    #[case(x9, GeneralPurpose64)]
    #[case(x10, GeneralPurpose64)]
    #[case(x11, GeneralPurpose64)]
    #[case(x12, GeneralPurpose64)]
    #[case(x13, GeneralPurpose64)]
    #[case(x14, GeneralPurpose64)]
    #[case(x15, GeneralPurpose64)]
    #[case(x16, GeneralPurpose64)]
    #[case(x17, GeneralPurpose64)]
    #[case(x18, GeneralPurpose64)]
    #[case(x19, GeneralPurpose64)]
    #[case(x20, GeneralPurpose64)]
    #[case(x21, GeneralPurpose64)]
    #[case(x22, GeneralPurpose64)]
    #[case(x23, GeneralPurpose64)]
    #[case(x24, GeneralPurpose64)]
    #[case(x25, GeneralPurpose64)]
    #[case(x26, GeneralPurpose64)]
    #[case(x27, GeneralPurpose64)]
    #[case(x28, GeneralPurpose64)]
    #[case(x29, GeneralPurpose64)]
    #[case(LR, GeneralPurpose64)]
    #[case(SP, GeneralPurpose64)]
    #[case(v0, Vector128)]
    #[case(v1, Vector128)]
    #[case(v2, Vector128)]
    #[case(v3, Vector128)]
    #[case(v4, Vector128)]
    #[case(v5, Vector128)]
    #[case(v6, Vector128)]
    #[case(v7, Vector128)]
    #[case(v8, Vector128)]
    #[case(v9, Vector128)]
    #[case(v10, Vector128)]
    #[case(v11, Vector128)]
    #[case(v12, Vector128)]
    #[case(v13, Vector128)]
    #[case(v14, Vector128)]
    #[case(v15, Vector128)]
    #[case(v16, Vector128)]
    #[case(v17, Vector128)]
    #[case(v18, Vector128)]
    #[case(v19, Vector128)]
    #[case(v20, Vector128)]
    #[case(v21, Vector128)]
    #[case(v22, Vector128)]
    #[case(v23, Vector128)]
    #[case(v24, Vector128)]
    #[case(v25, Vector128)]
    #[case(v26, Vector128)]
    #[case(v27, Vector128)]
    #[case(v28, Vector128)]
    #[case(v29, Vector128)]
    #[case(v30, Vector128)]
    #[case(v31, Vector128)]
    fn register_type(#[case] register: AllRegisters, #[case] expected_type: KnownRegisterType) {
        assert_eq!(register.register_type(), expected_type);
    }

    #[rstest]
    #[case(w0, x0)]
    #[case(w1, x1)]
    #[case(w2, x2)]
    #[case(w3, x3)]
    #[case(w4, x4)]
    #[case(w5, x5)]
    #[case(w6, x6)]
    #[case(w7, x7)]
    #[case(w8, x8)]
    #[case(w9, x9)]
    #[case(w10, x10)]
    #[case(w11, x11)]
    #[case(w12, x12)]
    #[case(w13, x13)]
    #[case(w14, x14)]
    #[case(w15, x15)]
    #[case(w16, x16)]
    #[case(w17, x17)]
    #[case(w18, x18)]
    #[case(w19, x19)]
    #[case(w20, x20)]
    #[case(w21, x21)]
    #[case(w22, x22)]
    #[case(w23, x23)]
    #[case(w24, x24)]
    #[case(w25, x25)]
    #[case(w26, x26)]
    #[case(w27, x27)]
    #[case(w28, x28)]
    #[case(w29, x29)]
    #[case(w30, LR)]
    #[case(w31, SP)]
    fn extend_register(#[case] input: AllRegisters, #[case] expected: AllRegisters) {
        assert_eq!(input.extend(), expected);
    }

    #[rstest]
    #[case(x0)]
    #[case(x1)]
    #[case(x2)]
    #[case(x3)]
    #[case(x4)]
    #[case(x5)]
    #[case(x6)]
    #[case(x7)]
    #[case(x8)]
    #[case(x9)]
    #[case(x10)]
    #[case(x11)]
    #[case(x12)]
    #[case(x13)]
    #[case(x14)]
    #[case(x15)]
    #[case(x16)]
    #[case(x17)]
    #[case(x18)]
    #[case(x19)]
    #[case(x20)]
    #[case(x21)]
    #[case(x22)]
    #[case(x23)]
    #[case(x24)]
    #[case(x25)]
    #[case(x26)]
    #[case(x27)]
    #[case(x28)]
    #[case(x29)]
    #[case(LR)]
    #[case(SP)]
    #[case(v0)]
    #[case(v1)]
    #[case(v2)]
    #[case(v3)]
    #[case(v4)]
    #[case(v5)]
    #[case(v6)]
    #[case(v7)]
    #[case(v8)]
    #[case(v9)]
    #[case(v10)]
    #[case(v11)]
    #[case(v12)]
    #[case(v13)]
    #[case(v14)]
    #[case(v15)]
    #[case(v16)]
    #[case(v17)]
    #[case(v18)]
    #[case(v19)]
    #[case(v20)]
    #[case(v21)]
    #[case(v22)]
    #[case(v23)]
    #[case(v24)]
    #[case(v25)]
    #[case(v26)]
    #[case(v27)]
    #[case(v28)]
    #[case(v29)]
    #[case(v30)]
    #[case(v31)]
    fn extend_non_w_registers(#[case] register: AllRegisters) {
        assert_eq!(register.extend(), register);
    }
}
