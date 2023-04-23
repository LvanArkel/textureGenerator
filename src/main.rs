use graph::{TextureGraph, Node};
use image::{RgbImage, Rgb};
use texture_generators::{SolidColor, Blend, BlendOptions};

fn main() -> () {
    let mut graph = TextureGraph::<RgbImage>::new();
    let node1 = Node::new(String::from("Solid red"), Box::new(SolidColor{color: Rgb([255, 0, 0])}));
    let node2 = Node::new(String::from("Solid Green"), Box::new(SolidColor{color: Rgb([0, 255, 0])}));
    let node3 = Node::new(String::from("Blend additive"), Box::new(Blend{operator: BlendOptions::ADD}));
    let index1 = graph.add_node(node1);
    let index2 = graph.add_node(node2);
    let index3 = graph.add_node(node3);
    graph.add_edge(index1, index3, 0).unwrap();
    graph.add_edge(index2, index3, 1).unwrap();
    graph.generate().unwrap();
    let result = graph.get_result(&index3);
    let img = result.unwrap();
    img.save("test.png");
}