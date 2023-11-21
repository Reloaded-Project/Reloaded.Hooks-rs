//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;

/// Contains the public namespaces for x86
pub mod x86 {
    pub mod length_disassembler;
    pub mod preset_calling_convention;
    pub mod register;
    pub use register::Register;
    pub mod jit;
    pub mod rewriter;
}

/// Contains the public namespaces for x64
pub mod x64 {
    pub mod length_disassembler;
    pub mod register;
    pub mod rewriter;
    pub use register::Register;
    pub mod jit;
}

pub(crate) mod common {

    pub(crate) mod util {
        pub mod get_instruction_length;
        pub mod get_stolen_instructions;
        pub mod iced_extensions;
        pub mod invert_branch_condition;

        #[cfg(test)]
        pub(crate) mod test_utilities;
    }

    pub mod rewriter {
        pub mod code_rewriter;
        pub mod patches;
    }
    pub mod jit_common;

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    pub mod jit_conversions_common;
}

/// This namespace contains the code for encoding the JIT instructions
pub(crate) mod instructions {
    pub mod call_absolute;
    pub mod call_ip_relative;
    pub mod call_relative;
    pub mod jump_absolute;
    pub mod jump_absolute_indirect;
    pub mod jump_ip_relative;
    pub mod jump_relative;
    pub mod mov;
    pub mod mov_from_stack;
    pub mod multi_pop;
    pub mod multi_push;
    pub mod pop;
    pub mod push;
    pub mod push_const;
    pub mod push_stack;
    pub mod ret;
    pub mod stack_alloc;
    pub mod xchg;
}
