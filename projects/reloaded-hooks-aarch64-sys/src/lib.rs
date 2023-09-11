//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(test), no_std)]

#[cfg(not(tarpaulin_include))]
pub mod all_registers;

/// Utility for writing aarch64 code.
pub mod code_writer;

pub mod jit;
