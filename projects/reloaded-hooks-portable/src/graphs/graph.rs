extern crate alloc;
use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

use super::node::Node;

/// Represents a graph data structure.
///
/// A `Graph` is a collection of nodes, where each `Node` contains a value of some generic, Eq and PartialEq-compliant type `T`,
/// and edges to other nodes. An edge from one node to another implies that they are directly connected in the graph.
///
/// In this library, a `Graph` is used to represent a sequence of MOV register operations.
///
/// # Example
///
/// ```ignore
/// extern crate alloc;
/// use reloaded_hooks_portable::graphs::node::Node;
/// use reloaded_hooks_portable::graphs::graph::Graph;
/// use alloc::{rc::Rc};
/// use std::cell::RefCell;
///
/// let mut graph = Graph::new();
///
/// let value = 5;
/// let node = Rc::new(RefCell::new(Node::new(value)));
///
/// graph.add_node(node);
///
/// assert_eq!(graph.values.len(), 1);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Graph<T>
where
    T: PartialEq + Eq,
{
    /// The nodes stored inside this graph
    pub values: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T> Graph<T>
where
    T: PartialEq + Eq,
{
    /// Creates a new graph.
    ///
    /// The new graph will not contain any nodes.
    ///
    /// # Returns
    ///
    /// A new `Graph` instance with no nodes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use reloaded_hooks_portable::graphs::graph::Graph;
    ///
    /// let graph = Graph::<i32>::new();
    ///
    /// assert_eq!(graph.values.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Adds a node to the graph.
    ///
    /// This node becomes part of the graph's node collection.
    ///
    /// # Parameters
    ///
    /// - `node`: The node to add to the graph.
    ///
    /// # Example
    ///
    /// ```ignore
    /// extern crate alloc;
    /// use reloaded_hooks_portable::graphs::node::Node;
    /// use reloaded_hooks_portable::graphs::graph::Graph;
    /// use alloc::{rc::Rc};
    /// use std::cell::RefCell;
    ///
    /// let mut graph = Graph::new();
    ///
    /// let value = 5;
    /// let node = Rc::new(RefCell::new(Node::new(value)));
    ///
    /// graph.add_node(node.clone());
    ///
    /// assert_eq!(graph.values.len(), 1);
    /// assert_eq!(Rc::strong_count(&node), 2);
    /// ```
    pub fn add_node(&mut self, node: Rc<RefCell<Node<T>>>) {
        self.values.push(node);
    }

    /// Returns the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.values.len() == 0
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
