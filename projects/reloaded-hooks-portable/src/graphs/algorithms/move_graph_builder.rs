extern crate alloc;

use core::cell::RefCell;
use core::hash::BuildHasherDefault;
use core::hash::Hash;

use crate::api::jit::mov_operation::MovOperation;
use crate::graphs::{graph::Graph, node::Node};
use alloc::rc::Rc;
use hashbrown::HashMap;
use nohash::NoHashHasher;

// Define the new type.
type NodeMap<T> = HashMap<T, Rc<RefCell<Node<T>>>, BuildHasherDefault<NoHashHasher<u32>>>;

/// Builds a graph from a sequence of MOV register operations.
///
/// # Parameters
/// - `moves`: The sequence of MOV register operations to build a graph for.
///
/// # Returns
///
/// A complete graph, for example the sequence:
/// - mov eax, ebx
/// - mov ecx, eax
///
/// Would be represented as
/// ebx -> eax -> ecx
pub fn build_graph<T: Eq + Clone + Hash>(moves: &[MovOperation<T>]) -> Graph<T> {
    let mut graph = Graph::<T>::new();
    let mut nodes: NodeMap<T> =
        HashMap::with_capacity_and_hasher(moves.len(), BuildHasherDefault::default());

    // Create all nodes first.
    for mov in moves {
        nodes
            .entry(mov.source.clone())
            .or_insert_with(|| Rc::new(RefCell::new(Node::new(mov.source.clone()))));

        nodes
            .entry(mov.target.clone())
            .or_insert_with(|| Rc::new(RefCell::new(Node::new(mov.target.clone()))));
    }

    // Create edges for each move operation.
    for mov in moves {
        let source_node = nodes.get(&mov.source).unwrap().clone();
        let target_node = nodes.get(&mov.target).unwrap().clone();

        source_node.borrow_mut().add_edge(target_node);
    }

    // Add all nodes to the graph.
    for node in nodes.values() {
        graph.add_node(node.clone());
    }

    graph
}

#[cfg(test)]
mod tests {
    use crate::api::jit::mov_operation::MovOperation;

    use super::*;

    #[test]
    fn test_build_graph_empty() {
        let moves: Vec<MovOperation<u32>> = Vec::new();
        let graph = build_graph(&moves);

        assert_eq!(graph.values.len(), 0);
    }

    #[test]
    fn test_build_graph_single_move() {
        let moves = vec![MovOperation {
            source: 1,
            target: 2,
        }];

        let graph = build_graph(&moves);

        assert_eq!(graph.values.len(), 2);
        assert_eq!(graph.values[0].borrow().value, 1);
        assert_eq!(graph.values[1].borrow().value, 2);
        assert_eq!(graph.values[0].borrow().edges.len(), 1);
        assert_eq!(graph.values[0].borrow().edges[0].borrow().value, 2);
    }

    #[test]
    fn test_build_graph_multiple_moves() {
        let moves = vec![
            MovOperation {
                source: 1,
                target: 2,
            },
            MovOperation {
                source: 2,
                target: 3,
            },
        ];

        let graph = build_graph(&moves);

        assert_eq!(graph.values.len(), 3);
        assert_eq!(graph.values[0].borrow().value, 1);
        assert_eq!(graph.values[1].borrow().value, 2);
        assert_eq!(graph.values[2].borrow().value, 3);
        assert_eq!(graph.values[0].borrow().edges.len(), 1);
        assert_eq!(graph.values[0].borrow().edges[0].borrow().value, 2);
        assert_eq!(graph.values[1].borrow().edges.len(), 1);
        assert_eq!(graph.values[1].borrow().edges[0].borrow().value, 3);
    }
}
