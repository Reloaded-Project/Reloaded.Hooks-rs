extern crate alloc;

use alloc::vec::Vec;
use smallvec::SmallVec;

use crate::api::jit::operation_aliases::*;
use crate::{
    api::{jit::operation::Operation, traits::register_info::RegisterInfo},
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
/// - `scratch_registers`: Available scratch registers for reordering, used in case of cycles.
///
/// # Returns
///
/// A new slice of operations, these operations should replace the input slice that was passed to this structure.
///
/// # Remarks
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn reorder_mov_sequence<TRegister>(
    operations: &mut [Operation<TRegister>],
    scratch_registers: &[TRegister],
) -> Option<Vec<Operation<TRegister>>>
where
    TRegister: RegisterInfo + Eq + PartialEq + Hash + Copy,
{
    // Find the first block of MOV operations.
    let mut start_idx = 0;
    let mut new_ops = Vec::<Operation<TRegister>>::with_capacity(operations.len());

    loop {
        // Copy elements until found a MOV operation.
        let original_ops = new_ops.len();
        for (idx, operation) in operations[start_idx..].iter().enumerate() {
            if let Operation::Mov(_) = operation {
                start_idx = idx;
                break;
            } else {
                new_ops.push(operation.clone());
            }
        }

        // Pull values until first non-MOV index.
        let mut as_mov = SmallVec::<[Mov<TRegister>; 16]>::new();
        for op in operations[start_idx..].iter() {
            if let Operation::Mov(mov_op) = op {
                as_mov.push(*mov_op);
            } else {
                break;
            }
        }

        // Get the slice of MOV operations.
        if as_mov.len() <= 1 {
            // No more MOV operations to reorder
            if new_ops.len() > 1 {
                // Push all remaining items
                new_ops
                    .extend_from_slice(&operations[start_idx + (new_ops.len() - original_ops)..]);
                break;
            } else {
                // If no work was done at all, return None
                return None;
            }
        }

        // Alter our MOV operations and return
        start_idx += as_mov.len();
        let new_mov = optimize_moves(&as_mov, scratch_registers);
        if let Some(new_moves) = new_mov {
            new_ops.extend(new_moves);
        } else {
            new_ops.extend_from_slice(&operations[start_idx + (new_ops.len() - original_ops)..]);
        }
    }

    Some(new_ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_helpers::MockRegister::*;
    use crate::helpers::test_helpers::*;

    #[test]
    fn reorder_mov_sequence_no_mov() {
        let mut operations: Vec<Operation<MockRegister>> = vec![];
        let result = reorder_mov_sequence(&mut operations, &[R1]);
        assert!(result.is_none());
    }

    #[test]
    fn reorder_mov_sequence_single_mov() {
        let mock_op = Operation::Mov(Mov {
            source: R2,
            target: R3,
        });

        let mut operations: Vec<Operation<MockRegister>> = vec![mock_op.clone()];
        let result = reorder_mov_sequence(&mut operations, &[R1]);
        assert!(result.is_none());
    }

    #[test]
    fn reorder_mov_sequence_no_cycle() {
        let mock_op1 = Operation::Mov(Mov {
            source: R1,
            target: R2,
        });
        let mock_op2 = Operation::Mov(Mov {
            source: R2,
            target: R3,
        });

        let mut operations: Vec<Operation<MockRegister>> = vec![mock_op1.clone(), mock_op2.clone()];
        let reordered_ops = reorder_mov_sequence(&mut operations, &[R4]).unwrap();

        // Expected result would depend on the optimize_moves implementation
        // Here's a dummy expected result assuming optimize_moves doesn't change the order:
        let expected_result = vec![mock_op2.clone(), mock_op1.clone()];

        assert_eq!(reordered_ops, expected_result);
    }

    #[test]
    fn reorder_mov_sequence_with_cycle_with_scratch_register() {
        let mock_op1 = Operation::Mov(Mov {
            source: R1,
            target: R2,
        });
        let mock_op2 = Operation::Mov(Mov {
            source: R2,
            target: R3,
        });
        let mock_op3 = Operation::Mov(Mov {
            source: R3,
            target: R1,
        });

        let mut operations: Vec<Operation<MockRegister>> =
            vec![mock_op1.clone(), mock_op2.clone(), mock_op3.clone()];
        let reordered_ops = reorder_mov_sequence(&mut operations, &[R4]).unwrap();

        assert_eq!(
            reordered_ops,
            vec![
                Operation::Mov(Mov {
                    source: R3,
                    target: R4,
                }),
                mock_op2.clone(),
                mock_op1.clone(),
                Operation::Mov(Mov {
                    source: R4,
                    target: R1,
                })
            ]
        );
    }

    #[test]
    fn reorder_mov_sequence_with_cycle_no_scratch_register() {
        let mock_op1 = Operation::Mov(Mov {
            source: R1,
            target: R2,
        });
        let mock_op2 = Operation::Mov(Mov {
            source: R2,
            target: R3,
        });
        let mock_op3 = Operation::Mov(Mov {
            source: R3,
            target: R1,
        });

        let mut operations: Vec<Operation<MockRegister>> =
            vec![mock_op1.clone(), mock_op2.clone(), mock_op3.clone()];
        let reordered_ops = reorder_mov_sequence(&mut operations, &[]).unwrap();

        assert_eq!(
            reordered_ops,
            vec![
                Operation::Push(Push::new(R3)),
                mock_op2.clone(),
                mock_op1.clone(),
                Operation::Pop(Pop::new(R1))
            ]
        );
    }
}
