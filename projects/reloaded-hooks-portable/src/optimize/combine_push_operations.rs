extern crate alloc;

use crate::api::jit::operation::Operation;
use alloc::vec::Vec;

macro_rules! deduplicate_merge_ops {
    ($name:ident, $op:ident, $op_name:ident) => {
        /// Merges sequential $op operations within a given sequence of operations into a single
        /// 'Multi$op' operation.
        ///
        /// # Parameters
        ///
        /// - `operations` - The operations where $op will be merged.
        ///
        /// # Remarks
        ///
        /// This is an optional step that can be applied within the structs that implement the JIT trait.
        /// It can be used to optimise series of multiple $ops wherever possible.
        pub(crate) fn $name<TRegister: Clone>(
            operations: &mut [Operation<TRegister>],
        ) -> &mut [Operation<TRegister>] {
            let mut read_idx = 0;
            let mut write_idx = 0;
            while read_idx < operations.len() {
                match &operations[read_idx] {
                    Operation::$op(_) => {
                        let mut ops = Vec::new();

                        // Collect sequential $op Operations
                        while read_idx < operations.len()
                            && matches!(operations[read_idx], Operation::$op(_))
                        {
                            if let Operation::$op(op) = &operations[read_idx] {
                                ops.push(op.clone());
                            }
                            read_idx += 1;
                        }

                        // If there's more than one $op Operation, replace them with a Multi$op
                        if ops.len() > 1 {
                            operations[write_idx] = Operation::$op_name(ops);
                        } else {
                            // If there's only one, just copy the $op Operation
                            operations[write_idx] = operations[read_idx - 1].clone();
                        }
                        write_idx += 1;
                    }
                    // For all other operations, simply copy them over
                    _ => {
                        operations[write_idx] = operations[read_idx].clone();
                        read_idx += 1;
                        write_idx += 1;
                    }
                }
            }

            &mut operations[..write_idx]
        }
    };
}

deduplicate_merge_ops!(merge_push_operations, Push, MultiPush);
deduplicate_merge_ops!(merge_pop_operations, Pop, MultiPop);

#[cfg(test)]
mod tests {
    use crate::{
        api::jit::{
            mov_operation::MovOperation, pop_operation::PopOperation, push_operation::PushOperation,
        },
        helpers::test_helpers::MockRegister,
    };

    use super::*;

    #[test]
    fn merge_push_operations_baseline() {
        let mut ops = vec![
            Operation::Push(PushOperation {
                register: MockRegister::R1,
            }),
            Operation::Push(PushOperation {
                register: MockRegister::R2,
            }),
            Operation::Mov(MovOperation {
                source: MockRegister::R3,
                target: MockRegister::R4,
            }),
            Operation::Push(PushOperation {
                register: MockRegister::R3,
            }),
        ];

        let result = merge_push_operations(&mut ops);
        assert_eq!(result.len(), 3);
        match &result[0] {
            Operation::MultiPush(pushes) => {
                assert_eq!(pushes.len(), 2);
                assert_eq!(pushes[0].register, MockRegister::R1);
                assert_eq!(pushes[1].register, MockRegister::R2);
            }
            _ => panic!("Expected MultiPush operation"),
        }
        match &result[2] {
            Operation::Push(push_op) => {
                assert_eq!(push_op.register, MockRegister::R3);
            }
            _ => panic!("Expected Push operation"),
        }
    }

    #[test]
    fn merge_pop_operations_baseline() {
        let mut ops = vec![
            Operation::Pop(PopOperation {
                register: MockRegister::R1,
            }),
            Operation::Pop(PopOperation {
                register: MockRegister::R2,
            }),
            Operation::Mov(MovOperation {
                source: MockRegister::R3,
                target: MockRegister::R4,
            }),
            Operation::Pop(PopOperation {
                register: MockRegister::R3,
            }),
        ];

        let result = merge_pop_operations(&mut ops);
        assert_eq!(result.len(), 3);
        match &result[0] {
            Operation::MultiPop(pops) => {
                assert_eq!(pops.len(), 2);
                assert_eq!(pops[0].register, MockRegister::R1);
                assert_eq!(pops[1].register, MockRegister::R2);
            }
            _ => panic!("Expected MultiPop operation"),
        }
        match &result[2] {
            Operation::Pop(pop_op) => {
                assert_eq!(pop_op.register, MockRegister::R3);
            }
            _ => panic!("Expected Pop operation"),
        }
    }
}
