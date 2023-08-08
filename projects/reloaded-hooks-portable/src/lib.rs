//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all declarations that are exposed to library users.
pub mod api {

    /// APIs for allocating buffers in given proximity.
    pub mod buffers {
        pub mod buffer_abstractions;
        pub mod default_buffer;
        pub mod default_buffer_factory;
    }

    pub mod settings {
        pub mod proximity_target;
    }

    /// Platform and architecture specific integrations
    pub mod integration {
        pub mod architecture_details;
        pub mod platform_functions;
    }

    /// Public API related to Just In Time Compilation
    pub mod jit {
        pub mod call_absolute_operation;
        pub mod call_relative_operation;
        pub mod compiler;
        pub mod jump_absolute_operation;
        pub mod jump_relative_operation;
        pub mod mov_operation;
        pub mod operation;
        pub mod pop_operation;
        pub mod push_operation;
        pub mod push_stack_operation;
        pub mod sub_operation;
        pub mod xchg_operation;
    }

    pub mod function_attribute;
    pub mod function_info;
    pub mod wrapper_generator;
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
    pub mod convention_converter_helpers;

    #[cfg(test)]
    pub(crate) mod test_helpers;
}
