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
    pub size_x: usize,
    pub size_y: usize,
    pub color1: Color,
    pub color2: Color
}

pub enum LinesPosition {
    Start, Middle, End
}

pub struct LinesNode {
    pub scale: usize,
    pub thickness: f32,
    pub position: LinesPosition,
    pub color1: Color,
    pub color2: Color
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
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let section_width = WIDTH / (self.size_x + 1) as u32;
        let section_height = HEIGHT / (self.size_y + 1) as u32;
        ImageBuffer::from_fn(WIDTH, HEIGHT, |x, y| {
            if ((x / section_width)%2) == ((y / section_height)%2) {
                self.color1
            } else {
                self.color2
            }
        })
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for LinesNode {
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let section_height = HEIGHT / self.scale as u32;
        
        Rgb32FImage::from_fn(WIDTH, HEIGHT, |_x, y| {
            let d_y = (y % section_height) as f32 / section_height as f32;
            match self.position {
                LinesPosition::Start => if d_y <= self.thickness {self.color2} else {self.color1},
                LinesPosition::Middle => if (0.5-d_y).abs() <= self.thickness / 2.0 {self.color2} else {self.color1},
                LinesPosition::End => if (1.0-d_y) <= self.thickness {self.color2} else {self.color1},
            }
        })
    }

    fn inputs(&self) -> usize {
        0
    }
}

impl TextureTransformer<Rgb32FImage> for BrickNode {
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
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

    use crate::{SolidColorNode, Gradient, GradientNode, GradientNodeDirection::*, HEIGHT, WIDTH, CheckerboardNode, LinesNode, LinesPosition::{*, self}};

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

    #[test]
    fn test_checkerboard_default() {
        let node = CheckerboardNode{
            size_x: 1,
            size_y: 1,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]),
        };
        let image = node.generate(Vec::new());
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if (x < WIDTH/2) == (y < HEIGHT/2) {
                    assert_eq!(image.get_pixel(x, y).0, node.color1.0);
                } else {
                    assert_eq!(image.get_pixel(x, y).0, node.color2.0)
                }
            }
        }
    }

    #[test]
    fn test_checkerboard_higher_scale() {
        let node = CheckerboardNode{
            size_x: 3,
            size_y: 3,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]),
        };
        let image = node.generate(Vec::new());
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if ((x / (WIDTH/(node.size_x+1) as u32))%2) == ((y / (HEIGHT/(node.size_x+1) as u32))%2) {
                    assert_eq!(image.get_pixel(x, y).0, node.color1.0);
                } else {
                    assert_eq!(image.get_pixel(x, y).0, node.color2.0)
                }
            }
        }
    }

    #[test]
    fn test_checkerboard_separate_scales() {
        let node = CheckerboardNode{
            size_x: 1,
            size_y: 3,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]),
        };
        let image = node.generate(Vec::new());
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if ((x / (WIDTH/(node.size_x+1) as u32))%2) == ((y / (HEIGHT/(node.size_y+1) as u32))%2) {
                    assert_eq!(image.get_pixel(x, y).0, node.color1.0);
                } else {
                    assert_eq!(image.get_pixel(x, y).0, node.color2.0)
                }
            }
        }
    }

    fn test_line_helper(scale: usize, thickness: f32, position: LinesPosition) {
        let color1 = Rgb([0.0, 0.0, 0.0]);
        let color2 = Rgb([1.0, 1.0, 1.0]);
        let node = LinesNode {
            scale, thickness, position, color1, color2
        };
        let image = node.generate(Vec::new());
        for y in 0..HEIGHT {
            let head = image.get_pixel(0, y);
            for x in 0..WIDTH {
                assert_eq!(head, image.get_pixel(x, y));
            }
        }
        let section_size = HEIGHT/scale as u32;
        for i in 0..scale {
            for y in 0..section_size {
                let d_y = y as f32 / section_size as f32;
                let color = match node.position {
                    Start => if d_y <= thickness {color2} else {color1},
                    Middle => if (0.5-d_y).abs() <= thickness/2.0 {color2} else {color1},
                    End => if (1.0-d_y) <= thickness {color2} else {color1},
                };
                assert_eq!(&color, image.get_pixel(0, section_size*i as u32+y));
            }
        }
    }

    #[test]
    fn test_lines_single_scale() {
        test_line_helper(2, 0.5, Start);
    }

    #[test]
    fn test_lines_thin_line() {
        test_line_helper(2, 0.1, Start);
    }

    #[test]
    fn test_lines_middle() {
        test_line_helper(2, 0.5, Middle);
    }

    #[test]
    fn test_lines_end() {
        test_line_helper(2, 0.5, End);
    }
}