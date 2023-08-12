extern crate alloc;

use super::optimize_parameters_common::{find_pop_for_given_push, replace_optimized_operation};
use crate::api::jit::operation_aliases::*;
use crate::api::{jit::operation::Operation, traits::register_info::RegisterInfo};

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
/// - `operations`: All operations emitted during wrapper generation up to method call. i.e. Starting with
///                 push and ending on pop.
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
    push_stack: &PushStack<TRegister>,
    pop: &Pop<TRegister>,
) -> Option<MovFromStack<TRegister>> {
    if !push_stack.base_register.is_stack_pointer() {
        return None;
    }

    Some(MovFromStack {
        stack_offset: push_stack.offset as i32,
        target: pop.register.clone(),
    })
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
    use crate::api::jit::operation_aliases::*;
    use crate::helpers::test_helpers::MockRegister::*;

    use super::*;

    #[test]
    fn update_stack_push_offsets_no_change() {
        let mut ops = vec![
            Operation::PushStack(PushStack {
                base_register: R1,
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                base_register: R1,
                offset: 0,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_with_sp() {
        let mut ops = vec![
            Operation::PushStack(PushStack {
                base_register: SP,
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                base_register: SP,
                offset: 10,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_negative_adjustment() {
        let mut ops = vec![
            Operation::PushStack(PushStack {
                base_register: SP,
                offset: 10,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, -5);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                base_register: SP,
                offset: 5,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                base_register: R2,
                offset: 10,
                item_size: 4,
            })
        );
    }

    #[test]
    fn optimize_basic() {
        let mut operations = vec![
            Operation::PushStack(PushStack {
                item_size: 4,
                base_register: SP,
                offset: 4,
            }),
            Operation::PushStack(PushStack {
                item_size: 4,
                base_register: SP,
                offset: 12,
            }),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
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
        let mut operations = vec![Operation::PushStack(PushStack {
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
            Operation::PushStack(PushStack {
                item_size: 4,
                base_register: SP,
                offset: 4,
            }),
            // Some other operation that should prevent optimization
            Operation::Push(Push { register: R1 }),
            Operation::Pop(Pop {
                register: R1,
                // ... other fields as needed
            }),
        ];

        let optimized = optimize_stack_parameters(&mut operations);
        // The optimization should not occur because of the Push operation in between
        assert_eq!(optimized.len(), 3);
    }
}
