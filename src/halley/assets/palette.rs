use std::{collections::HashMap, io::Cursor};

use image::{Rgba, Rgba32FImage};

pub struct Palette {
    swap_data: [i32; 256],
    swap_color_to_index_map: HashMap<i32, i32>,
}

impl Palette {
    fn new(image: &Rgba32FImage) -> Option<Self> {
        let mut palette = Palette {
            swap_data: [0; 256],
            swap_color_to_index_map: HashMap::new(),
        };

        let mut n = 1;
        for i in 0..255 {
            palette.swap_data[i] = swap_pixel(i, image);
            if palette.swap_data[i] == 0 {
                palette.swap_data[i] = -16711680 + n;
                n += 16;
            }
            if palette
                .swap_color_to_index_map
                .contains_key(&(palette.swap_data[i] as i32))
            {
                return None;
            }
            palette
                .swap_color_to_index_map
                .insert(palette.swap_data[i] as i32, i as i32);
        }
        Some(palette)
    }

    fn swap_image_palette(&self, image: &Rgba32FImage) -> Rgba32FImage {
        let mut new_image = image.clone();
        for (x, y, pixel) in image.enumerate_pixels() {
            let pixel = pixel_to_i32(&pixel);
            let color = (self.swap_data[pixel as usize] & 0xff0000) >> 16;

            new_image.put_pixel(x, y, pixel_from_i32(color));
        }
        new_image
    }

    fn unswap_image_palette(&self, image: &Rgba32FImage) -> Rgba32FImage {
        let mut new_image = image.clone();
        for (x, y, pixel) in image.enumerate_pixels() {
            let pixel = pixel_to_i32(&pixel);

            let color = self.swap_color_to_index_map.get(&pixel).unwrap();
            let color = (0xFF000000u32 as i32) | (color << 16) | (color << 8) | color;

            new_image.put_pixel(x, y, pixel_from_i32(color));
        }
        new_image
    }
}

fn swap_pixel(pos: usize, palette_image: &Rgba32FImage) -> i32 {
    let (w, h) = palette_image.dimensions();
    let f = pos as f32 / 255.0;
    let f2: f32 = if 0.0625 < f { 0.0 } else { 1.0 };
    let f3 = (1.0 - f2) * if 0.125 < f { 0.0 } else { 1.0 };
    let f4 = mix(mix(0.0, 0.0, f2), 0.0, f3);
    let x = (f * w as f32) as u32;
    let y = (f4 * h as f32) as u32;
    let pixel = palette_image.get_pixel(x, y);
    pixel_to_i32(&pixel)
}

fn pixel_to_i32(pixel: &Rgba<f32>) -> i32 {
    let [a, r, g, b] = pixel.0;
    ((a * 255.0) as i32) << 24
        | ((r * 255.0) as i32) << 16
        | ((g * 255.0) as i32) << 8
        | (b * 255.0) as i32
}

fn pixel_from_i32(pixel: i32) -> Rgba<f32> {
    let a = ((pixel >> 24) & 0xff) as f32 / 255.0;
    let r = ((pixel >> 16) & 0xff) as f32 / 255.0;
    let g = ((pixel >> 8) & 0xff) as f32 / 255.0;
    let b = (pixel & 0xff) as f32 / 255.0;
    Rgba([a, r, g, b])
}

fn mix(f: f32, f2: f32, f3: f32) -> f32 {
    f * (1.0 - f3) + f2 * f3
}

pub fn load_palette(i: &[u8]) -> Result<Palette, image::error::ImageError> {
    let img = image::io::Reader::new(Cursor::new(i))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba32f();

    Ok(Palette::new(&img).unwrap())
}
