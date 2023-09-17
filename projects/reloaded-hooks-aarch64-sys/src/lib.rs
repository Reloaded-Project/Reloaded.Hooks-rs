//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;
pub mod jit;
pub mod opcodes {
    pub mod add_immediate;
    pub mod ldr_immediate_post_indexed;
    pub mod ldr_immediate_unsigned_offset;
    pub mod orr;
    pub mod str_immediate_pre_indexed;
    pub mod sub_immediate;
}
