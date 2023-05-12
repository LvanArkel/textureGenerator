use core::Gradient;

use graph::{TextureGraph, Node, NodeIndex};
use image::{RgbImage, Rgb, Rgb32FImage, buffer::ConvertBuffer};
use texture_generators::{SolidColorNode, GradientNode, CheckerboardNode, LinesNode, LinesPosition, BlendNode};

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
        Node::new(String::from("LineStart"),
            Box::new(LinesNode {
                scale:1, 
                thickness: 0.4, 
                position: LinesPosition::Start, 
                color1: Rgb([0.0, 0.0, 0.0]), 
                color2: Rgb([1.0, 1.0, 1.0]) 
            })),
        Node::new(String::from("LineMid"),
            Box::new(LinesNode {
                scale:1, 
                thickness: 0.4, 
                position: LinesPosition::Middle, 
                color1: Rgb([0.0, 0.0, 0.0]), 
                color2: Rgb([1.0, 1.0, 1.0]) 
            })),
        Node::new(String::from("LineEnd"),
            Box::new(LinesNode {
                scale:1, 
                thickness: 0.4, 
                position: LinesPosition::End, 
                color1: Rgb([0.0, 0.0, 0.0]), 
                color2: Rgb([1.0, 1.0, 1.0]) 
            })),
        Node::new(String::from("Multiscale"),
            Box::new(LinesNode {
                scale:4, 
                thickness: 0.2, 
                position: LinesPosition::Start, 
                color1: Rgb([0.0, 0.0, 0.0]), 
                color2: Rgb([1.0, 1.0, 1.0]) 
            })),
        Node::new(String::from("BasicBlend"),
            Box::new(BlendNode {
                option: texture_generators::BlendOptions::Multiply
            })),
        Node::new(String::from("GrayGradient"),
            Box::new(GradientNode {
                gradient: Gradient{start: Rgb([0.0, 0.0, 0.0]), end: Rgb([1.0, 1.0, 1.0])},
                direction: texture_generators::GradientNodeDirection::HORIZONTAL
            })),
        Node::new(String::from("MaskBlend"),
            Box::new(BlendNode {
                option: texture_generators::BlendOptions::Mask(0.4)
            }))
    ];
    let (mut graph, indices) = create_graph(nodes);
    let edges = vec![
        (1, 12, 0), (4, 12, 1),
        (13, 14, 0), (0, 14, 1)
    ];
    add_edges(&mut graph, &indices, edges).unwrap();
    graph.generate_graph().unwrap();
    for index in indices {
        let result = graph.get_generated_node(&index).unwrap();
        let img: RgbImage = result.convert();
        img.save(format!("testimages/{}.png", graph.get_node(index).unwrap().name)).unwrap();
    }
}