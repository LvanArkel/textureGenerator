use core::{WIDTH, HEIGHT, average_color};

use graph::TextureTransformer;
use image::{Rgb32FImage, Pixel, Rgb};

pub enum BlendOptions {
    Add,
    Subtract,
    Multiply,
    Mask(f32),
}

pub struct BlendNode {
    pub option: BlendOptions
}

impl TextureTransformer<Rgb32FImage> for BlendNode {
    fn generate(&self, inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let image1 = inputs[0];
        let image2 = inputs[1];
        Rgb32FImage::from_fn(WIDTH, HEIGHT, |x, y| {
            let pix1 = image1.get_pixel(x, y);
            let pix2 = image2.get_pixel(x, y);
            match self.option {
                BlendOptions::Add => pix1.map2(pix2, |a, b| (a + b).clamp(0.0, 1.0)),
                BlendOptions::Subtract => pix1.map2(pix2, |a, b| (a - b).clamp(0.0, 1.0)),
                BlendOptions::Multiply => pix1.map2(pix2, |a, b| (a * b).clamp(0.0, 1.0)),
                BlendOptions::Mask(threshold) => if average_color(pix1) >= threshold {*pix2} else {Rgb([0.0, 0.0, 0.0])}
            }
        })
    }

    fn inputs(&self) -> usize {
        2
    }
}

#[cfg(test)]
mod tests {
    use graph::TextureTransformer;
    use image::Rgb;
    use core::Gradient;

    use crate::{SolidColorNode, BlendNode, BlendOptions, GradientNode, GradientNodeDirection};

    #[test]
    fn test_blend_add() {
        let node1 = SolidColorNode{color: Rgb([1.0, 0.0, 0.2])};
        let node2 = SolidColorNode{color: Rgb([0.0, 1.0, 0.4])};
        let blend_node = BlendNode{option: BlendOptions::Add};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([1.0, 1.0, 0.6])))
    }

    #[test]
    fn test_blend_add_bounds() {
        let node1 = SolidColorNode{color: Rgb([1.0, 0.0, 0.2])};
        let node2 = SolidColorNode{color: Rgb([1.0, 1.0, 0.0])};
        let blend_node = BlendNode{option: BlendOptions::Add};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([1.0, 1.0, 0.2])))
    }

    #[test]
    fn test_blend_subtract() {
        let node1 = SolidColorNode{color: Rgb([1.0, 0.8, 0.2])};
        let node2 = SolidColorNode{color: Rgb([1.0, 0.4, 0.0])};
        let blend_node = BlendNode{option: BlendOptions::Subtract};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([0.0, 0.4, 0.2])))
    }
    
    #[test]
    fn test_blend_subtract_bounds() {
        let node1 = SolidColorNode{color: Rgb([0.0, 0.8, 0.2])};
        let node2 = SolidColorNode{color: Rgb([1.0, 0.4, 0.5])};
        let blend_node = BlendNode{option: BlendOptions::Subtract};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([0.0, 0.4, 0.0])))
    }

    #[test]
    fn test_blend_multiply() {
        let node1 = SolidColorNode{color: Rgb([0.0, 0.2, 0.4])};
        let node2 = SolidColorNode{color: Rgb([1.0, 2.0, 1.5])};
        let blend_node = BlendNode{option: BlendOptions::Multiply};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([0.0, 0.4, 0.6])))
    }

    #[test]
    fn test_blend_multiply_bounds() {
        let node1 = SolidColorNode{color: Rgb([0.0, 0.2, 0.4])};
        let node2 = SolidColorNode{color: Rgb([1.0, 2.0, 3.0])};
        let blend_node = BlendNode{option: BlendOptions::Multiply};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.pixels().all(|pix| *pix == Rgb([0.0, 0.4, 1.0])))

    }

    #[test]
    fn test_blend_mask() {
        let node1 = GradientNode{gradient: Gradient { start: Rgb([0.0, 0.0, 0.0]), end: Rgb([1.0, 1.0, 1.0]) }, direction: GradientNodeDirection::VERTICAL };
        let node2 = SolidColorNode{color: Rgb([1.0, 0.0, 0.0])};
        let blend_node = BlendNode{option: BlendOptions::Mask(0.4)};
        let image1 = node1.generate(vec![]);
        let image2 = node2.generate(vec![]);
        let targets = vec![&image1, &image2];
        let image = blend_node.generate(targets);
        assert!(image.enumerate_pixels().all(|(x, y, pix)| {
            let mask = image1.get_pixel(x, y).0;
            let threshold = (mask[0] + mask[1] + mask[2]) / 3.0;
            if threshold >= 0.4 {
                pix == &Rgb([1.0, 0.0, 0.0])
            } else {
                pix == &Rgb([0.0, 0.0, 0.0])
            }
        }))
    }

}