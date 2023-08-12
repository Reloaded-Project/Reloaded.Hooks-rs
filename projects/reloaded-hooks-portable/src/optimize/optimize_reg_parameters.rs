use super::optimize_parameters_common::{find_pop_for_given_push, replace_optimized_operation};
use crate::api::jit::operation_aliases::*;
use crate::api::{jit::operation::Operation, traits::register_info::RegisterInfo};

/// Optimizes the parameters that are pushed from register and then popped back
/// into another register.
///
/// More specifically, optimizes the following sequence:
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
/// # Params
///
/// - `operations`: All operations emitted during wrapper generation up to method call. i.e. Starting with
///                 push and ending on pop.
///
/// # Returns
///
/// A new list of operations, these operations should replace the input slice that was passed to this structure.
pub fn optimize_push_pop_parameters<TRegister: RegisterInfo + Copy>(
    operations: &mut [Operation<TRegister>],
) -> &mut [Operation<TRegister>] {
    let mut current_stack_offset = 0;

    // Note: The current implementation is very slow, and is effectively O(N^3)
    // However the input size is always small (for example it might be 20 operations if a function has 10 parameters)
    for push_idx in 0..operations.len() {
        let operation = &operations[push_idx];
        if let Operation::PushStack(x) = operation {
            current_stack_offset += x.item_size;
        } else if let Operation::Push(x) = operation {
            current_stack_offset += x.register.size_in_bytes();

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

            let opt_optimized_operation = encode_push_pop_to_mov(x, pop_op);
            if opt_optimized_operation.is_none() {
                continue;
            }

            // Time to replace the optimized operation 😉
            let opt_optimized_operation = opt_optimized_operation.unwrap();
            let new_slice = replace_optimized_operation(
                operations,
                push_idx,
                pop_idx,
                &Operation::Mov(opt_optimized_operation),
            );

            return optimize_push_pop_parameters(new_slice);
        }
    }

    operations
}

/// Accepts a push stack operation and a pop operation, and returns a mov operation that
/// is equivalent to both the operations.
fn encode_push_pop_to_mov<TRegister: Clone + RegisterInfo>(
    push: &Push<TRegister>,
    pop: &Pop<TRegister>,
) -> Option<Mov<TRegister>> {
    // This encode is only possible if both registers have the same 'type' according to JIT.
    if pop.register.register_type() != push.register.register_type() {
        return None;
    }

    Some(Mov {
        source: push.register.clone(),
        target: pop.register.clone(),
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::helpers::test_helpers::MockRegister::*;

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
}
