use super::optimize_parameters_common::find_pop_for_given_push;
use crate::api::jit::operation_aliases::*;
use crate::api::{jit::operation::Operation, traits::register_info::RegisterInfo};

/// Optimizes stack parameters that are:
///
/// - Pushed from register and then popped back into another register.
/// - Pushed from stack offset and then popped back into another register.
///
/// # Register to Register
///
/// Optimizes the following sequence:
///
/// ```asm
/// # Push register parameters of the function being returned (right to left, reverse loop)
/// push rdx
/// push rcx
///
/// # Pop parameters into registers of function being called
/// pop rdi
/// pop rsi
/// ```
///
/// ----
///
/// Into the following sequence:
///
/// ```asm
/// mov rdi, rcx # last push, first pop
/// mov rsi, rdx # second last push, second pop
/// ```
///
/// # Stack to Register
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
/// # Remarks
///
/// Input should only contain push-pop operations, not StackAlloc or anything of the kind.
///
/// # Returns
///
/// A new list of operations, these operations should replace the input slice that was passed to this structure.
pub fn optimize_push_pop_parameters<TRegister: RegisterInfo + Copy>(
    operations: &mut [Operation<TRegister>],
) -> &mut [Operation<TRegister>] {
    let mut current_stack_offset = 0;

    // Note: The current implementation is very slow, and is effectively O(N^2)
    // Anyway, for each PushStack or Pop operation, find the matching Pop or PushStack operation.

    let mut push_idx = 0;
    while push_idx < operations.len() {
        let operation = &operations[push_idx];
        match operation {
            Operation::PushStack(x) => {
                let item_size = x.item_size;
                current_stack_offset += item_size;

                // Found a push, now find the next pop.
                let pop =
                    find_pop_for_given_push(&operations[push_idx + 1..], current_stack_offset);
                if pop.is_none() {
                    push_idx += 1;
                    continue;
                }

                // We found a 'pop' for this push operation, try to encode optimized function.
                let pop_idx = unsafe { pop.unwrap_unchecked() } + push_idx + 1;
                let pop_op = match &operations[pop_idx] {
                    Operation::Pop(x) => x,
                    _ => unreachable!(),
                };
                let opt_optimized_operation = encode_push_stack_to_mov(x, pop_op);
                if opt_optimized_operation.is_none() {
                    push_idx += 1;
                    continue;
                }

                // Replace the optimized operation and insert nop.
                current_stack_offset -= item_size;
                unsafe {
                    *operations.get_unchecked_mut(push_idx) =
                        opt_optimized_operation.unwrap_unchecked().into();
                    *operations.get_unchecked_mut(pop_idx) = Operation::None;
                };

                update_stack_push_offsets(&mut operations[push_idx..], -(item_size as i32));
                push_idx += 1;
            }
            Operation::Push(x) => {
                let item_size = x.register.size_in_bytes();
                current_stack_offset += item_size;

                // Found a push, now find the next pop.
                let pop =
                    find_pop_for_given_push(&operations[push_idx + 1..], current_stack_offset);
                if pop.is_none() {
                    push_idx += 1;
                    continue;
                }

                // We found a 'pop' for this push operation, try to encode optimized function.
                let pop_idx = unsafe { pop.unwrap_unchecked() } + push_idx + 1;
                let pop_op = match &operations[pop_idx] {
                    Operation::Pop(x) => x,
                    _ => unreachable!(),
                };

                let opt_optimized_operation = encode_push_pop_to_mov(x, pop_op);
                if opt_optimized_operation.is_none() {
                    push_idx += 1;
                    continue;
                }

                // Replace the optimized operation and insert nop.
                current_stack_offset -= item_size;
                unsafe {
                    *operations.get_unchecked_mut(push_idx) =
                        opt_optimized_operation.unwrap_unchecked().into();
                    *operations.get_unchecked_mut(pop_idx) = Operation::None;
                };
                push_idx += 1;
            }
            _ => {
                push_idx += 1;
            }
        }
    }

    // Now scan through any of the 'nop' operations, and remove them.
    remove_nones(operations)
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
pub(crate) fn update_stack_push_offsets<TRegister: RegisterInfo>(
    items: &mut [Operation<TRegister>],
    offset_to_adjust_by: i32,
) {
    for item in items {
        if let Operation::PushStack(x) = item {
            x.offset += offset_to_adjust_by as isize;
        }
    }
}

/// Accepts a push stack operation and a pop operation, and returns a mov operation that
/// is equivalent to both the operations.
fn encode_push_pop_to_mov<TRegister: Copy + RegisterInfo>(
    push: &Push<TRegister>,
    pop: &Pop<TRegister>,
) -> Option<Mov<TRegister>> {
    // This encode is only possible if both registers have the same 'type' according to JIT.
    if pop.register.register_type() != push.register.register_type() {
        return None;
    }

    Some(Mov {
        source: push.register,
        target: pop.register,
    })
}

/// Accepts a push stack operation and a pop operation, and returns a mov operation that
/// is equivalent to both the operations.
fn encode_push_stack_to_mov<TRegister: Copy + RegisterInfo>(
    push_stack: &PushStack,
    pop: &Pop<TRegister>,
) -> Option<MovFromStack<TRegister>> {
    Some(MovFromStack {
        stack_offset: push_stack.offset as i32,
        target: pop.register,
    })
}

fn remove_nones<TRegister: Copy>(
    operations: &mut [Operation<TRegister>],
) -> &mut [Operation<TRegister>] {
    let mut write_idx = 0;

    for read_idx in 0..operations.len() {
        match operations[read_idx] {
            Operation::None => {}
            _ => {
                operations[write_idx] = operations[read_idx].clone();
                write_idx += 1;
            }
        }
    }

    &mut operations[0..write_idx]
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::helpers::test_helpers::MockRegister::{self, *};

    #[test]
    fn optimizes_push_pop_sequence() {
        let mut operations = vec![
            Operation::Push(Push { register: R2 }),
            Operation::Push(Push { register: R3 }),
            Operation::Pop(Pop { register: R4 }),
            Operation::Pop(Pop { register: R1 }),
        ];

        let new_ops = optimize_push_pop_parameters(&mut operations);

        assert_eq!(
            new_ops,
            vec![
                Operation::Mov(Mov {
                    source: R2,
                    target: R1,
                }),
                Operation::Mov(Mov {
                    source: R3,
                    target: R4,
                }),
            ]
        );
    }

    #[test]
    fn mixed_sequence_with_missing_pop() {
        let mut operations = vec![
            Operation::Push(Push { register: F1 }),
            Operation::Push(Push { register: F2 }),
            Operation::Pop(Pop { register: F3 }),
        ];

        let new_ops = optimize_push_pop_parameters(&mut operations);

        assert_eq!(
            new_ops,
            vec![
                Operation::Push(Push { register: F1 }),
                Operation::Mov(Mov {
                    source: F2,
                    target: F3,
                }),
            ]
        );
    }

    #[test]
    fn multiple_consecutive_push_pop_sequences_optimized() {
        let mut operations = vec![
            Operation::Push(Push { register: R1 }),
            Operation::Push(Push { register: R2 }),
            Operation::Pop(Pop { register: R3 }),
            Operation::Pop(Pop { register: R4 }),
            Operation::Push(Push { register: R3 }),
            Operation::Push(Push { register: R4 }),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        let new_ops = optimize_push_pop_parameters(&mut operations);

        assert_eq!(
            new_ops,
            vec![
                Operation::Mov(Mov {
                    source: R1,
                    target: R4,
                }),
                Operation::Mov(Mov {
                    source: R2,
                    target: R3,
                }),
                Operation::Mov(Mov {
                    source: R3,
                    target: R2,
                }),
                Operation::Mov(Mov {
                    source: R4,
                    target: R1,
                }),
            ]
        );
    }

    #[test]
    fn update_stack_push_offsets_no_change() {
        let mut ops: Vec<Operation<MockRegister>> = vec![
            Operation::PushStack(PushStack {
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                offset: 20,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_with_sp() {
        let mut ops: Vec<Operation<MockRegister>> = vec![
            Operation::PushStack(PushStack {
                offset: 0,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, 10);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                offset: 20,
                item_size: 4,
            })
        );
    }

    #[test]
    fn update_stack_push_offsets_negative_adjustment() {
        let mut ops: Vec<Operation<MockRegister>> = vec![
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            }),
            Operation::PushStack(PushStack {
                offset: 10,
                item_size: 4,
            }),
        ];

        update_stack_push_offsets(&mut ops, -5);

        assert_eq!(
            ops[0],
            Operation::PushStack(PushStack {
                offset: 5,
                item_size: 4,
            })
        );
        assert_eq!(
            ops[1],
            Operation::PushStack(PushStack {
                offset: 5,
                item_size: 4,
            })
        );
    }

    #[test]
    fn optimize_push_stack_basic() {
        let mut operations = vec![
            Operation::PushStack(PushStack {
                item_size: 4,
                offset: 4,
            }),
            Operation::PushStack(PushStack {
                item_size: 4,
                offset: 12,
            }),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        let optimized = optimize_push_pop_parameters(&mut operations);
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
    fn optimize_push_stack_no_matching_pop() {
        let mut operations = vec![Operation::PushStack(PushStack {
            item_size: 4,
            offset: 4,
        })];

        let optimized: &mut [Operation<MockRegister>] =
            optimize_push_pop_parameters(&mut operations);
        assert_eq!(optimized.len(), 1);
        match &optimized[0] {
            Operation::PushStack(_) => {}
            _ => panic!("Expected a PushStack operation."),
        }
    }
}
