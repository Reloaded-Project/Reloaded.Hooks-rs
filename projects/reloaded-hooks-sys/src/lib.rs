//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

pub mod common {
    pub mod move_operation;

    mod graph {
        mod graph;
        mod move_optimiser;
        mod node;
    }
}

pub mod x64 {
    pub mod function_attribute;
    pub mod preset_calling_convention;
    pub mod register;
}
