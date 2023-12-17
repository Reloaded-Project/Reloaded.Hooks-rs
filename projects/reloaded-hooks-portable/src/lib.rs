//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all declarations that are exposed to library users.
pub mod api {

    /// The errors that can occur when generating a wrapper.
    pub mod errors {
        pub mod assembly_hook_error;
        pub mod inline_branch_error;
        pub mod wrapper_generation_error;
    }

    /// APIs for allocating buffers in given proximity.
    pub mod buffers {
        pub mod buffer_abstractions;
        pub mod default_buffer;
        pub mod default_buffer_factory;
    }

    /// Settings passed to other methodss
    pub mod settings {
        pub mod assembly_hook_settings;
        pub mod proximity_target;
    }

    /// Settings passed to other methodss
    pub mod hooks {
        pub mod assembly {
            pub mod assembly_hook;
        }
    }

    /// Platform and architecture specific integrations
    /// Note: Depends on STD crate, but implementation in crate is no-std.
    pub mod platforms {

        #[allow(warnings)]
        pub mod platform_functions;

        // The easiest OS to work with tbh
        #[cfg(target_os = "windows")]
        pub mod platform_functions_windows;

        // Good boys and girls that follow the unix standard go here
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
        pub mod platform_functions_unix;

        // and apple goes here...
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        pub mod platform_functions_apple;

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        pub(crate) mod platform_functions_mmap_rs;
    }

    /// Public API related to Just In Time Compilation
    pub mod jit {
        pub mod call_absolute_operation;
        pub mod call_relative_operation;
        pub mod call_rip_relative_operation;
        pub mod compiler;
        pub mod jump_absolute_indirect_operation;
        pub mod jump_absolute_operation;
        pub mod jump_relative_operation;
        pub mod jump_rip_relative_operation;
        pub mod mov_from_stack_operation;
        pub mod mov_operation;
        pub mod operation;
        pub mod operation_aliases;
        pub mod pop_operation;
        pub mod push_constant_operation;
        pub mod push_operation;
        pub mod push_stack_operation;
        pub mod return_operation;
        pub mod stack_alloc_operation;
        pub mod xchg_operation;
    }

    /// Traits implemented by consumers.
    pub mod traits {
        pub mod register_info;
    }

    /// Contains the code rewriter, which is used to rewrite code from one address to another.
    pub mod rewriter {
        pub mod code_rewriter;
    }

    /// Trait for determining length of disassembled instructions
    pub mod length_disassembler;

    pub mod calling_convention_info;
    pub mod function_info;
    pub mod wrapper_generator;
    pub mod wrapper_instruction_generator;
}

pub(crate) mod internal {
    pub(crate) mod assembly_hook;
}

/// Code for all the graph algorithms.
pub(crate) mod graphs {
    pub mod algorithms {
        pub mod move_graph_builder;
        pub mod move_optimizer;
        pub mod move_validator;
    }
    pub mod graph;
    pub mod node;
}

/// Helper functions for the library.
pub mod helpers {
    pub mod alignment_space_finder;
    pub mod allocate_with_proximity;
    pub mod atomic_write;
    pub mod icache_clear;
    pub mod jit_jump_operation;
    pub mod make_inline_rel_branch;
    pub mod overwrite_code;

    /// For Benchmarks and tests only. Do not use in production code.
    #[doc(hidden)]
    pub mod test_helpers;
}

/// Code optimization algorithms.
pub(crate) mod optimize {
    pub mod combine_push_operations;
    pub mod eliminate_common_callee_saved_registers;
    pub mod merge_stackalloc_and_return;
    pub mod optimize_parameters_common;
    pub mod optimize_push_pop_parameters;
    pub mod reorder_mov_sequence;
}
