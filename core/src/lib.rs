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
