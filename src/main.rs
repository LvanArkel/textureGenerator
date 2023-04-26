use std::iter::zip;

use graph::{TextureGraph, Node};
use image::{RgbImage, Rgb, Rgb32FImage, DynamicImage, buffer::ConvertBuffer};
use texture_generators::{SolidColorNode, GradientNode, Gradient};

fn main() -> () {
    let mut graph = TextureGraph::<Rgb32FImage>::new();
    let node1 = Node::new(String::from("Solid"), Box::new(SolidColorNode{color: Rgb([1.0, 0.0, 0.0])}));
    let node2 = Node::new(String::from("GradientHorizontal"), 
        Box::new(GradientNode{gradient:Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        }, direction: texture_generators::GradientNodeDirection::HORIZONTAL }));
    let node3 = Node::new(String::from("GradientVertical"), 
        Box::new(GradientNode{gradient:Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        }, direction: texture_generators::GradientNodeDirection::VERTICAL }));
    let node4 = Node::new(String::from("GradientRadial"), 
        Box::new(GradientNode{gradient:Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        }, direction: texture_generators::GradientNodeDirection::RADIAL }));
    let index1 = graph.add_node(node1);
    let index2 = graph.add_node(node2);
    let index3 = graph.add_node(node3);
    let index4 = graph.add_node(node4);
    graph.generate_graph().unwrap();
    let indices = vec![index1, index2, index3, index4];
    for index in indices {
        let result = graph.get_generated_node(&index).unwrap();
        let img: RgbImage = result.convert();
        img.save(format!("{}.png", graph.get_node(index).name)).unwrap();
    }
    // graph.add_edge(index1, index3, 0).unwrap();
    // graph.add_edge(index2, index3, 1).unwrap();
    // graph.generate_graph().unwrap();
    // let end = graph.get_generated_node(&index3).unwrap();
    // let img: RgbImage = end.convert();
    // img.save("test.png").unwrap();
}