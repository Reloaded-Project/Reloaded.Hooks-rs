//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;
pub mod jit;
pub mod instructions {
    pub mod add_immediate;
    pub mod errors;
    pub mod ldr_immediate_post_indexed;
    pub mod ldr_immediate_unsigned_offset;
    pub mod orr;
    pub mod str_immediate_pre_indexed;
    pub mod sub_immediate;
}

pub(crate) mod jit_instructions {
    pub mod branch_relative;
    pub mod mov;
    pub mod mov_from_stack;
    pub mod pop;
    pub mod push;
    pub mod stackalloc;
}

#[cfg(test)]
pub(crate) mod test_helpers;
