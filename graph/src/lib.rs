use std::collections::HashMap;
use std::fmt::Debug;

use petgraph::Direction::Incoming;
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Topo, EdgeRef};

pub trait TextureTransformer<T> {
    fn generate(&self, inputs: Vec<&T>) -> T;
    fn inputs(&self) -> usize;
}

pub struct Node<T> {
    name: String,
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

pub struct TextureGraph<T> {
    g: DiGraph<Node<T>, usize>,
    results: HashMap<NodeIndex, T>,
    cached: bool
}

impl<T> TextureGraph<T> {
    pub fn new() -> Self{
        TextureGraph {
            g: DiGraph::new(),
            results: HashMap::new(),
            cached: false
         }
    }

    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }

    pub fn add_node(&mut self, test_node: Node<T>) -> NodeIndex {
        self.cached = false;
        self.g.add_node(test_node)
    }

    fn insert_edge(&mut self, src: NodeIndex, dest: NodeIndex, target_input: usize) -> Result<(), String> {
        let new_edge = self.g.add_edge(src, dest, target_input);
        if is_cyclic_directed(&self.g) {
            self.g.remove_edge(new_edge);
            Err(String::from("Would have created cycle"))
        } else {
            Ok(())
        }
    }

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

    pub fn is_complete(&self) -> bool {
        self.g.node_indices().all(|i| {
            let node = &self.g[i];
            node.function.inputs() == self.g.neighbors_directed(i, Incoming).count()
        })
    }

    pub fn get_result(&self, index: &NodeIndex) -> Option<&T> {
        match self.cached {
            true => Some(&self.results[index]),
            false => None
        }
    }

    pub fn generate(&mut self) -> Result<&HashMap<NodeIndex, T>, String> {
        if self.cached {
            return Ok(&self.results)
        }
        if !self.is_complete() {
            //TODO: Change error type to Err<Vec<NodeIndex>>
            return Err(String::from("Graph not complete"))
        }
        self.results = HashMap::<NodeIndex, T>::new();
        let mut topo = Topo::new(&self.g);
        while let Some(i) = topo.next(&self.g) {
            let node = &self.g[i];
            println!("Traversing node {}", node.name);
            let mut arguments: Vec<_> = self.g.edges_directed(i, Incoming)
                .map(|e| {
                    let source = e.source();
                    let target = e.weight();
                    (*target, source)
                }).collect();
            arguments.sort_by_key(|(t, _)| *t);
            let generated_value = node.function.generate(
                arguments.iter().map(|(_, source)| &self.results[source]).collect()
            );
            self.results.insert(i, generated_value);
        }
        self.cached = true;
        Ok(&self.results)
    }
}

#[cfg(test)]
mod tests {
    use petgraph::{algo::is_cyclic_directed, dot::{Dot, Config}};

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
        fn generate(&self, inputs: Vec<&i32>) -> i32 {
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
    fn complete_graph() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let node4 = Node::new(String::from("N4"), Box::new(Double{}));
        // Empty graph is complete
        assert!(graph.is_complete());
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        // Const does not need inputs
        assert!(graph.is_complete());
        let index3 = graph.add_node(node3);
        // Add needs 2 inputs
        assert!(!graph.is_complete());
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        assert!(!graph.is_complete());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(graph.is_complete());
        // Double needs 1 input
        let index4 = graph.add_node(node4);
        assert!(!graph.is_complete());
        assert!(graph.add_edge(index3, index4, 0).is_ok());
        assert!(graph.is_complete());
    }

    #[test]
    fn simple_graph_generation() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let node3 = Node::new(String::from("N3"), Box::new(Add{}));
        let index1 = graph.add_node(node1);
        let index2 = graph.add_node(node2);
        let index3 = graph.add_node(node3);
        let node_count = graph.node_count();
        assert_eq!(3, node_count);
        assert!(graph.add_edge(index1, index3, 0).is_ok());
        // Incomplete graph cannot be generated
        assert!(graph.generate().is_err());
        assert!(graph.add_edge(index2, index3, 1).is_ok());
        assert!(!graph.cached);
        let gen_result = graph.generate();
        assert!(gen_result.is_ok());
        let result = gen_result.unwrap();
        assert_eq!(node_count, result.len());
        assert_eq!(1, result[&index1]);
        assert_eq!(2, result[&index2]);
        assert_eq!(3, result[&index3]);
        assert!(graph.cached);
    }

    #[test]
    fn uncache_node() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Const(2)));
        let _index1 = graph.add_node(node1);
        assert!(graph.generate().is_ok());
        assert!(graph.cached);
        let _index2 = graph.add_node(node2);
        assert!(!graph.cached);
        assert!(graph.generate().is_ok());
        assert!(graph.cached);
    }

    #[test]
    fn uncache_edge() {
        let mut graph = TextureGraph::<i32>::new();
        let node1 = Node::new(String::from("N1"), Box::new(Const(1)));
        let node2 = Node::new(String::from("N2"), Box::new(Double{}));
        let node3 = Node::new(String::from("N3"), Box::new(Const(2)));
        let node4 = Node::new(String::from("N4"), Box::new(Double{}));
        let indices = vec![
            graph.add_node(node1),
            graph.add_node(node2),
            graph.add_node(node3),
            graph.add_node(node4),
        ];
        assert!(graph.add_edge(indices[0], indices[1], 0).is_ok());
        assert!(graph.add_edge(indices[2], indices[3], 0).is_ok());
        assert!(graph.generate().is_ok());
        assert!(graph.cached);
        assert!(graph.add_edge(indices[0], indices[3], 0).is_ok());
        assert!(graph.add_edge(indices[2], indices[1], 0).is_ok());
        assert!(!graph.cached);
        assert!(graph.generate().is_ok());
        assert!(graph.cached);

    }

}