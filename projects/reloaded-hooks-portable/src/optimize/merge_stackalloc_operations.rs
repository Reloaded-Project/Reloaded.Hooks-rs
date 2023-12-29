extern crate alloc;
use crate::api::jit::operation::Operation;
use crate::api::jit::operation_aliases::StackAlloc;
use alloc::vec::Vec;

/// Merges sequential [`StackAlloc`] operations within a given sequence of
/// [`Operation`]s into a single [`StackAlloc`] operation with the total size.
///
/// This can optimize sequences of multiple small stack allocations into a single
/// allocation, reducing instructions.
///
/// # Parameters
///
/// * `operations` - The vector of [`Operation`]s to optimize. This vector will be
///   mutated in place.
///
/// # Example
///
/// ```compile_fail
/// use reloaded_hooks_portable::api::jit::operation_aliases::StackAlloc
/// use reloaded_hooks_portable::api::jit::operation::Operation;
/// use reloaded_hooks_portable::optimize::merge_stackalloc_operations::combine_stack_alloc_operations;
///
/// let mut operations = vec![
///     Operation::StackAlloc(StackAlloc::new(8)),
///     Operation::StackAlloc(StackAlloc::new(16)),
///     // ...
/// ];
///
/// combine_stack_alloc_operations(&mut operations);
/// ```
pub(crate) fn combine_stack_alloc_operations<TRegister: Copy + Clone>(
    operations: &mut Vec<Operation<TRegister>>,
) {
    let mut read_idx = 0;
    let mut write_idx = 0;

    while read_idx < operations.len() {
        if let Operation::StackAlloc(_) = &operations[read_idx] {
            let mut size = 0;

            // Sum up sizes of sequential StackAlloc ops
            while read_idx < operations.len() {
                if let Operation::StackAlloc(op) = &operations[read_idx] {
                    size += op.operand;
                    read_idx += 1;
                } else {
                    break;
                }
            }

            // Replace with single StackAlloc
            operations[write_idx] = Operation::StackAlloc(StackAlloc::new(size));
            write_idx += 1;
        } else {
            // For other ops, copy as-is
            if read_idx != write_idx {
                operations.swap(read_idx, write_idx);
            }
            read_idx += 1;
            write_idx += 1;
        }
    }

    operations.truncate(write_idx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::jit::operation_aliases::{Pop, StackAlloc},
        helpers::test_helpers::MockRegister,
    };

    #[test]
    fn combine_stack_alloc() {
        let mut operations = vec![
            Operation::StackAlloc(StackAlloc::new(8)),
            Operation::StackAlloc(StackAlloc::new(16)),
            Operation::Pop(Pop::new(MockRegister::R1)),
            Operation::StackAlloc(StackAlloc::new(4)),
            Operation::StackAlloc(StackAlloc::new(8)),
            Operation::Pop(Pop::new(MockRegister::R1)),
        ];

        combine_stack_alloc_operations(&mut operations);

        assert_eq!(operations.len(), 4);
        assert_eq!(operations[0], Operation::StackAlloc(StackAlloc::new(24)));
        assert_eq!(operations[1], Operation::Pop(Pop::new(MockRegister::R1)));
        assert_eq!(operations[2], Operation::StackAlloc(StackAlloc::new(12)));
        assert_eq!(operations[3], Operation::Pop(Pop::new(MockRegister::R1)));
    }
}
