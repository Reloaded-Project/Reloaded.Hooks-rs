extern crate alloc;

use crate::api::{
    jit::{
        mov_from_stack_operation::MovFromStackOperation, operation::Operation,
        pop_operation::PopOperation, push_stack_operation::PushStackOperation,
    },
    traits::register_size::RegisterInfo,
};
use alloc::vec::Vec;

/// Optimizes the parameters that are passed via the stack.
///
/// More specifically, optimizes the following sequence:
///
/// ```asm
/// # Re-push STDCALL arguments to stack
/// push dword [esp + {x}]
/// push dword [esp + {x}]
///
/// # Pop into correct registers
/// pop eax
/// pop ecx
/// ```
///
/// ----
///
/// Into the following sequence:
///
/// ```asm
/// # Move directly to registers
/// mov eax, [esp + {x}]
/// mov ecx, [esp + {x}]
/// ```
///
/// # Params
///
/// - `moves`: All operations emitted during wrapper generation up to method call. i.e. Starting with
///            push and ending on pop.
///
/// # Returns
///
/// A new list of operations, these operations should replace the input slice that was passed to this structure.
///
/// # Remarks
///
/// This algorithm is not supposed to handle the sequence:
/// - push
/// - pop
/// - push
///
/// Although right now, such sequences are handled, results of this may not be guaranteed to be
/// correct if such a sequence is passed in the future.
pub fn optimize_stack_parameters<TRegister: RegisterInfo + Copy>(
    operations: &mut [Operation<TRegister>],
) -> &mut [Operation<TRegister>] {
    let mut current_stack_offset = 0;

    // Note: The current implementation is very slow, and is effectively O(N^3)
    // However the input size is always small (for example it might be 20 operations if a function has 10 parameters)
    for push_idx in 0..operations.len() {
        let operation = &operations[push_idx];
        if let Operation::PushStack(x) = operation {
            current_stack_offset += x.item_size;

            // Found a push, now find the next pop.
            let pop = find_pop_for_given_push(&operations[push_idx + 1..], current_stack_offset);
            if pop.is_none() {
                continue;
            }

            // We found a 'pop' for this push operation, try to encode optimized function.
            let pop_idx = pop.unwrap() + push_idx + 1;
            let pop_op = match &operations[pop_idx] {
                Operation::Pop(x) => x,
                _ => unreachable!(),
            };
            let opt_optimized_operation = encode_push_stack_to_mov(x, pop_op);
            if opt_optimized_operation.is_none() {
                continue;
            }

            // Time to replace the optimized operation 😉
            let opt_optimized_operation = opt_optimized_operation.unwrap();
            let new_slice = replace_optimized_operation(
                operations,
                push_idx,
                pop_idx,
                &Operation::MovFromStack(opt_optimized_operation),
            );

            update_stack_push_offsets(new_slice, -opt_optimized_operation.stack_offset);
            return optimize_stack_parameters(new_slice);
        } else if let Operation::Push(x) = operation {
            current_stack_offset += x.register.size_in_bytes();
        }
    }

    operations
}

/// Accepts a push stack operation and a pop operation, and returns a mov operation that
/// is equivalent to both the operations.
fn encode_push_stack_to_mov<TRegister: Clone + RegisterInfo>(
    push_stack: &PushStackOperation<TRegister>,
    pop: &PopOperation<TRegister>,
) -> Option<MovFromStackOperation<TRegister>> {
    if !push_stack.base_register.is_stack_pointer() {
        return None;
    }

    Some(MovFromStackOperation {
        stack_offset: push_stack.offset as i32,
        target: pop.register.clone(),
    })
}

/// Replaces an existing pair of push+pop operations with an optimized single operation.
/// The entries at push_index and pop_index are removed from the slice, and the optimized
/// operation is now inserted in place of where push_index originally was.
///
/// # Parameters
///
/// - `operations` - The slice of operations to modify.
/// - `push_index` - The index of the push operation to remove.
/// - `pop_index` - The index of the pop operation to remove.
/// - `new_operation` - The new operation to replace the push+pop with.
///                     This replaces item at push_index
fn replace_optimized_operation<'a, TRegister: Copy>(
    operations: &'a mut [Operation<TRegister>],
    push_index: usize,
    pop_index: usize,
    new_operation: &Operation<TRegister>,
) -> &'a mut [Operation<TRegister>] {
    // Replace the push operation with the new optimized operation.
    operations[push_index] = *new_operation;

    // Copy the items after pop_index backwards by one position to remove the pop operation.
    operations.copy_within(pop_index + 1.., pop_index);

    // Return a slice that excludes the last (removed) element
    let num_items = operations.len();
    &mut operations[..num_items - 1]
}

/// Finds a `pop` operation which corresponds to the current `push` operation.
/// This is done by waiting until a 'pop' instruction is found whereby the stack
/// offset before the pop instruction is equal to the given `stack_offset_to_find`
/// (offset after 'push' instruction.)
///
/// # Parameters
///
/// - `items` - The slice of operations to search.
///             This slice should start on the item after the push for which we are finding the pop.
///
/// - `stack_offset_to_find` - The stack offset to find a pop for.
///
/// # Returns
///
/// Index of the pop operation if found, otherwise None.
fn find_pop_for_given_push<TRegister: RegisterInfo>(
    items: &[Operation<TRegister>],
    stack_offset_to_find: usize,
) -> Option<usize> {
    let mut current_stack_offset = stack_offset_to_find;

    for item in items.iter().enumerate() {
        // Push if at top of stack
        if let Operation::PushStack(x) = item.1 {
            current_stack_offset += x.item_size;
        } else if let Operation::Push(x) = item.1 {
            current_stack_offset += x.register.size_in_bytes();
        }

        // Pop if at bottom of the stack
        if let Operation::Pop(x) = item.1 {
            // Check is current item's stack matches.
            if current_stack_offset == stack_offset_to_find {
                return Some(item.0);
            }

            current_stack_offset -= x.register.size_in_bytes();
        }
    }

    None
}

/// Updates the stack offsets of all push (PushStack) operations in the given slice.
///
/// # Parameters
///
/// - `items` - The slice of operations to update.
/// - `offset_to_adjust_by` - The offset to adjust the stack pointer by.
///
/// # Remarks
///
/// We call this after replacing a stack pointer relative push with a mov, as future
/// operations need to be updated.
fn update_stack_push_offsets<TRegister: RegisterInfo>(
    items: &mut [Operation<TRegister>],
    offset_to_adjust_by: i32,
) {
    for item in items {
        if let Operation::PushStack(x) = item {
            if x.base_register.is_stack_pointer() {
                x.offset += offset_to_adjust_by as isize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::jit::{
            pop_operation::PopOperation, push_operation::PushOperation,
            push_stack_operation::PushStackOperation,
        },
        helpers::test_helpers::MockRegister::*,
    };

    use super::*;

    #[test]
    fn update_stack_push_offsets_no_change() {
        let mut ops = vec![
            Operation::PushStack(PushStackOperation {
                base_register: R1,
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStackOperation {
                base_register: R1,
                offset: 0,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_with_sp() {
        let mut ops = vec![
            Operation::PushStack(PushStackOperation {
                base_register: SP,
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStackOperation {
                base_register: SP,
                offset: 10,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_negative_adjustment() {
        let mut ops = vec![
            Operation::PushStack(PushStackOperation {
                base_register: SP,
                offset: 10,
                item_size: 4,
            }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, -5);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStackOperation {
                base_register: SP,
                offset: 5,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn find_pop_for_given_push_basic() {
        let ops = vec![
            Operation::Push(PushOperation { register: R1 }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                item_size: 4,
                offset: 4,
            }),
            Operation::Pop(PopOperation { register: R1 }),
            Operation::Pop(PopOperation { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, Some(2));
    }

    #[test]
    fn find_pop_for_given_push_no_matching_pop() {
        let ops = vec![
            Operation::Push(PushOperation { register: R1 }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                item_size: 8,
                offset: 4,
            }),
            Operation::Push(PushOperation { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_missing_just_one_pop() {
        let ops = vec![
            Operation::Push(PushOperation { register: R1 }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                item_size: 4,
                offset: 4,
            }),
            Operation::Pop(PopOperation { register: R1 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_stack_and_register() {
        let ops = vec![
            Operation::Push(PushOperation { register: R1 }),
            Operation::PushStack(PushStackOperation {
                base_register: R2,
                item_size: 8, // This register is double the size; so offsets the pops by 1.
                offset: 4,
            }),
            Operation::Push(PushOperation { register: R3 }),
            Operation::Pop(PopOperation { register: R4 }),
            Operation::Pop(PopOperation { register: R1 }),
            Operation::Pop(PopOperation { register: R3 }),
            Operation::Pop(PopOperation { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, Some(5));
    }

    #[test]
    fn can_replace_optimized_operation() {
        // Sample operations list
        let mut operations = [
            Operation::PushStack(PushStackOperation {
                base_register: SP,
                offset: 4,
                item_size: 4,
            }),
            Operation::Push(PushOperation { register: R1 }),
            Operation::Pop(PopOperation { register: R2 }),
            Operation::Pop(PopOperation { register: R3 }),
        ];

        let mov_op = Operation::MovFromStack(MovFromStackOperation {
            stack_offset: 4,
            target: R3,
        });

        let result = replace_optimized_operation(&mut operations, 0, 3, &mov_op);

        // Expected result
        let expected = [
            Operation::MovFromStack(MovFromStackOperation {
                stack_offset: 4,
                target: R3,
            }),
            Operation::Push(PushOperation { register: R1 }),
            Operation::Pop(PopOperation { register: R2 }),
        ];

        assert_eq!(result, &expected);
    }

    #[test]
    fn optimize_basic() {
        let mut operations = vec![
            Operation::PushStack(PushStackOperation {
                item_size: 4,
                base_register: SP,
                offset: 4,
            }),
            Operation::PushStack(PushStackOperation {
                item_size: 4,
                base_register: SP,
                offset: 12,
            }),
            Operation::Pop(PopOperation { register: R1 }),
            Operation::Pop(PopOperation { register: R2 }),
        ];

        let optimized = optimize_stack_parameters(&mut operations);
        assert_eq!(optimized.len(), 2);
        match &optimized[0] {
            Operation::MovFromStack(x) => {
                assert!(x.stack_offset == 4)
            }
            _ => panic!("Expected a MovFromStack operation."),
        }
        match &optimized[1] {
            Operation::MovFromStack(x) => {
                assert!(x.stack_offset == 8)
            }
            _ => panic!("Expected a MovFromStack operation."),
        }
    }

    #[test]
    fn optimize_no_matching_pop() {
        let mut operations = vec![Operation::PushStack(PushStackOperation {
            item_size: 4,
            base_register: SP,
            offset: 4,
        })];

        let optimized = optimize_stack_parameters(&mut operations);
        assert_eq!(optimized.len(), 1);
        match &optimized[0] {
            Operation::PushStack(_) => {}
            _ => panic!("Expected a PushStack operation."),
        }
    }

    #[test]
    fn optimize_operation_between_push_pop() {
        let mut operations = vec![
            Operation::PushStack(PushStackOperation {
                item_size: 4,
                base_register: SP,
                offset: 4,
            }),
            // Some other operation that should prevent optimization
            Operation::Push(PushOperation { register: R1 }),
            Operation::Pop(PopOperation {
                register: R1,
                // ... other fields as needed
            }),
        ];

        let optimized = optimize_stack_parameters(&mut operations);
        // The optimization should not occur because of the Push operation in between
        assert_eq!(optimized.len(), 3);
    }
}
