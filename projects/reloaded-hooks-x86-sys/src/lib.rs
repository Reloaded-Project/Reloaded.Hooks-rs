//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;
pub mod jit_common;

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub mod jit_conversions_common;

pub mod preset_calling_convention;
pub mod x86 {
    pub mod register;
    pub use register::Register;
    pub mod jit;
}
pub mod x64 {
    pub mod register;
    pub use register::Register;
    pub mod jit;
}
