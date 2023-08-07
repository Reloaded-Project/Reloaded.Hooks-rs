//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all declarations that are exposed to library users.
pub mod api {
    pub mod buffers {
        pub mod buffer_abstractions;
        pub mod default_buffer;
        pub mod default_buffer_factory;
    }

    pub mod settings {
        pub mod proximity_target;
    }

    pub mod function_attribute;
    pub mod function_info;
    pub mod init;
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

/// Contains all of the structures used by the library.
pub mod structs {
    pub mod mov_operation;
    pub mod operation;
    pub mod pop_operation;
    pub mod push_operation;
    pub mod push_stack_operation;
    pub mod sub_operation;
    pub mod xchg_operation;
}
