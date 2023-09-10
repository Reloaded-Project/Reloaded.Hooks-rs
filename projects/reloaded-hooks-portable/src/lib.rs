//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all declarations that are exposed to library users.
pub mod api {

    /// The errors that can occur when generating a wrapper.
    pub mod errors {
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
        pub mod proximity_target;
    }

    /// Platform and architecture specific integrations
    /// Note: Depends on STD crate, but implementation in crate is no-std.
    pub mod platforms {
        pub mod platform_functions;
        mod platform_functions_mmap_rs;
    }

    /// Public API related to Just In Time Compilation
    pub mod jit {
        pub mod call_absolute_operation;
        pub mod call_relative_operation;
        pub mod call_rip_relative_operation;
        pub mod compiler;
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

    pub mod function_attribute;
    pub mod function_info;
    pub mod wrapper_generator;
    pub mod wrapper_instruction_generator;
}

/// Code for all the graph algorithms.
pub mod graphs {
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

    // Benchmark and test only.
    #[doc(hidden)]
    pub mod test_helpers;
}

/// Code optimization algorithms.
pub mod optimize {
    pub mod combine_push_operations;
    pub mod eliminate_common_callee_saved_registers;
    pub mod merge_stackalloc_and_return;
    pub mod optimize_parameters_common;
    pub mod optimize_push_pop_parameters;
    pub mod reorder_mov_sequence;
}
