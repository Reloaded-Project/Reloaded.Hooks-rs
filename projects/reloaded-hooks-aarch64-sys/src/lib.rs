//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all of the functional registers for the AArch64 architecture.
#[cfg(not(tarpaulin_include))]
pub mod all_registers;

/// Contains the Just in Time Assembler that integrates with reloaded-hooks-rs.
pub mod jit;

/// Contains Code Rewriter which translates code from one address to another.
pub mod rewriter;

/// Rewriting the code from one address to another!
pub(crate) mod code_rewriter {
    pub mod aarch64_rewriter;
    pub mod helpers;
    pub mod instruction_rewrite_result;

    pub(crate) mod instructions {
        pub mod adr;
        pub mod b;
        pub mod b_cond;
        pub mod cbz;
        pub mod ldr_literal;
        pub mod tbz;
    }
}

/// This namespace contains the raw instruction encodings for various
/// AArch64 instructions.
pub(crate) mod instructions {
    pub mod add_immediate;
    pub mod adr;
    pub mod b;
    pub mod bcc;
    pub mod branch_register;
    pub mod cbz;
    pub mod errors;
    pub mod ldp_immediate;
    pub mod ldr_immediate_post_indexed;
    pub mod ldr_immediate_unsigned_offset;
    pub mod ldr_literal;
    pub mod mov_immediate;
    pub mod orr;
    pub mod orr_vector;
    pub mod stp_immediate;
    pub mod str_immediate_pre_indexed;
    pub mod sub_immediate;
    pub mod tbz;
}

/// This namespace contains the code for encoding the JIT instructions
/// using the raw instructions in the [`crate::instructions`] namespace.
pub(crate) mod jit_instructions {
    pub mod branch_absolute;
    pub mod branch_ip_relative;
    pub mod branch_relative;
    pub mod jump_absolute_indirect;
    pub mod load_pc_relative_address;
    pub mod load_pc_relative_value;
    pub mod mov;
    pub mod mov_from_stack;
    pub mod mov_two_from_stack;
    pub mod multi_pop;
    pub mod multi_push;
    pub mod pop;
    pub mod pop_two;
    pub mod push;
    pub mod push_constant;
    pub mod push_stack;
    pub mod push_two;
    pub mod ret;
    pub mod stackalloc;
    pub mod xchg;
}

#[cfg(test)]
pub(crate) mod test_helpers;
