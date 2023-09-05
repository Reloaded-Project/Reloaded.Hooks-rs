use crate::api::{jit::operation::Operation, traits::register_info::RegisterInfo};

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
#[inline(always)]
pub(crate) fn replace_optimized_operation<'a, TRegister: Copy>(
    operations: &'a mut [Operation<TRegister>],
    push_index: usize,
    pop_index: usize,
    new_operation: &Operation<TRegister>,
) -> &'a mut [Operation<TRegister>] {
    unsafe {
        // Replace the push operation with the new optimized operation.
        *operations.get_unchecked_mut(push_index) = new_operation.clone();

        // Manually shift the elements to the left starting from pop_index
        for x in pop_index..operations.len() - 1 {
            *operations.get_unchecked_mut(x) = operations.get_unchecked(x + 1).clone();
        }

        // Return a slice that excludes the last (removed) element
        let num_items = operations.len();

        operations.get_unchecked_mut(..num_items - 1)
    }
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
pub(crate) fn find_pop_for_given_push<TRegister: RegisterInfo>(
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

#[cfg(test)]
pub mod tests {

    use crate::api::jit::operation_aliases::*;
    use crate::{
        api::{jit::operation::Operation, traits::register_info::RegisterInfo},
        helpers::test_helpers::MockRegister::*,
        optimize::optimize_parameters_common::{
            find_pop_for_given_push, replace_optimized_operation,
        },
    };

    #[test]
    fn find_pop_for_given_push_basic() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack {
                item_size: 4,
                offset: 4,
            }),
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
            Operation::PushStack(PushStack {
                item_size: 8,
                offset: 4,
            }),
            Operation::Push(Push { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_missing_just_one_pop() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack {
                item_size: 4,
                offset: 4,
            }),
            Operation::Pop(Pop { register: R1 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, None);
    }

    #[test]
    fn find_pop_for_given_push_stack_and_register() {
        let ops = vec![
            Operation::Push(Push { register: R1 }),
            Operation::PushStack(PushStack {
                item_size: 8, // This register is double the size; so offsets the pops by 1.
                offset: 4,
            }),
            Operation::Push(Push { register: R3 }),
            Operation::Pop(Pop { register: R4 }),
            Operation::Pop(Pop { register: R1 }),
            Operation::Pop(Pop { register: R3 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        let idx = find_pop_for_given_push(&ops[1..], R1.size_in_bytes());

        assert_eq!(idx, Some(5));
    }

    #[test]
    fn can_replace_optimized_operation() {
        // Sample operations list
        let mut operations = [
            Operation::PushStack(PushStack {
                offset: 4,
                item_size: 4,
            }),
            Operation::Push(Push { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
            Operation::Pop(Pop { register: R3 }),
        ];

        let mov_op = Operation::MovFromStack(MovFromStack {
            stack_offset: 4,
            target: R3,
        });

        let result = replace_optimized_operation(&mut operations, 0, 3, &mov_op);

        // Expected result
        let expected = [
            Operation::MovFromStack(MovFromStack {
                stack_offset: 4,
                target: R3,
            }),
            Operation::Push(Push { register: R1 }),
            Operation::Pop(Pop { register: R2 }),
        ];

        assert_eq!(result, &expected);
    }
}
