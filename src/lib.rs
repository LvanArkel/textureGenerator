use image::Rgb;
use interpolation::lerp;

pub type Color = Rgb<f32>;

pub trait ImageOperation {
    fn apply_pixel(p1: Color, p2: Color) -> Color;
}

/// A linear gradient between 2 colors.
pub struct Gradient {
    pub start: Color,
    pub end: Color
}

impl Gradient {
    pub fn get_color(&self, fraction: f32) -> Color {
        Rgb(lerp(&self.start.0, &self.end.0, &fraction))
    }
}

pub fn average_color(color: &Color) -> f32 {
    let [r, g, b] = color.0;
    (r+g+b) / 3.0
}

pub trait TextureTransformer<T> {
    /// Generates the value of the nodes given its target inputs in the correct order.
    /// Function should assume inputs are valid
    fn generate(&self, inputs: Vec<&T>) -> T;
    /// Returns the amount of inputs this Transformer expects.
    fn inputs(&self) -> usize;
    /// Checks whether the inputs of the node conform. Can be used to see if images are the same size.
    fn is_valid(&self, _inputs: &Vec<&T>) -> bool {
        true
    }
}

mod app;
pub use app::NodeGraphExample;

mod nodes;
pub use crate::nodes::generators::GeneratorProperties;
pub use crate::nodes::generators::SolidColorNode;
pub use crate::nodes::generators::{GradientNode, GradientNodeDirection};
pub use crate::nodes::generators::CheckerboardNode;
pub use crate::nodes::generators::{LinesNode, LinesPosition};
pub use crate::nodes::transformers::{BlendNode, BlendOptions};
