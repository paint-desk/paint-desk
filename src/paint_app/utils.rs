use std::collections::hash_map::IntoIter;
use crate::paint_app::canvas_layer::CanvasLayer;
use super::data_types::*;

fn blend_color(top: Color, bottom: Color) -> Color {
    let combined_alpha = top.alpha as u16 + ((255 - top.alpha as u16) * bottom.alpha as u16 + 127) / 255;

    if combined_alpha == 0 {
        return Color { red: 0, green: 0, blue: 0, alpha: 0 };
    }

    Color {
        red: ((top.red as u16 * top.alpha as u16 + (255 - top.alpha as u16) * bottom.red as u16 * bottom.alpha as u16 / 255 + 127) / combined_alpha) as u8,
        green: ((top.green as u16 * top.alpha as u16 + (255 - top.alpha as u16) * bottom.green as u16 * bottom.alpha as u16 / 255 + 127) / combined_alpha) as u8,
        blue: ((top.blue as u16 * top.alpha as u16 + (255 - top.alpha as u16) * bottom.blue as u16 * bottom.alpha as u16 / 255 + 127) / combined_alpha) as u8,
        alpha: combined_alpha as u8,
    }
}

/// applies color_a over color_b
pub fn pixel_overlap2(color_a : Color, color_b : Color) -> Color {
    let color_a_f32 = glam::Vec4::new(color_a.red as f32, color_a.green as f32, color_a.blue as f32, color_a.alpha as f32) * (1.0 / 255.0);
    let color_b_f32 = glam::Vec4::new(color_b.red as f32, color_b.green as f32, color_b.blue as f32, color_b.alpha as f32) * (1.0 / 255.0);

    let alpha_a = color_a_f32.w;
    let mut alpha_b = color_b_f32.w;
    if (alpha_a + alpha_b) >= 1.0 {
        alpha_b = 1.0 - alpha_a;
    }

    let x = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).x;
    let y = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).y;
    let z = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).z;
    let w = (alpha_a + alpha_b) * 255.0;
    Color::new(
        x as u8,
        y as u8,
        z as u8,
        w as u8
    )
}
/// Applies color_a over color_b
pub fn pixel_overlap(color_a: Color, color_b: Color) -> Color {
    // Convert colors to integer representation
    let color_a_int = color_a;
    let color_b_int = color_b;

    // Calculate weighted sum without using floats
    let alpha_a = color_a_int.alpha as i32;
    let mut alpha_b = color_b_int.alpha as i32;
    if (alpha_a + alpha_b) >= 255 {
        alpha_b = 255 - alpha_a;
    }

    let x = ((color_a_int.red as i32 * alpha_a + color_b_int.red as i32 * alpha_b) / 255) as u8;
    let y = ((color_a_int.green as i32 * alpha_a + color_b_int.green as i32 * alpha_b) / 255) as u8;
    let z = ((color_a_int.blue as i32 * alpha_a + color_b_int.blue as i32 * alpha_b) / 255) as u8;
    let w = (alpha_a + alpha_b) as u8;

    Color::new(x, y, z, w)
}

//pub fn checkers_pattern(pixel_pos : PixelPos, grid_len : usize) -> Color
//{
//    let x = pixel_pos.x as usize;
//    let y = pixel_pos.y as usize;
//    if (x / grid_len) % 2 == 0 {
//        if (y / grid_len) % 2 == 0 {
//            Color::new(255, 255, 255, 255)
//        } else {
//            Color::new(0, 0, 0, 255)
//        }
//    } else if (y / grid_len) % 2 == 0 {
//        Color::new(0, 0, 0, 255)
//    } else {
//        Color::new(255, 255, 255, 255)
//    }
//}
pub fn checkers_pattern(pixel_pos : PixelPos, grid_len : usize, square_color_a: Color, square_color_b: Color) -> Color
{
    let x = pixel_pos.x as usize;
    let y = pixel_pos.y as usize;
    if (x / grid_len) % 2 == 0 {
        if (y / grid_len) % 2 == 0 {
            square_color_a
        } else {
            square_color_b
        }
    } else if (y / grid_len) % 2 == 0 {
        square_color_b
    } else {
        square_color_a
    }
}

// return iterable of pixelpos, dont return vec
pub fn rasterize_line(start : PixelPos, end : PixelPos) -> Vec<PixelPos> {
    let mut result = Vec::new();
    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let mut x1 = end.x as i32;
    let mut y1 = end.y as i32;

    let mut steep = false;
    if (x0-x1).abs() < (y0-y1).abs() {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    let dx = x1-x0;
    let dy = y1-y0;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = y0;
    for x in x0..x1+1 {
        if steep {
            result.push(PixelPos{x: y as u32, y: x as u32});
        } else {
            result.push(PixelPos{x: x as u32, y: y as u32});
        }
        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
    result

}

fn rasterize_rect(start : PixelPos, end : PixelPos) -> Vec<PixelPos> {
    let mut result = Vec::new();
    let mut x0 = start.x as i32;
    let mut y0 = start.y as i32;
    let mut x1 = end.x as i32;
    let mut y1 = end.y as i32;

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }
    for x in x0..x1+1 {
        for y in y0..y1+1 {
            result.push(PixelPos{x: x as u32, y: y as u32});
        }
    }
    result
}

pub fn draw_rect(canvas: &mut dyn CanvasLayer, start : PixelPos, end : PixelPos, color: Color) {
    let pixels = rasterize_rect(start, end);
    for pixel in pixels {
        canvas.set_pixel(pixel, color);
    }
}

// tests:

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_overlap() {
        //let color_a = Color::new(255, 0, 0, 255);
        //let color_b = Color::new(0, 255, 0, 255);
        //let result = pixel_overlap(color_a, color_b);
        //assert_eq!(result, Color::new(255, 0, 0, 255));

        let color_a = Color::new(255, 0, 0, 120);
        let color_b = Color::new(0, 255, 0, 255);
        let result = pixel_overlap(color_a, color_b);
        assert_eq!(result, Color::new(120, 135, 0, 255));
    }
}