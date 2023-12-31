extern crate alloc;

use core::hash::{BuildHasherDefault, Hash};
use hashbrown::HashSet;
use nohash::NoHashHasher;

use crate::api::jit::operation_aliases::*;

/// Validates an existing sequence of MOV register operations
/// to ensure that a serues if MOV operations from register
/// to register will not write invalid data.
///
/// # Parameters
///
/// - `moves`: The sequence of MOV operations to validate.
///
/// # About
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn validate_moves<T>(moves: &[Mov<T>]) -> bool
where
    T: Eq + PartialEq + Hash + Clone,
{
    // Hashmap but without hashing for keys; we we expect integer (or integer convertible) keys.
    let mut used_targets =
        HashSet::<T, BuildHasherDefault<NoHashHasher<u32>>>::with_capacity_and_hasher(
            moves.len(),
            BuildHasherDefault::default(),
        );

    for move_op in moves {
        if used_targets.contains(&move_op.source) {
            return false;
        }

        used_targets.insert(move_op.target.clone());
    }

    true
}

#[cfg(test)]
pub mod tests {

    use crate::api::jit::operation_aliases::*;
    use crate::graphs::algorithms::move_validator::validate_moves;

    #[test]
    fn valid_when_no_overwrites() {
        let moves = vec![Mov {
            source: 1,
            target: 0,
        }];

        assert!(validate_moves(&moves));
    }

    #[test]
    fn valid_when_last_source_overwritten() {
        let moves = vec![
            Mov {
                source: 1,
                target: 0,
            },
            Mov {
                source: 2,
                target: 1,
            },
        ];

        assert!(validate_moves(&moves));
    }

    #[test]
    fn valid_when_last_source_overwritten_with_multiple_registers() {
        let moves = vec![
            Mov {
                source: 1,
                target: 0,
            },
            Mov {
                source: 2,
                target: 1,
            },
            Mov {
                source: 3,
                target: 2,
            },
        ];

        assert!(validate_moves(&moves));
    }

    #[test]
    fn invalid_when_source_register_already_overwritten() {
        let moves = vec![
            Mov {
                source: 1,
                target: 0,
            },
            Mov {
                source: 0,
                target: 2,
            },
        ];

        assert!(!validate_moves(&moves));
    }
}
