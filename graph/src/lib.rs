use std::collections::HashMap;
use std::fmt::Debug;

use tex_core::TextureTransformer;

use petgraph::Direction::Incoming;
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::DiGraph;
use petgraph::visit::{Topo, EdgeRef, Bfs};

pub type NodeIndex = petgraph::graph::NodeIndex;

pub struct Node<T> {
    pub name: String,
    function: Box<dyn TextureTransformer<T>>
}

impl<T> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").field("name", &self.name).finish()
    }
}

impl<T> Node<T> {
    pub fn new(name: String, function: Box<dyn TextureTransformer<T>>) -> Self {
        Node {
            name,
            function: function
        }
    }
}

/// Main datastructure for a graph of transformers. Contains all nodes in the system, as well as generated values.
pub struct TextureGraph<T> {
    g: DiGraph<Node<T>, usize>,
    results: HashMap<NodeIndex, T>,
    cached: bool
}

impl<T> TextureGraph<T> {
    /// Creates a new Texture graph with no nodes or edges.
    pub fn new() -> Self{
        TextureGraph {
            g: DiGraph::new(),
            results: HashMap::new(),
            cached: false
         }
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }

    /// Returns a node in the graph based on the index.
    pub fn get_node(&self, index: NodeIndex) -> Option<&Node<T>> {
        self.g.node_weight(index)
    }

    /// Adds a new node to the graph, unconnected to any other nodes.
    pub fn add_node(&mut self, test_node: Node<T>) -> NodeIndex {
        self.cached = false;
        self.g.add_node(test_node)
    }

    /// Adds an edge between two nodes in the graph, with a given target input.
    /// Fails if:
    /// * Either the source or destination index does not exist in the graph.
    /// * The source and destination are the same node.
    /// * The added node would create a cycle.
    /// * The target is invalid for the destination node
    /// 
    /// In the case that an edge to the destination already exists with the given target input, the new edges replaces the old.
    pub fn add_edge(&mut self, src: NodeIndex, dest: NodeIndex, target_input: usize) -> Result<(), String> {
        if self.g.node_weight(src).is_none() {
            return Err(format!("Unknown node {:?}", src));
        }
        if self.g.node_weight(dest).is_none() {
            return Err(format!("Unknown node {:?}", dest));
        }
        if src == dest {
            return Err(format!("Self feeding node {:?}", src))
        }
        let dest_node = &self.g[dest];
        if target_input >= dest_node.function.inputs() {
            return Err(format!("Invalid target {} for node {:?}", target_input, dest_node));
        }
        match self.g.edges_directed(dest, Incoming)
                    .find(|edge| *edge.weight() == target_input) {
            Some(e) => {
                let old_source = e.source().clone();
                let old_target = e.target().clone();
                let old_weight = e.weight().clone();
                self.g.remove_edge(e.id());
                let new_edge = self.g.add_edge(src, dest, target_input);
                if is_cyclic_directed(&self.g) {
                    self.g.add_edge(old_source, old_target, old_weight);
                    self.g.remove_edge(new_edge);
                    Err(String::from("Edge would create cycle"))
                } else {
                    self.cached = false;
                    self.invalidate_nodes(dest);
                    Ok(())
                }
            },
            None => {
                let edge = self.g.add_edge(src, dest, target_input);
                if is_cyclic_directed(&self.g) {
                    self.g.remove_edge(edge);
                    Err(String::from("Edge would create cycle"))
                } else {
                    self.cached = false;
                    Ok(())
                }
            },
        }
    }

    /// Removes the results of all nodes reachable from the source node.
    /// This can be used when regenerating a node to lazily propogate the regeneration to other nodes.
    pub fn invalidate_nodes(&mut self, source_index: NodeIndex) {
        let mut bfs = Bfs::new(&self.g, source_index);
        while let Some(nx) = bfs.next(&self.g) {
            self.results.remove(&nx);
        }
    }

    /// Checks if all targets of a given node are connected by edges..
    pub fn node_complete(&self, node_index: NodeIndex) -> bool {
        let node = &self.g[node_index];
        node.function.inputs() == self.g.neighbors_directed(node_index, Incoming).count()
    }

    /// Checks if all targets of all nodes are connected by edges.
    pub fn graph_complete(&self) -> bool {
        self.g.node_indices().all(|i| self.node_complete(i))
    }

    /// Returns the generated value of a given node index.
    pub fn get_result(&self, index: &NodeIndex) -> Option<&T> {
        match self.cached {
            true => Some(&self.results[index]),
            false => None
        }
    }

    /// Generates the value of a given node.
    /// Fails if:
    /// * Node is not connected by enough targets
    /// * An input of an input node is not generated
    /// * Inputs of the node function are not valid.
    pub fn generate_node(&mut self, index: NodeIndex) -> Result<(), String> {
        if !self.node_complete(index) {
            return Err(String::from("Node not completed"));
        }
        let mut inputs: Vec<_> = self.g.edges_directed(index, Incoming)
            .map(|e| {
                let source = e.source();
                let target = e.weight();
                (*target, source)
            }).collect();
        inputs.sort_by_key(|(t, _)| *t);
        let targets: Vec<_> = inputs.iter().map(|(_, source)| source).collect();
        if targets.iter().any(|src| !self.results.contains_key(src)) {
            return Err(String::from("Predecessors of node not generated"))
        }
        let targets = targets.iter().map(|&target| &self.results[target]).collect();
        let node = &self.g[index];
        if !node.function.is_valid(&targets) {
            return Err(String::from("Input images not valid"))
        }
        let generated_value = node.function.generate(targets);
        self.results.insert(index, generated_value);
        Ok(())
    }
    
    /// Generates the entire graph in a topological order.
    /// This function does not skip any previously generated nodes.
    pub fn generate_graph(&mut self) -> Result<(), String> {
        let mut topo = Topo::new(&self.g);
        while let Some(index) = topo.next(&self.g) {
            match self.generate_node(index) {
                Ok(_) => continue,
                Err(msg) => return Err(msg),
            }
        }
        Ok(())
    }

    /// Generates the entire graph in a topological order.
    /// This function skips any previously generated nodes.
    pub fn generate_graph_missing(&mut self) -> Result<(), String> {
        let mut topo = Topo::new(&self.g);
        while let Some(index) = topo.next(&self.g) {
            if self.results.contains_key(&index) {
                continue
            }
            match self.generate_node(index) {
                Ok(_) => continue,
                Err(msg) => return Err(msg),
            }
        }
        Ok(())
    }    

    /// Returns the generated value of a given index if it exists.
    pub fn get_generated_node(&mut self, index: &NodeIndex) -> Option<&T> {
        self.results.get(index)
    }
}

#[cfg(test)]
mod tests {
    use petgraph::algo::is_cyclic_directed;

    use crate::{TextureGraph, Node, TextureTransformer};

    struct Add{}
    impl TextureTransformer<i32> for Add {
        fn generate(&self, inputs: Vec<&i32>) -> i32 {
            inputs.iter().copied().sum()
        }

        fn inputs(&self) -> usize {
            2
        }
    }

    struct Const(i32);
    impl TextureTransformer<i32> for Const {
        fn generate(&self, _inputs: Vec<&i32>) -> i32 {
            self.0
        }

        fn inputs(&self) -> usize {
            0
        }
    }

    struct Double{}
    impl TextureTransformer<i32> for Double {
        fn generate(&self, inputs: Vec<&i32>) -> i32 {
            inputs[0] * 2
        }

        fn inputs(&self) -> usize {
            1
        }

        fn is_valid(&self, inputs: &Vec<&i32>) -> bool {
            inputs.len() > 0 && *inputs[0] >= 0
        }
    }

    #[test]
    fn new_graph() {
        let graph = TextureGraph::<i32>::new();
        assert_eq!(0, graph.node_count());
    }

    #[test]
    fn new_nodes() {
        let node = Node::new(String::from("Test node"), Box::new(Add{}));
        assert_eq!(String::from("Test node"), node.name);
    }

    #[test]
    fn add_node() {
        let mut graph = TextureGraph::<i32>::new();
        let test_node = Node::new(String::from("Test node"), Box::new(Add{}));
        graph.add_node(test_node);
        assert_eq!(1, graph.node_count());
    }
    
    #[test]
    fn single_edge() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert_eq!(1, graph.g.edge_count());
    }


    #[test]
    fn double_edge() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert!(graph.add_edge(index1, index2, 1).is_ok());
        assert_eq!(2, graph.g.edge_count());
    }
    

    #[test]
    fn replace_edge() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert_eq!(1, graph.g.edge_count());
    }

    #[test]
    fn overwrite_edge() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.add_edge(index2, index3, 0).is_ok());
        assert_eq!(1, graph.g.edge_count());
        assert!(graph.g.find_edge(index1, index3).is_none());
        assert!(graph.g.find_edge(index2, index3).is_some());
    }

    #[test]
    fn edge_self_feed() {
        let mut graph = TextureGraph::<i32>::new();
        let node = Node::new(String::from("N"), Box::new(Add{}));
        let index = graph.add_node(node);
        assert!(graph.add_edge(index, index, 0).is_err());
    }

    #[test]
    fn unknown_node() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N"), Box::new(Add{}));
        let node2 = Node::new(String::from("N"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        graph.g.remove_node(index2);
        assert!(graph.add_edge(index1, index2, 0).is_err());
        assert_eq!(0, graph.g.edge_count());
        assert!(graph.add_edge(index2, index1, 0).is_err());
        assert_eq!(0, graph.g.edge_count());
    }

    #[test]
    fn no_cycles() {
        let mut graph: TextureGraph<i32> = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(!is_cyclic_directed(&graph.g));
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert!(graph.add_edge(index2, index1, 0).is_err());
        assert_eq!(1, graph.g.edge_count());
        assert!(!is_cyclic_directed(&graph.g));
        assert!(graph.add_edge(index2, index3, 0).is_ok());
        assert!(graph.add_edge(index3, index1, 0).is_err());
        assert_eq!(2, graph.g.edge_count());
    }

    #[test]
    fn no_cycles_overwrite() {
        let mut graph: TextureGraph<i32> = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Add{}));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(!is_cyclic_directed(&graph.g));
        assert!(graph.add_edge(index1, index2, 0).is_ok());
        assert!(graph.add_edge(index2, index1, 0).is_err());
        assert!(graph.g.find_edge(index1, index2).is_some());
        assert!(graph.g.find_edge(index2, index1).is_none());
        assert_eq!(1, graph.g.edge_count());
        assert!(!is_cyclic_directed(&graph.g));
        assert!(graph.add_edge(index2, index3, 0).is_ok());
        assert!(graph.add_edge(index3, index2, 0).is_err());
        assert!(graph.g.find_edge(index2, index3).is_some());
        assert!(graph.g.find_edge(index3, index2).is_none());
        assert_eq!(2, graph.g.edge_count());
    }

    #[test]
    fn invalid_target() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Add{}));
        let node2 = Node::new(String::from("N2"), Box::new(Const(3)));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        assert!(graph.add_edge(index1, index2, 0).is_err());
        assert!(graph.add_edge(index2, index1, 2).is_err());
    }

    #[test]
    fn complete_nodes() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(!graph.graph_complete());
        assert!(graph.node_complete(index1));
        assert!(graph.node_complete(index2));
        assert!(!graph.node_complete(index3));
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(!graph.node_complete(index3));
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(graph.node_complete(index1));
        assert!(graph.node_complete(index2));
        assert!(graph.node_complete(index3));
        assert!(graph.graph_complete());
    }

    #[test]
    fn generate_node() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(graph.generate_node(index1).is_ok());
        assert!(graph.generate_node(index2).is_ok());
        assert!(graph.generate_node(index3).is_ok());
        assert!(*graph.get_generated_node(&index1).unwrap() == 1);
        assert!(*graph.get_generated_node(&index2).unwrap() == 2);
        assert!(*graph.get_generated_node(&index3).unwrap() == 3);
    }

    #[test]
    fn generate_not_complete() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let _index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.generate_node(index3).is_err());
        assert!(graph.get_generated_node(&index3).is_none());
    }

    #[test]
    fn generate_previous_not_generated() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(graph.generate_graph().is_ok());
        assert!(*graph.get_generated_node(&index1).unwrap() == 1);
        assert!(*graph.get_generated_node(&index2).unwrap() == 2);
        assert!(*graph.get_generated_node(&index3).unwrap() == 3);
    }

    #[test]
    fn generate_complete_graph() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
    }

    #[test]
    fn add_edge_invalidate() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let node4 = Node::new(String::from("N4"), Box::new(Double{}));
        let node5 = Node::new(String::from("N2"), Box::new(Const(3)));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        let index4 = graph.add_node(node4);
        let index5 = graph.add_node(node5);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(graph.add_edge(index3, index4, 0).is_ok());
        assert!(graph.generate_graph().is_ok());
        assert!(graph.add_edge(index5, index3, 0).is_ok());
        assert!(graph.get_generated_node(&index1).is_some());
        assert!(graph.get_generated_node(&index2).is_some());
        assert!(graph.get_generated_node(&index3).is_none());
        assert!(graph.get_generated_node(&index4).is_none());
        assert!(graph.get_generated_node(&index5).is_some());
    }

    #[test]
    fn valid_targets() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(-1)));
        let node2 = Node::new(String::from("N2"), Box::new(Double{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        graph.add_edge(index1, index2, 0).unwrap();
        graph.generate_node(index1).unwrap();
        assert!(graph.generate_node(index2).is_err());

    }

}