extern crate alloc;
use alloc::vec::Vec;
use core::hash::{BuildHasherDefault, Hash};
use hashbrown::HashSet;
use nohash::NoHashHasher;

type NoHashHashSet<T> = HashSet<T, BuildHasherDefault<NoHashHasher<u32>>>;

/// Eliminates common elements in the two slices.
pub fn eliminate_common_callee_saved_registers<TRegister: Hash + Eq + Clone>(
    a: &[TRegister],
    b: &[TRegister],
) -> Vec<TRegister> {
    let mut result = Vec::new();
    if a.len() > 10 || b.len() > 10 {
        let a_set: NoHashHashSet<_> = a.iter().cloned().collect();
        let b_set: NoHashHashSet<_> = b.iter().cloned().collect();

        let mut result = Vec::new();

        for item in a {
            if !b_set.contains(item) {
                result.push(item.clone());
            }
        }

        for item in b {
            if !a_set.contains(item) {
                result.push(item.clone());
            }
        }

        result
    } else {
        for item in a {
            if !b.contains(item) {
                result.push(item.clone());
            }
        }

        for item in b {
            if !a.contains(item) {
                result.push(item.clone());
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::test_helpers::MockRegister::{self, *};

    use super::*;

    #[test]
    fn test_no_common_elements() {
        let a = [R1, R2, R3];
        let b = [F1, F2, F3];
        let result = eliminate_common_callee_saved_registers(&a, &b);
        assert_eq!(result, vec![R1, R2, R3, F1, F2, F3]);
    }

    #[test]
    fn test_some_common_elements() {
        let a = [R1, R2, R3];
        let b = [R2, R3, F1];
        let result = eliminate_common_callee_saved_registers(&a, &b);
        assert_eq!(result, vec![R1, F1]);
    }

    #[test]
    fn test_all_common_elements() {
        let a = [R1, R2, R3];
        let b = [R1, R2, R3];
        let result = eliminate_common_callee_saved_registers(&a, &b);
        assert_eq!(result, Vec::<MockRegister>::new());
    }

    #[test]
    fn test_large_no_common_elements() {
        let a = [R1, R2, R3, R4, F1, F2, F3, F4, SP, R1, LR];
        let b = [R1, R2, R3, R4, F1, F2, F3, F4, SP, F1, LR];
        let result = eliminate_common_callee_saved_registers(&a, &b);
        assert_eq!(result, Vec::<MockRegister>::new());
    }
}
