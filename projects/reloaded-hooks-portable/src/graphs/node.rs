extern crate alloc;
use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

/// Represents a node in a graph structure.
///
/// Each node contains a value of some generic, Eq and PartialEq-compliant type `T`,
/// and edges to other nodes. An edge from one node to another implies that they are directly connected
/// in the graph.
///
/// # Example
///
/// ```
/// extern crate alloc;
/// use reloaded_hooks_portable::graphs::node::Node;
/// use alloc::rc::Rc;
/// use std::cell::RefCell;
///
/// let value = 5;
/// let node = Node::new(value);
/// assert_eq!(node.value, value);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<T>
where
    T: PartialEq + Eq,
{
    /// The value of type `T` stored in the node.
    pub value: T,

    /// The collection of nodes to which this node is directly connected.
    pub edges: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T> Node<T>
where
    T: PartialEq + Eq,
{
    /// Creates a new node with the provided value.
    ///
    /// The new node will not be connected to any other nodes.
    ///
    /// # Parameters
    ///
    /// - `value`: The value to be stored in the node.
    ///
    /// # Returns
    ///
    /// A new `Node` instance with the provided value.
    ///
    /// # Example
    ///
    /// ```
    /// use reloaded_hooks_portable::graphs::node::Node;
    /// use std::cell::RefCell;
    ///
    /// let value = 5;
    /// let node = Node::new(value);
    /// assert_eq!(node.value, value);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            value,
            edges: Vec::new(),
        }
    }

    /// Adds an edge from this node to the provided node.
    ///
    /// This creates a direct link from this node to the given node.
    ///
    /// # Parameters
    ///
    /// - `node`: The node to connect to.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate alloc;
    /// use reloaded_hooks_portable::graphs::node::Node;
    /// use alloc::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// let value1 = 5;
    /// let mut node1 = Node::new(value1);
    ///
    /// let value2 = 10;
    /// let node2 = Node::new(value2);
    ///
    /// node1.add_edge(Rc::new(RefCell::new(node2)));
    ///
    /// assert_eq!(node1.edges[0].borrow().value, value2);
    /// ```
    pub fn add_edge(&mut self, node: Rc<RefCell<Node<T>>>) {
        self.edges.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let value = 5;
        let node = Node::new(value);

        assert_eq!(node.value, value);
        assert!(node.edges.is_empty());
    }

    #[test]
    fn test_add_edge() {
        let value1 = 5;
        let mut node1 = Node::new(value1);

        let value2 = 10;
        let node2 = Node::new(value2);

        node1.add_edge(Rc::new(RefCell::new(node2.clone())));

        assert_eq!(node1.edges.len(), 1);
        assert_eq!(node1.edges[0].borrow().value, value2);
    }
}
