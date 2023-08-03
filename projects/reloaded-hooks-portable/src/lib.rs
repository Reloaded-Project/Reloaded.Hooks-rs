//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

/// Contains all declarations that are exposed to library users.
pub mod api {
    pub mod function_attribute;
}

/// Code for all the graph algorithms.
pub mod graph {
    pub mod graph;
    pub mod node;
}

/// Code for all the graph algorithms.
pub mod structs {
    pub mod move_operation;
}
