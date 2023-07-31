extern crate alloc;
use alloc::vec::Vec;

use crate::common::move_operation::MoveOperation;

/// Validates an existing sequence of MOV register operations
/// to ensure that a serues if MOV operations from register
/// to register will not write invalid data.
///
/// # About
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn validate_moves<T>(moves: Vec<MoveOperation<T>>) {}

/// Reorders a sequence of MOV register operations to prevent
/// them from writing invalid data.
///
/// # About
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn reorder_moves<T>(moves: Vec<MoveOperation<T>>) {}
