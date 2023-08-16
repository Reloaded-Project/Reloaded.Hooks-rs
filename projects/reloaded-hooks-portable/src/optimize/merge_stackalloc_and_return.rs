use crate::api::jit::operation_aliases::{Op, Return};

/// Optimizes the code to merge [StackAlloc](crate::api::jit::stack_alloc_Op::StackAllocOperation) and
/// [Return](crate::api::jit::return_Op::ReturnOperation) operations into a single operation,
/// for supported architectures.
///
/// More specifically, optimizes the following sequence:
///
/// ```asm
/// add esp, 12 # Restore 12 bytes of stack
/// ret 0 # Return
/// ```
///
/// # Params
///
/// - `operations`: All operations emitted during wrapper generation up to method call. i.e. Starting with
///                 push and ending on pop.
///
/// # Returns
///
/// A new list of operations, these operations should replace the input slice that was passed to this structure.
pub fn merge_stackalloc_and_return<TRegister: Clone>(
    operations: &mut [Op<TRegister>],
) -> &mut [Op<TRegister>] {
    if operations.len() < 2 {
        return operations;
    }

    // Check if the last two operations are StackAlloc and Return
    if let (Op::StackAlloc(stack_alloc), Op::Return(ret)) = (
        &operations[operations.len() - 2],
        &operations[operations.len() - 1],
    ) {
        // Merge the operations
        let new_offset = ret.offset as i32 + -stack_alloc.operand;
        if new_offset >= 0 {
            operations[operations.len() - 2] = Op::Return(Return::new(new_offset as usize));
            let slice_max = operations.len() - 1;
            return &mut operations[..slice_max];
        }
    }

    operations
}

#[cfg(test)]
mod tests {
    use crate::{
        api::jit::operation_aliases::{Op, StackAlloc},
        helpers::test_helpers::MockRegister,
    };

    use super::*;

    #[test]
    fn test_merge_stackalloc_and_return() {
        let mut operations: Vec<Op<MockRegister>> = vec![
            Op::StackAlloc(StackAlloc { operand: -4 }),
            Op::Return(Return::new(0)),
        ];

        let optimized = merge_stackalloc_and_return(&mut operations);

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0], Op::Return(Return::new(4)));
    }

    #[test]
    fn test_no_merge_with_insufficient_operations() {
        let mut operations: Vec<Op<MockRegister>> = vec![Op::Return(Return::new(0))];
        let optimized = merge_stackalloc_and_return(&mut operations);

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0], Op::Return(Return::new(0)));
    }

    #[test]
    fn test_no_merge_with_non_matching_operations() {
        let mut operations: Vec<Op<MockRegister>> =
            vec![Op::Return(Return::new(0)), Op::Return(Return::new(0))];

        let optimized = merge_stackalloc_and_return(&mut operations);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], Op::Return(Return::new(0)));
        assert_eq!(optimized[1], Op::Return(Return::new(0)));
    }

    #[test]
    fn test_no_merge_with_negative_resulting_offset() {
        let mut operations: Vec<Op<MockRegister>> = vec![
            Op::StackAlloc(StackAlloc { operand: 4 }),
            Op::Return(Return::new(0)),
        ];

        let optimized = merge_stackalloc_and_return(&mut operations);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], Op::StackAlloc(StackAlloc { operand: 4 }));
        assert_eq!(optimized[1], Op::Return(Return::new(0)));
    }
}
