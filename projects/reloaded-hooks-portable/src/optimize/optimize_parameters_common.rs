use crate::api::{jit::operation::Operation, traits::register_info::RegisterInfo};

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
#[inline(always)]
pub(crate) fn find_pop_for_given_push<TRegister: RegisterInfo>(
    items: &[Operation<TRegister>],
    stack_offset_to_find: usize,
) -> Option<usize> {
    let mut current_stack_offset = stack_offset_to_find;

    for item in items.iter().enumerate() {
        // Push if at top of stack
        if let Operation::PushStack(x) = item.1 {
            current_stack_offset += x.item_size as usize;
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

#[cfg(test)]
pub mod tests {

    use crate::api::jit::operation_aliases::*;
    use crate::{
        api::{jit::operation::Operation, traits::register_info::RegisterInfo},
        helpers::test_helpers::MockRegister::*,
        optimize::optimize_parameters_common::find_pop_for_given_push,
    };

    #[test]
    fn find_pop_for_given_push_basic() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack::with_offset_and_size(4, 4)),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, Some(2));
    }

    #[test]
    fn find_pop_for_given_push_no_matching_pop() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack::with_offset_and_size(4, 8)),
            Operation::Push(Push { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_missing_just_one_pop() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack::with_offset_and_size(4, 4)),
            Operation::Pop(Pop { register: R1 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_stack_and_register() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            // Size: This register is double the size; so offsets the pops by 1.
            Operation::PushStack(PushStack::with_offset_and_size(4, 8)),
            Operation::Push(Push { register: R3 }),
            Operation::Pop(Pop { register: R4 }),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R3 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, Some(5));
    }
}
