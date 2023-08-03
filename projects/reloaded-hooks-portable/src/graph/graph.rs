extern crate alloc;
use alloc::{sync::Arc, vec::Vec};

use super::node::Node;

/// The graph containing all the nodes.
#[derive(Debug, PartialEq, Eq)]
pub struct Graph<T>
where
    T: PartialEq + Eq,
{
    /// The nodes stored inside this graph
    pub values: Vec<Arc<Node<T>>>,
}

impl<T> Graph<T>
where
    T: PartialEq + Eq,
{
    /// Creates a new graph.
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: Arc<Node<T>>) {
        self.values.push(node);
    }
}

impl<T> Default for Graph<T>
where
    T: PartialEq + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
