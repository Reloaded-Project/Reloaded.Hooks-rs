extern crate alloc;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::hash::{BuildHasherDefault, Hash};
use hashbrown::HashSet;
use nohash::NoHashHasher;

use crate::api::jit::operation::Operation;
use crate::api::jit::operation_aliases::*;
use crate::graphs::algorithms::move_graph_builder;
use crate::graphs::algorithms::move_validator::validate_moves;
use crate::graphs::node::Node;

/// Reorders a sequence of MOV register operations to prevent
/// them from writing invalid data.
///
/// # Parameter
///
/// - `moves`: The sequence of MOV register operations to create a valid order for.
/// - `scratch_register`: The scratch register to use for reordering, used in case of cycles.
///
/// # About
///
/// For more info about this, see `Design Docs -> Wrapper Generation`,
/// section `Reordering Operations`.
pub fn optimize_moves<T>(
    moves: &[Mov<T>],
    scratch_register: &Option<T>,
) -> Option<Vec<Operation<T>>>
where
    T: Eq + PartialEq + Hash + Clone,
{
    // Check if the moves are already valid.
    if (moves.is_empty()) || validate_moves(moves) {
        return None;
    }

    let mut results = Vec::<Operation<T>>::with_capacity(moves.len() * 2);
    let scratch_register: Option<T> = scratch_register.clone();
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

    Some(results)
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
                results.push(Operation::Xchg(XChg {
                    register1: node.borrow().value.clone(),
                    register2: neighbour.borrow().value.clone(),
                    scratch: None,
                }));

                return;
            }

            // Backup Register (or use scratch)
            if let Some(scratch) = scratch_register {
                results.push(Operation::Mov(Mov {
                    source: node.borrow().value.clone(),
                    target: scratch.clone(),
                }));
            } else {
                results.push(Operation::Push(Push {
                    register: node.borrow().value.clone(),
                }));
            }

            // Unwind without the element
            unwind(rec_stack, results);

            // Restore
            if let Some(scratch) = scratch_register {
                results.push(Operation::Mov(Mov {
                    source: scratch.clone(),
                    target: neighbour.borrow().value.clone(),
                }));
            } else {
                results.push(Operation::Pop(Pop {
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
        results.push(Operation::Mov(Mov {
            source: last.borrow().value.clone(),
            target: current.borrow().value.clone(),
        }));

        // Move element up.
        current_len = current_len.wrapping_sub(1);
    }
}

#[cfg(test)]
pub mod tests {
    use crate::api::jit::operation::Operation;
    use crate::api::jit::operation_aliases::*;
    use crate::graphs::algorithms::move_optimizer::optimize_moves;

    #[test]
    fn when_valid_moves_no_action() {
        let moves = vec![Mov {
            source: 1,
            target: 0,
        }];

        let new_operations = optimize_moves(&moves, &None);
        assert!(new_operations.is_none());
    }

    #[test]
    fn when_empty_moves_no_action() {
        let moves: Vec<Mov<i32>> = vec![];
        let new_operations = optimize_moves(&moves, &None);
        assert!(new_operations.is_none());
    }

    #[test]
    fn when_single_cyclic_move_use_xchg() {
        let moves = vec![
            Mov {
                source: 0,
                target: 1,
            },
            Mov {
                source: 1,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &None).unwrap();
        assert_eq!(
            new_operations,
            vec![Operation::Xchg(XChg {
                register1: 1,
                register2: 0,
                scratch: None,
            })]
        );
    }

    #[test]
    fn when_multiple_cyclic_moves_use_stack() {
        let moves = vec![
            Mov {
                source: 0,
                target: 1,
            },
            Mov {
                source: 1,
                target: 2,
            },
            Mov {
                source: 2,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &None).unwrap();
        assert_eq!(
            new_operations,
            vec![
                Operation::Push(Push { register: 2 }),
                Operation::Mov(Mov {
                    source: 1,
                    target: 2,
                }),
                Operation::Mov(Mov {
                    source: 0,
                    target: 1,
                }),
                Operation::Pop(Pop { register: 0 }),
            ]
        );
    }

    #[test]
    fn when_multiple_cyclic_moves_with_scratch_use_scratch() {
        let moves = vec![
            Mov {
                source: 0,
                target: 1,
            },
            Mov {
                source: 1,
                target: 2,
            },
            Mov {
                source: 2,
                target: 0,
            },
        ];

        let new_operations = optimize_moves(&moves, &Some(3)).unwrap();
        assert_eq!(
            new_operations,
            vec![
                Operation::Mov(Mov {
                    source: 2,
                    target: 3,
                }),
                Operation::Mov(Mov {
                    source: 1,
                    target: 2,
                }),
                Operation::Mov(Mov {
                    source: 0,
                    target: 1,
                }),
                Operation::Mov(Mov {
                    source: 3,
                    target: 0,
                }),
            ]
        );
    }
}
