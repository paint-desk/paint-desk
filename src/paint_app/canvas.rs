use std::collections::HashMap;
use super::data_types::*;


pub trait Canvas{
    fn get_pixel(&self, pixel_pos: PixelPos) -> Color;
    fn set_pixel(&mut self, pixel_pos: PixelPos, color: Color);
    fn apply_to_canvas(&self, target_canvas: &mut dyn Canvas);
    fn clear(&mut self);
}

pub struct FlatCanvas {
    width: u32,
    height: u32,
    data: Vec<Color>
}
impl FlatCanvas {
    pub fn new(w: u32, h: u32) -> FlatCanvas {
        FlatCanvas {
            width: w,
            height: h,
            data: vec!(Color::new(0, 0, 0, 0); (w * h) as usize)
        }
    }
}

impl Canvas for FlatCanvas {
    fn get_pixel(&self, pixel_pos: PixelPos) -> Color {
        self.data[(pixel_pos.x + pixel_pos.y * self.width) as usize]
    }

    fn set_pixel(&mut self, pixel_pos: PixelPos, color: Color) {
        self.data[(pixel_pos.x + pixel_pos.y * self.width) as usize] = color;
    }

    fn apply_to_canvas(&self, target_canvas: &mut dyn Canvas) {
        for x in 0..self.width {
            for y in 0..self.height {
                target_canvas.set_pixel(PixelPos{x, y}, self.get_pixel(PixelPos{x, y}));
            }
        }
    }

    fn clear(&mut self) {
        for i in 0..self.width as usize * self.height as usize {
            self.data[i] = Color::new(0, 0, 0, 0);
        }
    }

}

pub struct HashMapCanvas {
    width: u32,
    height: u32,
    data: HashMap<PixelPos, Color>
}

impl HashMapCanvas {
    pub fn new(w: u32, h: u32) -> HashMapCanvas {
        HashMapCanvas {
            width: w,
            height: h,
            data: HashMap::new()
        }
    }
}

impl Canvas for HashMapCanvas {
    fn get_pixel(&self, pixel_pos: PixelPos) -> Color {
        self.data.get(&pixel_pos).unwrap_or(&Color::new(0, 0, 0, 0)).clone()
    }

    fn set_pixel(&mut self, pixel_pos: PixelPos, color: Color) {
        self.data.insert(pixel_pos, color);
    }

    fn apply_to_canvas(&self, target_canvas: &mut dyn Canvas) {
        self.data.iter().for_each(|(pos, color)| {
            target_canvas.set_pixel(*pos, *color);
        });
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}