use std::iter::zip;

use graph::TextureTransformer;
use image::{ImageBuffer, Rgb, RgbImage, Pixel, DynamicImage, Rgb32FImage};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

pub struct SolidColor {
    pub color: Rgb<f32>
}

impl TextureTransformer<Rgb32FImage> for SolidColor {
    fn generate(&self, _inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        Rgb32FImage::from_pixel(WIDTH, HEIGHT, self.color)
    }

    fn inputs(&self) -> usize {
        0
    }
}

pub enum BlendOptions {
    ADD,
    // SUBTRACT,
    // MULTIPLY,
}

pub struct Blend {
    pub operator: BlendOptions
}

impl TextureTransformer<Rgb32FImage> for Blend {
    fn generate(&self, inputs: Vec<&Rgb32FImage>) -> Rgb32FImage {
        let i1 = inputs[0];
        let i2 = inputs[1];
        let mut img: Rgb32FImage = ImageBuffer::new(WIDTH, HEIGHT);
        for ((x1, y1, pixel1), (_x2, _y2, pixel2)) in zip(i1.enumerate_pixels(), i2.enumerate_pixels()) {
            let pixel = match self.operator {
                BlendOptions::ADD => pixel1.map2(pixel2, |a, b| a + b),
                // BlendOptions::SUBTRACT => pixel1.map2(pixel2, |a, b| a - b),
                // BlendOptions::MULTIPLY => piz,
            };
            img.put_pixel(x1, y1, pixel);
        }
        img
    }

    fn inputs(&self) -> usize {
        2
    }
}