extern crate alloc;
use crate::api::jit::operation::Operation;
use alloc::vec::Vec;
use smallvec::SmallVec;

macro_rules! deduplicate_merge_ops {
    ($name:ident, $op:ident, $op_name:ident) => {
        /// Merges sequential $op operations within a given sequence of operations into a single
        /// 'Multi$op' operation.
        ///
        /// # Parameters
        ///
        /// - `operations` - The vector of operations where $op will be merged.
        ///
        /// # Remarks
        ///
        /// This is an optional step that can be applied within the structs that implement the JIT trait.
        /// It can be used to optimize series of multiple $ops wherever possible.
        #[allow(dead_code)]
        pub(crate) fn $name<TRegister: Copy + Clone>(operations: &mut Vec<Operation<TRegister>>) {
            let mut read_idx = 0;
            let mut write_idx = 0;
            while read_idx < operations.len() {
                if let Operation::$op(_) = operations[read_idx] {
                    let mut ops = SmallVec::new();

                    // Collect sequential $op (Push/Pop) Operations
                    while read_idx < operations.len() {
                        if let Operation::$op(op) = operations[read_idx] {
                            ops.push(op);
                            read_idx += 1;
                        } else {
                            break;
                        }
                    }

                    // If there's more than one $op Operation, replace them with a Multi$op
                    if ops.len() > 1 {
                        operations[write_idx] = Operation::$op_name(ops);
                    } else {
                        // Only one $op Operation, no need to replace
                        if read_idx - 1 != write_idx {
                            operations.swap(write_idx, read_idx - 1);
                        }
                    }
                    write_idx += 1;
                } else {
                    // For all other operations, simply move them forward if necessary
                    if read_idx != write_idx {
                        operations.swap(read_idx, write_idx);
                    }
                    read_idx += 1;
                    write_idx += 1;
                }
            }

            // Truncate the vector to remove unused elements
            operations.truncate(write_idx);
        }
    };
}

deduplicate_merge_ops!(merge_push_operations, Push, MultiPush);
deduplicate_merge_ops!(merge_pop_operations, Pop, MultiPop);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::api::jit::operation_aliases::*;
    use crate::helpers::test_helpers::MockRegister;

    #[test]
    fn merge_push_operations_baseline() {
        let mut ops = vec![
            Operation::Push(Push {
                register: MockRegister::R1,
            }),
            Operation::Push(Push {
                register: MockRegister::R2,
            }),
            Operation::Mov(Mov {
                source: MockRegister::R3,
                target: MockRegister::R4,
            }),
            Operation::Push(Push {
                register: MockRegister::R3,
            }),
        ];

        merge_push_operations(&mut ops);
        assert_eq!(ops.len(), 3);
        match &ops[0] {
            Operation::MultiPush(pushes) => {
                assert_eq!(pushes.len(), 2);
                assert_eq!(pushes[0].register, MockRegister::R1);
                assert_eq!(pushes[1].register, MockRegister::R2);
            }
            _ => panic!("Expected MultiPush operation"),
        }
        match &ops[2] {
            Operation::Push(push_op) => {
                assert_eq!(push_op.register, MockRegister::R3);
            }
            _ => panic!("Expected Push operation"),
        }
    }

    #[test]
    fn merge_pop_operations_baseline() {
        let mut ops = vec![
            Operation::Pop(Pop {
                register: MockRegister::R1,
            }),
            Operation::Pop(Pop {
                register: MockRegister::R2,
            }),
            Operation::Mov(Mov {
                source: MockRegister::R3,
                target: MockRegister::R4,
            }),
            Operation::Pop(Pop {
                register: MockRegister::R3,
            }),
        ];

        merge_pop_operations(&mut ops);
        assert_eq!(ops.len(), 3);
        match &ops[0] {
            Operation::MultiPop(pops) => {
                assert_eq!(pops.len(), 2);
                assert_eq!(pops[0].register, MockRegister::R1);
                assert_eq!(pops[1].register, MockRegister::R2);
            }
            _ => panic!("Expected MultiPop operation"),
        }
        match &ops[2] {
            Operation::Pop(pop_op) => {
                assert_eq!(pop_op.register, MockRegister::R3);
            }
            _ => panic!("Expected Pop operation"),
        }
    }
}
