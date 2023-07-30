use std::sync::Arc;

/// A node in a graph.
pub struct Node<T> {
    /// The value stored inside this node.
    pub value: T,

    /// The other nodes this node corrects to.
    pub edges: Vec<Arc<Node<T>>>,
}

impl<T> Node<T> {
    /// Creates a new node with the given value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            edges: Vec::new(),
        }
    }

    /// Adds an edge to the given node.
    pub fn add_edge(&mut self, node: Arc<Node<T>>) {
        self.edges.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
        let arc_node2 = Arc::new(node2);

        node1.add_edge(arc_node2.clone());

        assert_eq!(node1.edges.len(), 1);
        assert_eq!(Arc::strong_count(&arc_node2), 2);
        assert_eq!(node1.edges[0].value, value2);
    }
}
