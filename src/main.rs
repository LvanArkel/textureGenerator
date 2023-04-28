use graph::{TextureGraph, Node, NodeIndex};
use image::{RgbImage, Rgb, Rgb32FImage, buffer::ConvertBuffer};
use texture_generators::{SolidColorNode, GradientNode, Gradient, CheckerboardNode};

fn create_graph(nodes: Vec<Node<Rgb32FImage>>) -> (TextureGraph<Rgb32FImage>, Vec<NodeIndex>) {
    let mut graph = TextureGraph::<Rgb32FImage>::new();
    let mut indices = Vec::new();
    for node in nodes {
        indices.push(graph.add_node(node));
    }
    (graph, indices)
}

fn add_edges(graph: &mut TextureGraph<Rgb32FImage>, indices: &Vec<NodeIndex>, edge_indices: Vec<(usize, usize, usize)>) -> Result<(), String> {
    edge_indices.iter().map(|(a, b, target)| (indices[*a], indices[*b], target))
        .map(|(e1, e2, target)| graph.add_edge(e1, e2, *target)).collect()
    
}

fn main() -> () {
    let nodes = vec![
        Node::new(String::from("Solid"), Box::new(SolidColorNode{color: Rgb([1.0, 0.0, 0.0])})),
        Node::new(String::from("GradientHorizontal"), 
            Box::new(GradientNode{gradient:Gradient{
                start: Rgb([0.0, 0.0, 0.0]),
                end: Rgb([0.0, 1.0, 0.0])
            }, direction: texture_generators::GradientNodeDirection::HORIZONTAL })),
        Node::new(String::from("GradientVertical"), 
            Box::new(GradientNode{gradient:Gradient{
                start: Rgb([0.0, 0.0, 0.0]),
                end: Rgb([1.0, 0.0, 0.0])
            }, direction: texture_generators::GradientNodeDirection::VERTICAL })),
        Node::new(String::from("GradientRadial"), 
            Box::new(GradientNode{gradient:Gradient{
                start: Rgb([0.0, 0.0, 0.0]),
                end: Rgb([1.0, 0.0, 0.0])
            }, direction: texture_generators::GradientNodeDirection::RADIAL })),
        Node::new(String::from("CheckerboardS1"),
            Box::new(CheckerboardNode{
                size_x: 1,
                size_y: 1,
                color1: Rgb([0.0, 0.0, 0.0]),
                color2: Rgb([1.0, 1.0, 1.0]),
            })),
        Node::new(String::from("CheckerboardS2"),
            Box::new(CheckerboardNode{
                size_x: 2,
                size_y: 2,
                color1: Rgb([0.0, 0.0, 0.0]),
                color2: Rgb([0.0, 1.0, 1.0]),
            })),
        Node::new(String::from("CheckerboardS3"),
            Box::new(CheckerboardNode{
                size_x: 3,
                size_y: 3,
                color1: Rgb([0.0, 0.0, 0.0]),
                color2: Rgb([1.0, 0.0, 0.0]),
            })),
        Node::new(String::from("CheckerboardS1-3"),
            Box::new(CheckerboardNode{
                size_x: 1,
                size_y: 3,
                color1: Rgb([0.0, 0.0, 0.0]),
                color2: Rgb([0.0, 1.0, 0.0]),
            })),
    ];
    let (mut graph, indices) = create_graph(nodes);
    let edges = vec![
        
    ];
    add_edges(&mut graph, &indices, edges).unwrap();
    graph.generate_graph().unwrap();
    for index in indices {
        let result = graph.get_generated_node(&index).unwrap();
        let img: RgbImage = result.convert();
        img.save(format!("testimages/{}.png", graph.get_node(index).name)).unwrap();
    }
    // graph.add_edge(index1, index3, 0).unwrap();
    // graph.add_edge(index2, index3, 1).unwrap();
    // graph.generate_graph().unwrap();
    // let end = graph.get_generated_node(&index3).unwrap();
    // let img: RgbImage = end.convert();
    // img.save("test.png").unwrap();
}