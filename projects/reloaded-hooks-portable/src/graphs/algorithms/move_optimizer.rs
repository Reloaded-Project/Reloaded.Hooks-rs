extern crate alloc;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::hash::{BuildHasherDefault, Hash};
use hashbrown::HashSet;
use nohash::NoHashHasher;

use crate::api::jit::mov_operation::MovOperation;
use crate::api::jit::operation::Operation;
use crate::api::jit::pop_operation::PopOperation;
use crate::api::jit::push_operation::PushOperation;
use crate::api::jit::xchg_operation::XChgOperation;
use crate::graphs::algorithms::move_graph_builder;
use crate::graphs::algorithms::move_validator::validate_moves;
use crate::graphs::node::Node;

/// Reorders a sequence of MOV register operations to prevent
/// them from writing invalid data.
///
/// # Parameter
///
/// - `moves`: The sequence of MOV register operations to create a valid order for.
///
/// # About
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn optimize_moves<T>(moves: &[MovOperation<T>], scratch_registers: &[T]) -> Vec<Operation<T>>
where
    T: Eq + PartialEq + Hash + Clone,
{
    // Check if the moves are already valid.
    if (moves.is_empty()) || validate_moves(moves) {
        return moves
            .iter()
            .map(|mov| Operation::Mov(mov.clone()))
            .collect();
    }

    let mut results = Vec::<Operation<T>>::with_capacity(moves.len() * 2);
    let scratch_register: Option<T> = scratch_registers.first().cloned();
    let graph = move_graph_builder::build_graph(moves);
    let mut visited: HashSet<T, BuildHasherDefault<NoHashHasher<u32>>> =
        HashSet::with_capacity_and_hasher(graph.len(), BuildHasherDefault::default());
    let mut node_stack: Vec<Rc<RefCell<Node<T>>>> = Vec::with_capacity(graph.len());

    for node in &graph.values {
        node_stack.clear();
        if !visited.contains(&node.borrow().value) {
            dfs(
                node,
                &mut visited,
                &mut node_stack,
                &scratch_register,
                &mut results,
            );
        }
    }

    results
}

fn dfs<T: Eq + Clone + Hash>(
    node: &Rc<RefCell<Node<T>>>,
    visited: &mut HashSet<T, BuildHasherDefault<NoHashHasher<u32>>>,
    rec_stack: &mut Vec<Rc<RefCell<Node<T>>>>,
    scratch_register: &Option<T>,
    results: &mut Vec<Operation<T>>,
) {
    visited.insert(node.borrow().value.clone());
    rec_stack.push(node.clone());

    let borrowed = &node.borrow();

    if borrowed.edges.is_empty() {
        // The node is a leaf node. Rewind to top.
        unwind(rec_stack, results);
        return;
    }

    for neighbour in &borrowed.edges {
        if !visited.contains(&neighbour.borrow().value) {
            dfs(neighbour, visited, rec_stack, scratch_register, results);
        } else if rec_stack.contains(&neighbour.clone()) {
            // The node is in a cycle, disconnect the cycle, then unwind stack
            // In this case `neighbour` is the first node in the cycle, `node` is last before
            // first node again.

            // Special case: There are only 2 nodes, and they are in a cycle.
            // We can swap them directly on architectures like x86.
            if rec_stack.len() == 2 {
                results.push(Operation::Xchg(XChgOperation {
                    register1: node.borrow().value.clone(),
                    register2: neighbour.borrow().value.clone(),
                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                    scratch: None,
                }));

                return;
            }

            // Backup Register (or use scratch)
            if let Some(scratch) = scratch_register {
                results.push(Operation::Mov(MovOperation {
                    source: node.borrow().value.clone(),
                    target: scratch.clone(),
                }));
            } else {
                results.push(Operation::Push(PushOperation {
                    register: node.borrow().value.clone(),
                }));
            }

            // Unwind without the element
            unwind(rec_stack, results);

            // Restore
            if let Some(scratch) = scratch_register {
                results.push(Operation::Mov(MovOperation {
                    source: scratch.clone(),
                    target: neighbour.borrow().value.clone(),
                }));
            } else {
                results.push(Operation::Pop(PopOperation {
                    register: neighbour.borrow().value.clone(),
                }));
            }

            return;
        }
    }
}

fn unwind<T: Eq + Clone + Hash>(
    rec_stack: &[Rc<RefCell<Node<T>>>],
    results: &mut Vec<Operation<T>>,
) {
    let mut current_len = rec_stack.len();
    loop {
        let last_opt = rec_stack.get(current_len.wrapping_sub(2));

        if last_opt.is_none() {
            break;
        }

        let last = last_opt.unwrap();
        let current = rec_stack.get(current_len.wrapping_sub(1)).unwrap();

        // Encode this as a move.
        results.push(Operation::Mov(MovOperation {
            source: last.borrow().value.clone(),
            target: current.borrow().value.clone(),
        }));

        // Move element up.
        current_len = current_len.wrapping_sub(1);
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        api::jit::{
            mov_operation::MovOperation, operation::Operation, pop_operation::PopOperation,
            push_operation::PushOperation, xchg_operation::XChgOperation,
        },
        graphs::algorithms::move_optimizer::optimize_moves,
    };

    #[test]
    fn when_valid_moves_no_action() {
        let moves = vec![MovOperation {
            source: 1,
            target: 0,
        }];

        let original_operations: Vec<Operation<i32>> =
            moves.iter().map(|mov| Operation::Mov(*mov)).collect();

        let new_operations = optimize_moves(&moves, &[]);
        assert_eq!(new_operations, original_operations);
    }

    #[test]
    fn when_empty_moves_no_action() {
        let moves: Vec<MovOperation<i32>> = vec![];
        let new_operations = optimize_moves(&moves, &[]);
        assert!(new_operations.is_empty());
    }

    #[test]
    fn when_single_cyclic_move_use_xchg() {
        let moves = vec![
            MovOperation {
                source: 0,
                target: 1,
            },
            MovOperation {
                source: 1,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &[]);
        assert_eq!(
            new_operations,
            vec![Operation::Xchg(XChgOperation {
                register1: 1,
                register2: 0,
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                scratch: None,
            })]
        );
    }

    #[test]
    fn when_multiple_cyclic_moves_use_stack() {
        let moves = vec![
            MovOperation {
                source: 0,
                target: 1,
            },
            MovOperation {
                source: 1,
                target: 2,
            },
            MovOperation {
                source: 2,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &[]);
        assert_eq!(
            new_operations,
            vec![
                Operation::Push(PushOperation { register: 2 }),
                Operation::Mov(MovOperation {
                    source: 1,
                    target: 2,
                }),
                Operation::Mov(MovOperation {
                    source: 0,
                    target: 1,
                }),
                Operation::Pop(PopOperation { register: 0 }),
            ]
        );
    }

    #[test]
    fn when_multiple_cyclic_moves_with_scratch_use_scratch() {
        let moves = vec![
            MovOperation {
                source: 0,
                target: 1,
            },
            MovOperation {
                source: 1,
                target: 2,
            },
            MovOperation {
                source: 2,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &[3]);
        assert_eq!(
            new_operations,
            vec![
                Operation::Mov(MovOperation {
                    source: 2,
                    target: 3,
                }),
                Operation::Mov(MovOperation {
                    source: 1,
                    target: 2,
                }),
                Operation::Mov(MovOperation {
                    source: 0,
                    target: 1,
                }),
                Operation::Mov(MovOperation {
                    source: 3,
                    target: 0,
                }),
            ]
        );
    }
}
