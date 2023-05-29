use tex_core::{Color, Gradient};
use tex_core::TextureTransformer;

use image::{Rgb32FImage, ImageBuffer};

pub struct GeneratorProperties {
    pub width: u32,
    pub height: u32
}

impl GeneratorProperties {
    pub fn default() -> Self {
        GeneratorProperties { width: 128, height: 128 }
    }
}

/// A node that generates a solid color.
pub struct SolidColorNode {
    pub color: Color,
    pub properties: GeneratorProperties
}

impl TextureTransformer<Rgb32FImage> for SolidColorNode {
    fn generate(&self, _: Vec<&Rgb32FImage>) -> Rgb32FImage {
        ImageBuffer::from_pixel(self.properties.width, self.properties.height, self.color)
    }

    fn inputs(&self) -> usize {
        0
    }
}

/// The direction that a gradient moves to
pub enum GradientNodeDirection {
    /// The gradient will move from left to right, and is constant in the vertical direction
    HORIZONTAL,
    /// The gradient will move from top to bottom, and is constant in the vertical direction
    VERTICAL,
    /// The gradient is circular with the start color in the center of the screen, and moves to the end color in the corners of the image
    RADIAL
}

/// A node that produces a smooth gradient in a specified direction
pub struct GradientNode {
    pub gradient: Gradient,
    pub direction: GradientNodeDirection,
    pub properties: GeneratorProperties,
}

impl TextureTransformer<Rgb32FImage> for GradientNode {
    fn generate(&self, _: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let width = self.properties.width;
        let height = self.properties.height;
        match self.direction {
            GradientNodeDirection::HORIZONTAL => ImageBuffer::from_fn(width, height, |x, _| {
                self.gradient.get_color(x as f32 / width as f32)
            }),
            GradientNodeDirection::VERTICAL => ImageBuffer::from_fn(width, height, |_, y| {
                self.gradient.get_color(y as f32 / width as f32)
            }),
            GradientNodeDirection::RADIAL => ImageBuffer::from_fn(width, height, |x, y| {
                let u = x as f32 / width as f32 - 0.5;
                let v = y as f32 / height as f32 - 0.5;
                let dist = (u*u+v*v).sqrt();
                self.gradient.get_color(dist / 2.0_f32.sqrt())
            }),
        }
    }

    fn inputs(&self) -> usize {
        0
    }
}

/// A node that produces a checkerboard pattern.
pub struct CheckerboardNode {
    /// The amount of tiles in the horizontal direction. The amount of tiles in the image is x+1
    pub size_x: usize,
    /// The amount of tiles in the vertical direction. The amount of tiles in the image is y+1
    pub size_y: usize,
    /// The color of the tiles starting in the top-left corner
    pub color1: Color,
    /// The color of the tiles not starting in the top-left corner.
    pub color2: Color,
    pub properties: GeneratorProperties,
}

impl TextureTransformer<Rgb32FImage> for CheckerboardNode {
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let width = self.properties.width;
        let height = self.properties.height;
        let section_width = width / (self.size_x + 1) as u32;
        let section_height = height / (self.size_y + 1) as u32;
        ImageBuffer::from_fn(width, height, |x, y| {
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

/// Generates a pattern of lines on a colored background
pub enum LinesPosition {
    Start, Middle, End
}

pub struct LinesNode {
    /// The amount of lines on an image
    pub scale: usize,
    /// The proportion of line/background
    pub thickness: f32, 
    /// The position of the line on the background
    pub position: LinesPosition,
    /// The color of the background
    pub color1: Color,
    /// The color of the line 
    pub color2: Color,
    pub properties: GeneratorProperties,
}

impl TextureTransformer<Rgb32FImage> for LinesNode {
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let width = self.properties.width;
        let height = self.properties.height;
        let section_height = height / self.scale as u32;
        
        Rgb32FImage::from_fn(width, height, |_x, y| {
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

#[cfg(test)]
pub mod tests {
    use tex_core::Gradient;
    use tex_core::TextureTransformer;
    
    use image::Rgb;

    use crate::{nodes::generators::{SolidColorNode, GeneratorProperties}, CheckerboardNode, LinesPosition, LinesNode, GradientNodeDirection, GradientNode};

    #[test]
    fn test_solid() {
        let color = Rgb([1.0, 0.0, 0.0]);
        let node = SolidColorNode{color, properties: GeneratorProperties::default()};
        let image = node.generate(Vec::new());
        assert!(image.pixels().all(|&pix| pix == color));
    }

    #[test]
    fn test_solid_properties() {
        let color = Rgb([1.0, 0.0, 0.0]);
        let node = SolidColorNode{color, properties: GeneratorProperties { width: 128, height: 128 }};
        let image = node.generate(Vec::new());
        assert_eq!(128, image.width());
        assert_eq!(128, image.height());
        let node = SolidColorNode{color, properties: GeneratorProperties { width: 64, height: 64 }};
        let image = node.generate(Vec::new());
        assert_eq!(64, image.width());
        assert_eq!(64, image.height());
    }
    
    #[test]
    fn test_gradient_solid() {
        let color = Rgb([1.0, 0.0, 0.0]);
        let gradient = Gradient{
            start: color,
            end: color
        };
        let node = GradientNode{gradient: gradient, direction: GradientNodeDirection::HORIZONTAL, properties: GeneratorProperties::default() };
        let image = node.generate(Vec::new());
        assert!(image.pixels().all(|&pix| pix == color));
    }
    
    #[test]
    fn test_gradient_horizontal() {
        let gradient = Gradient{
            start: Rgb([0.0, 0.0, 0.0]),
            end: Rgb([1.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, direction: GradientNodeDirection::HORIZONTAL, properties: GeneratorProperties::default() };
        let image = node.generate(Vec::new());
        for y in 0..image.height() {
            for x in 0..image.width()-1 {
                assert!(image.get_pixel(x, y).0[0] <= image.get_pixel(x+1, y).0[0]);
            }
        }
        for x in 0..image.width() {
            let head = image.get_pixel(x, 0);
            for y in 0..image.height() {
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
        let node = GradientNode{gradient: gradient, direction: GradientNodeDirection::VERTICAL, properties: GeneratorProperties::default() };
        let image = node.generate(Vec::new());
        for y in 0..image.height() {
            let head = image.get_pixel(0, y);
            for x in 0..image.width() {
                assert_eq!(head, image.get_pixel(x, y));
            }
        }
        for x in 0..image.width() {
            for y in 0..image.height()-1 {
                assert!(image.get_pixel(x, y).0[0] <= image.get_pixel(x, y+1).0[0]);
            }
        }
    }

    #[test]
    fn test_gradient_radial() {
        let gradient = Gradient{
            start: Rgb([1.0, 0.0, 0.0]),
            end: Rgb([0.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, direction: GradientNodeDirection::RADIAL, properties: GeneratorProperties::default() };
        let image = node.generate(Vec::new());
        for y in 0..image.height() {
            for x in 0..image.width()/2-1 {
                assert!(image.get_pixel(x, y).0 <= image.get_pixel(x+1, y).0);
                assert!(image.get_pixel(image.width()-1-x, y).0 <= image.get_pixel(image.width()-2-x, y).0);
            }
        }
        for x in 0..image.width() {
            for y in 0..image.height()/2-1 {
                assert!(image.get_pixel(x, y).0 <= image.get_pixel(x, y+1).0);
                assert!(image.get_pixel(x, image.height()-1-y).0 <= image.get_pixel(x, image.height()-2-y).0);
            }
        }
    }
    
    #[test]
    fn test_gradient_properties() {
        let gradient = Gradient{
            start: Rgb([1.0, 0.0, 0.0]),
            end: Rgb([0.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, 
            direction: GradientNodeDirection::RADIAL, 
            properties: GeneratorProperties { width: 128, height: 128 } };
        let image = node.generate(Vec::new());
        assert_eq!(128, image.width());
        assert_eq!(128, image.height());
        let gradient = Gradient{
            start: Rgb([1.0, 0.0, 0.0]),
            end: Rgb([0.0, 0.0, 0.0])
        };
        let node = GradientNode{gradient: gradient, 
            direction: GradientNodeDirection::RADIAL, 
            properties: GeneratorProperties { width: 64, height: 64 } };
        let image = node.generate(Vec::new());
        assert_eq!(64, image.width());
        assert_eq!(64, image.height());
    }
    
    #[test]
    fn test_checkerboard_default() {
        let node = CheckerboardNode{
            size_x: 1,
            size_y: 1,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]), 
            properties: GeneratorProperties::default(),
        };
        let image = node.generate(Vec::new());
        for x in 0..image.width() {
            for y in 0..image.height() {
                if (x < image.width()/2) == (y < image.height()/2) {
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
            properties: GeneratorProperties::default(),
        };
        let image = node.generate(Vec::new());
        for x in 0..image.width() {
            for y in 0..image.height() {
                if ((x / (image.width()/(node.size_x+1) as u32))%2) == ((y / (image.height()/(node.size_x+1) as u32))%2) {
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
            properties: GeneratorProperties::default(),
        };
        let image = node.generate(Vec::new());
        for x in 0..image.width() {
            for y in 0..image.height() {
                if ((x / (image.width()/(node.size_x+1) as u32))%2) == ((y / (image.height()/(node.size_y+1) as u32))%2) {
                    assert_eq!(image.get_pixel(x, y).0, node.color1.0);
                } else {
                    assert_eq!(image.get_pixel(x, y).0, node.color2.0)
                }
            }
        }
    }
        
    #[test]
    fn test_checkerboard_properties() {
        let node = CheckerboardNode{
            size_x: 1,
            size_y: 3,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]),
            properties: GeneratorProperties { width: 128, height: 128 }
        };
        let image = node.generate(Vec::new());
        assert_eq!(128, image.width());
        assert_eq!(128, image.height());
        let node = CheckerboardNode{
            size_x: 1,
            size_y: 3,
            color1: Rgb([0.0, 0.0, 0.0]),
            color2: Rgb([1.0, 1.0, 1.0]),
            properties: GeneratorProperties { width: 64, height: 64 }
        };
        let image = node.generate(Vec::new());
        assert_eq!(64, image.width());
        assert_eq!(64, image.height());
    }

    fn test_line_helper(scale: usize, thickness: f32, position: LinesPosition) {
        let color1 = Rgb([0.0, 0.0, 0.0]);
        let color2 = Rgb([1.0, 1.0, 1.0]);
        let node = LinesNode {
            scale, thickness, position, color1, color2, properties: GeneratorProperties::default()
        };
        let image = node.generate(Vec::new());
        for y in 0..image.height() {
            let head = image.get_pixel(0, y);
            for x in 0..image.width() {
                assert_eq!(head, image.get_pixel(x, y));
            }
        }
        let section_size = image.height()/scale as u32;
        for i in 0..scale {
            for y in 0..section_size {
                let d_y = y as f32 / section_size as f32;
                let color = match node.position {
                    LinesPosition::Start => if d_y <= thickness {color2} else {color1},
                    LinesPosition::Middle => if (0.5-d_y).abs() <= thickness/2.0 {color2} else {color1},
                    LinesPosition::End => if (1.0-d_y) <= thickness {color2} else {color1},
                };
                assert_eq!(&color, image.get_pixel(0, section_size*i as u32+y));
            }
        }
    }

    #[test]
    fn test_lines_single_scale() {
        test_line_helper(2, 0.5, LinesPosition::Start);
    }

    #[test]
    fn test_lines_thin_line() {
        test_line_helper(2, 0.1, LinesPosition::Start);
    }

    #[test]
    fn test_lines_middle() {
        test_line_helper(2, 0.5, LinesPosition::Middle);
    }

    #[test]
    fn test_lines_end() {
        test_line_helper(2, 0.5, LinesPosition::End);
    }
            
    #[test]
    fn test_lines_properties() {
        let color1 = Rgb([0.0, 0.0, 0.0]);
        let color2 = Rgb([1.0, 1.0, 1.0]);
        let node = LinesNode {
            scale: 2, thickness: 0.5, position: LinesPosition::Start, color1, color2, 
            properties: GeneratorProperties { width: 128, height: 128 }
        };
        let image = node.generate(Vec::new());
        assert_eq!(128, image.width());
        assert_eq!(128, image.height());
        let color1 = Rgb([0.0, 0.0, 0.0]);
        let color2 = Rgb([1.0, 1.0, 1.0]);
        let node = LinesNode {
            scale: 2, thickness: 0.5, position: LinesPosition::Start, color1, color2, 
            properties: GeneratorProperties { width: 64, height: 64 }
        };
        let image = node.generate(Vec::new());
        assert_eq!(64, image.width());
        assert_eq!(64, image.height());
    }
}