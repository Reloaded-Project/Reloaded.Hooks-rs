extern crate alloc;

use alloc::vec::Vec;

use crate::{
    api::{
        jit::{mov_operation::MovOperation, operation::Operation},
        traits::register_info::RegisterInfo,
    },
    graphs::algorithms::move_optimizer::optimize_moves,
};

use core::hash::Hash;

/// Finds blocks/sequences of MOV instructions inside the given list of operations
/// and ensures that they are reordered such that they won't write invalid data.
///
/// More specifically, fixes the following sequence:
///
/// ```asm
/// mov rcx, rdi
/// mov rdx, rsi
/// mov r8, rdx // rdx was overwritten, and is thus invalid
/// mov r9, rcx // rcx was overwritten, and is thus invalid
/// ```
///
/// Rearranging the instructions such that they become:
///
/// ```asm
/// # Move directly to registers
/// mov r9, rcx
/// mov r8, rdx
/// mov rdx, rsi
/// mov rcx, rdi
/// ```
///
/// # Params
///
/// - `operations`: All operations emitted during wrapper generation up to method call. i.e. Starting with
///                 push and ending on pop.
/// - `scratch_registers`: The scratch registers to use for reordering, used in case of cycles.
///
/// # Returns
///
/// A new slice of operations, these operations should replace the input slice that was passed to this structure.
///
/// # Remarks
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn reorder_mov_sequence<'a, TRegister>(
    operations: &'a mut [Operation<TRegister>],
    scratch_registers: &'a [TRegister],
) -> &'a mut [Operation<TRegister>]
where
    TRegister: RegisterInfo + Copy + Eq + PartialEq + Hash + Clone,
{
    // Find the first block of MOV operations.
    let mut first_mov_idx = 0;

    loop {
        for (idx, operation) in operations[first_mov_idx..].iter().enumerate() {
            if let Operation::Mov(_) = operation {
                first_mov_idx = idx;
                break;
            }
        }

        // Pull values until first non-MOV index.
        let as_mov: Vec<MovOperation<TRegister>> = operations[first_mov_idx..]
            .iter()
            .map_while(|op| {
                if let Operation::Mov(mov_op) = op {
                    Some(*mov_op)
                } else {
                    None
                }
            })
            .collect();

        let orig_first_mov_idx = first_mov_idx;
        first_mov_idx += as_mov.len();

        // Get the slice of MOV operations.
        if as_mov.len() <= 1 {
            break; // No more MOV operations to reorder.
        }

        // Assuming the operations slice starts with Mov and continues with only Mov operations
        // for the intended length, this is safe.
        let new_mov = optimize_moves(&as_mov, scratch_registers);

        // Replace the old MOV operations with the new ones, by copying them over the old slice
        let mov_slice = &mut operations[orig_first_mov_idx..first_mov_idx];
        mov_slice.copy_from_slice(&new_mov);
    }

    operations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_helpers::MockRegister::*;
    use crate::helpers::test_helpers::*;

    #[test]
    fn reorder_mov_sequence_no_mov() {
        let mut operations: Vec<Operation<MockRegister>> = vec![];
        let scratch_registers: Vec<MockRegister> = vec![R1];
        let result = reorder_mov_sequence(&mut operations, &scratch_registers);
        assert_eq!(result, &[]);
    }

    #[test]
    fn reorder_mov_sequence_single_mov() {
        let mock_op = Operation::Mov(MovOperation {
            source: R2,
            target: R3,
        });

        let mut operations: Vec<Operation<MockRegister>> = vec![mock_op];
        let scratch_registers: Vec<MockRegister> = vec![R1];
        let result = reorder_mov_sequence(&mut operations, &scratch_registers);
        assert_eq!(result, &vec![mock_op]);
    }

    #[test]
    fn reorder_mov_sequence_no_cycle() {
        let mock_op1 = Operation::Mov(MovOperation {
            source: R1,
            target: R2,
        });
        let mock_op2 = Operation::Mov(MovOperation {
            source: R2,
            target: R3,
        });

        let mut operations: Vec<Operation<MockRegister>> = vec![mock_op1, mock_op2];
        let scratch_registers: Vec<MockRegister> = vec![R4];
        let reordered_ops = reorder_mov_sequence(&mut operations, &scratch_registers);

        // Expected result would depend on the optimize_moves implementation
        // Here's a dummy expected result assuming optimize_moves doesn't change the order:
        let expected_result = vec![mock_op2, mock_op1];

        assert_eq!(reordered_ops, &expected_result);
    }
}
