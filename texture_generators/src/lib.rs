use std::iter::zip;

use graph::TextureTransformer;
use image::{ImageBuffer, Rgb, RgbImage, Pixel};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;

pub struct SolidColor {
    pub color: Rgb<u8>
}

impl TextureTransformer<RgbImage> for SolidColor {
    fn generate(&self, _inputs: Vec<&RgbImage>) -> RgbImage {
        ImageBuffer::from_fn(WIDTH, HEIGHT, |_,_| -> Rgb<u8> {self.color})
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

impl TextureTransformer<RgbImage> for Blend {
    fn generate(&self, inputs: Vec<&RgbImage>) -> RgbImage {
        let i1 = inputs[0];
        let i2 = inputs[1];
        let mut img: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);
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