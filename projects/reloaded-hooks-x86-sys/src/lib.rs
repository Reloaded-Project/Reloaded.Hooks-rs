//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;

pub(crate) mod common {

    pub(crate) mod util {
        pub mod get_stolen_instructions;
        pub mod invert_branch_condition;
    }

    pub mod disasm;
    pub mod jit_common;

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    pub mod jit_conversions_common;
}

/// Contains the public namespaces for x86
pub mod x86 {
    pub(crate) mod code_rewriter {
        pub mod x86_rewriter;
    }

    pub mod preset_calling_convention;
    pub mod register;
    pub use register::Register;
    pub mod jit;
}

/// Contains the public namespaces for x64
pub mod x64 {
    pub(crate) mod code_rewriter {
        pub mod x64_rewriter;
    }

    pub mod register;
    pub use register::Register;
    pub mod jit;
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
