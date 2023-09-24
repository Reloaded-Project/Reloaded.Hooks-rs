use reloaded_hooks_portable::api::traits::register_info::{
    KnownRegisterType, KnownRegisterType::*, RegisterInfo,
};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AllRegisters {
    // 32 bit general purpose registers
    w0,
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
    w31,

    // 64 bit general purpose registers
    x0,
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
    SP,

    // 128 bit SIMD registers
    v0,
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
    v31,
}

impl AllRegisters {
    pub fn register_number(&self) -> u32 {
        match *self {
            AllRegisters::w0 | AllRegisters::x0 | AllRegisters::v0 => 0,
            AllRegisters::w1 | AllRegisters::x1 | AllRegisters::v1 => 1,
            AllRegisters::w2 | AllRegisters::x2 | AllRegisters::v2 => 2,
            AllRegisters::w3 | AllRegisters::x3 | AllRegisters::v3 => 3,
            AllRegisters::w4 | AllRegisters::x4 | AllRegisters::v4 => 4,
            AllRegisters::w5 | AllRegisters::x5 | AllRegisters::v5 => 5,
            AllRegisters::w6 | AllRegisters::x6 | AllRegisters::v6 => 6,
            AllRegisters::w7 | AllRegisters::x7 | AllRegisters::v7 => 7,
            AllRegisters::w8 | AllRegisters::x8 | AllRegisters::v8 => 8,
            AllRegisters::w9 | AllRegisters::x9 | AllRegisters::v9 => 9,
            AllRegisters::w10 | AllRegisters::x10 | AllRegisters::v10 => 10,
            AllRegisters::w11 | AllRegisters::x11 | AllRegisters::v11 => 11,
            AllRegisters::w12 | AllRegisters::x12 | AllRegisters::v12 => 12,
            AllRegisters::w13 | AllRegisters::x13 | AllRegisters::v13 => 13,
            AllRegisters::w14 | AllRegisters::x14 | AllRegisters::v14 => 14,
            AllRegisters::w15 | AllRegisters::x15 | AllRegisters::v15 => 15,
            AllRegisters::w16 | AllRegisters::x16 | AllRegisters::v16 => 16,
            AllRegisters::w17 | AllRegisters::x17 | AllRegisters::v17 => 17,
            AllRegisters::w18 | AllRegisters::x18 | AllRegisters::v18 => 18,
            AllRegisters::w19 | AllRegisters::x19 | AllRegisters::v19 => 19,
            AllRegisters::w20 | AllRegisters::x20 | AllRegisters::v20 => 20,
            AllRegisters::w21 | AllRegisters::x21 | AllRegisters::v21 => 21,
            AllRegisters::w22 | AllRegisters::x22 | AllRegisters::v22 => 22,
            AllRegisters::w23 | AllRegisters::x23 | AllRegisters::v23 => 23,
            AllRegisters::w24 | AllRegisters::x24 | AllRegisters::v24 => 24,
            AllRegisters::w25 | AllRegisters::x25 | AllRegisters::v25 => 25,
            AllRegisters::w26 | AllRegisters::x26 | AllRegisters::v26 => 26,
            AllRegisters::w27 | AllRegisters::x27 | AllRegisters::v27 => 27,
            AllRegisters::w28 | AllRegisters::x28 | AllRegisters::v28 => 28,
            AllRegisters::w29 | AllRegisters::x29 | AllRegisters::v29 => 29,
            AllRegisters::w30 | AllRegisters::LR | AllRegisters::v30 => 30,
            AllRegisters::w31 | AllRegisters::SP | AllRegisters::v31 => 31,
        }
    }

    // Implement size(), is_32(), is_64() etc. functions
    pub fn size(&self) -> usize {
        match *self {
            // 32-bit GP registers
            AllRegisters::w0 => 4,
            AllRegisters::w1 => 4,
            AllRegisters::w2 => 4,
            AllRegisters::w3 => 4,
            AllRegisters::w4 => 4,
            AllRegisters::w5 => 4,
            AllRegisters::w6 => 4,
            AllRegisters::w7 => 4,
            AllRegisters::w8 => 4,
            AllRegisters::w9 => 4,
            AllRegisters::w10 => 4,
            AllRegisters::w11 => 4,
            AllRegisters::w12 => 4,
            AllRegisters::w13 => 4,
            AllRegisters::w14 => 4,
            AllRegisters::w15 => 4,
            AllRegisters::w16 => 4,
            AllRegisters::w17 => 4,
            AllRegisters::w18 => 4,
            AllRegisters::w19 => 4,
            AllRegisters::w20 => 4,
            AllRegisters::w21 => 4,
            AllRegisters::w22 => 4,
            AllRegisters::w23 => 4,
            AllRegisters::w24 => 4,
            AllRegisters::w25 => 4,
            AllRegisters::w26 => 4,
            AllRegisters::w27 => 4,
            AllRegisters::w28 => 4,
            AllRegisters::w29 => 4,
            AllRegisters::w30 => 4,
            AllRegisters::w31 => 4,

            // 64-bit GP registers
            AllRegisters::x0 => 8,
            AllRegisters::x1 => 8,
            AllRegisters::x2 => 8,
            AllRegisters::x3 => 8,
            AllRegisters::x4 => 8,
            AllRegisters::x5 => 8,
            AllRegisters::x6 => 8,
            AllRegisters::x7 => 8,
            AllRegisters::x8 => 8,
            AllRegisters::x9 => 8,
            AllRegisters::x10 => 8,
            AllRegisters::x11 => 8,
            AllRegisters::x12 => 8,
            AllRegisters::x13 => 8,
            AllRegisters::x14 => 8,
            AllRegisters::x15 => 8,
            AllRegisters::x16 => 8,
            AllRegisters::x17 => 8,
            AllRegisters::x18 => 8,
            AllRegisters::x19 => 8,
            AllRegisters::x20 => 8,
            AllRegisters::x21 => 8,
            AllRegisters::x22 => 8,
            AllRegisters::x23 => 8,
            AllRegisters::x24 => 8,
            AllRegisters::x25 => 8,
            AllRegisters::x26 => 8,
            AllRegisters::x27 => 8,
            AllRegisters::x28 => 8,
            AllRegisters::x29 => 8,
            AllRegisters::LR => 8,
            AllRegisters::SP => 8,

            // 128-bit SIMD registers
            AllRegisters::v0 => 16,
            AllRegisters::v1 => 16,
            AllRegisters::v2 => 16,
            AllRegisters::v3 => 16,
            AllRegisters::v4 => 16,
            AllRegisters::v5 => 16,
            AllRegisters::v6 => 16,
            AllRegisters::v7 => 16,
            AllRegisters::v8 => 16,
            AllRegisters::v9 => 16,
            AllRegisters::v10 => 16,
            AllRegisters::v11 => 16,
            AllRegisters::v12 => 16,
            AllRegisters::v13 => 16,
            AllRegisters::v14 => 16,
            AllRegisters::v15 => 16,
            AllRegisters::v16 => 16,
            AllRegisters::v17 => 16,
            AllRegisters::v18 => 16,
            AllRegisters::v19 => 16,
            AllRegisters::v20 => 16,
            AllRegisters::v21 => 16,
            AllRegisters::v22 => 16,
            AllRegisters::v23 => 16,
            AllRegisters::v24 => 16,
            AllRegisters::v25 => 16,
            AllRegisters::v26 => 16,
            AllRegisters::v27 => 16,
            AllRegisters::v28 => 16,
            AllRegisters::v29 => 16,
            AllRegisters::v30 => 16,
            AllRegisters::v31 => 16,
        }
    }

    pub fn is_32(&self) -> bool {
        matches!(
            self,
            AllRegisters::w0
                | AllRegisters::w1
                | AllRegisters::w2
                | AllRegisters::w3
                | AllRegisters::w4
                | AllRegisters::w5
                | AllRegisters::w6
                | AllRegisters::w7
                | AllRegisters::w8
                | AllRegisters::w9
                | AllRegisters::w10
                | AllRegisters::w11
                | AllRegisters::w12
                | AllRegisters::w13
                | AllRegisters::w14
                | AllRegisters::w15
                | AllRegisters::w16
                | AllRegisters::w17
                | AllRegisters::w18
                | AllRegisters::w19
                | AllRegisters::w20
                | AllRegisters::w21
                | AllRegisters::w22
                | AllRegisters::w23
                | AllRegisters::w24
                | AllRegisters::w25
                | AllRegisters::w26
                | AllRegisters::w27
                | AllRegisters::w28
                | AllRegisters::w29
                | AllRegisters::w30
                | AllRegisters::w31
        )
    }

    pub fn is_64(&self) -> bool {
        matches!(
            self,
            AllRegisters::x0
                | AllRegisters::x1
                | AllRegisters::x2
                | AllRegisters::x3
                | AllRegisters::x4
                | AllRegisters::x5
                | AllRegisters::x6
                | AllRegisters::x7
                | AllRegisters::x8
                | AllRegisters::x9
                | AllRegisters::x10
                | AllRegisters::x11
                | AllRegisters::x12
                | AllRegisters::x13
                | AllRegisters::x14
                | AllRegisters::x15
                | AllRegisters::x16
                | AllRegisters::x17
                | AllRegisters::x18
                | AllRegisters::x19
                | AllRegisters::x20
                | AllRegisters::x21
                | AllRegisters::x22
                | AllRegisters::x23
                | AllRegisters::x24
                | AllRegisters::x25
                | AllRegisters::x26
                | AllRegisters::x27
                | AllRegisters::x28
                | AllRegisters::x29
                | AllRegisters::LR
                | AllRegisters::SP
        )
    }

    pub fn is_128(&self) -> bool {
        matches!(
            self,
            AllRegisters::v0
                | AllRegisters::v1
                | AllRegisters::v2
                | AllRegisters::v3
                | AllRegisters::v4
                | AllRegisters::v5
                | AllRegisters::v6
                | AllRegisters::v7
                | AllRegisters::v8
                | AllRegisters::v9
                | AllRegisters::v10
                | AllRegisters::v11
                | AllRegisters::v12
                | AllRegisters::v13
                | AllRegisters::v14
                | AllRegisters::v15
                | AllRegisters::v16
                | AllRegisters::v17
                | AllRegisters::v18
                | AllRegisters::v19
                | AllRegisters::v20
                | AllRegisters::v21
                | AllRegisters::v22
                | AllRegisters::v23
                | AllRegisters::v24
                | AllRegisters::v25
                | AllRegisters::v26
                | AllRegisters::v27
                | AllRegisters::v28
                | AllRegisters::v29
                | AllRegisters::v30
                | AllRegisters::v31
        )
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
}
