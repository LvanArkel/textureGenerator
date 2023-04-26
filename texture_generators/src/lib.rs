use graph::TextureTransformer;
use image::{Rgb, Rgb32FImage, ImageBuffer};
use interpolation::lerp;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

type Color = Rgb<f32>;

pub struct Gradient {
    pub start: Color,
    pub end: Color
}

fn lerp_gradient(gradient: &Gradient, fraction: f32) -> Color {
    Rgb(lerp(&gradient.start.0, &gradient.end.0, &fraction))
}

// Generating nodes
pub struct SolidColorNode {
    pub color: Color
}

pub enum GradientNodeDirection {
    HORIZONTAL, VERTICAL, RADIAL
}
pub struct GradientNode {
    pub gradient: Gradient,
    pub direction: GradientNodeDirection
}

pub struct CheckerboardNode {

}

pub struct LinesNode {

}

pub struct BrickNode {

}


// Generating nodes
impl TextureTransformer<Rgb32FImage> for SolidColorNode {
    fn generate(&self, _: Vec<&Rgb32FImage>) -> Rgb32FImage {
        ImageBuffer::from_pixel(WIDTH, HEIGHT, self.color)
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for GradientNode {
    fn generate(&self, _: Vec<&Rgb32FImage>) -> Rgb32FImage {
        match self.direction {
            GradientNodeDirection::HORIZONTAL => ImageBuffer::from_fn(WIDTH, HEIGHT, |x, _| {
                lerp_gradient(&self.gradient, x as f32 / WIDTH as f32)
            }),
            GradientNodeDirection::VERTICAL => ImageBuffer::from_fn(WIDTH, HEIGHT, |_, y| {
                lerp_gradient(&self.gradient, y as f32 / HEIGHT as f32)
            }),
            GradientNodeDirection::RADIAL => ImageBuffer::from_fn(WIDTH, HEIGHT, |x, y| {
                let u = x as f32 / WIDTH as f32 - 0.5;
                let v = y as f32 / HEIGHT as f32 - 0.5;
                let dist = (u*u+v*v).sqrt();
                lerp_gradient(&self.gradient, dist / 2.0_f32.sqrt())
            }),
        }
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for CheckerboardNode {
    fn generate(&self, inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        todo!()
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for LinesNode {
    fn generate(&self, inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        todo!()
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for BrickNode {
    fn generate(&self, inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        todo!()
    }

    fn inputs(&self) -> usize {
        0
    }
}


#[cfg(test)]
mod tests {
    use graph::TextureTransformer;
    use image::{Rgb, GenericImageView};

    use crate::{SolidColorNode, Gradient, GradientNode, GradientNodeDirection::*, HEIGHT, WIDTH};

    #[test]
    fn test_solid() {
        let color = Rgb([1.0, 0.0, 0.0]);
        let node = SolidColorNode{color: color};
        let image = node.generate(Vec::new());
        assert!(image.pixels().all(|&pix| pix == color));
    }

    #[test]
    fn test_gradient_solid() {
        let color = Rgb([1.0, 0.0, 0.0]);
        let gradient = Gradient{
            start: color,
            end: color
        };
        let node = GradientNode{gradient: gradient, direction: HORIZONTAL };
        let image = node.generate(Vec::new());
        assert!(image.pixels().all(|&pix| pix == color));
    }

    #[test]
    fn test_gradient_horizontal() {
        let gradient = Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, direction: HORIZONTAL };
        let image = node.generate(Vec::new());
        for y in 0..HEIGHT {
            for x in 0..WIDTH-1 {
                assert!(image.get_pixel(x, y).0[0] <= image.get_pixel(x+1, y).0[0]);
            }
        }
        for x in 0..WIDTH {
            let head = image.get_pixel(x, 0);
            for y in 0..HEIGHT {
                assert_eq!(head, image.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_gradient_vertical() {
        let gradient = Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, direction: VERTICAL };
        let image = node.generate(Vec::new());
        for y in 0..HEIGHT {
            let head = image.get_pixel(0, y);
            for x in 0..WIDTH {
                assert_eq!(head, image.get_pixel(x, y));
            }
        }
        for x in 0..WIDTH {
            for y in 0..HEIGHT-1 {
                assert!(image.get_pixel(x, y).0[0] <= image.get_pixel(x, y+1).0[0]);
            }
        }
    }

    #[test]
    fn test_gradient_radial() {
        let gradient = Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, direction: VERTICAL };
        let image = node.generate(Vec::new());
        for y in 0..HEIGHT {
            for x in 0..WIDTH/2 {
                assert!(image.get_pixel(x, y).0 <= image.get_pixel(x+1, y).0);
                assert!(image.get_pixel(WIDTH-1-x, y).0 >= image.get_pixel(WIDTH-2-x, y).0);
            }
        }
        for x in 0..WIDTH {
            for y in 0..HEIGHT/2 {
                assert!(image.get_pixel(x, y).0 <= image.get_pixel(x, y+1).0);
                assert!(image.get_pixel(x, HEIGHT-1-y).0 >= image.get_pixel(x, HEIGHT-2-y).0);
            }
        }
    }
}