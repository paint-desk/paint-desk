use std::collections::hash_map::IntoIter;
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

pub fn pixel_overlap(color_a : Color, color_b : Color) -> Color {
    let color_a_f32 = glam::Vec4::new(color_a.red as f32, color_a.green as f32, color_a.blue as f32, color_a.alpha as f32) * (1.0 / 255.0);
    let color_b_f32 = glam::Vec4::new(color_b.red as f32, color_b.green as f32, color_b.blue as f32, color_b.alpha as f32) * (1.0 / 255.0);

    let alpha_a = color_a_f32.w;
    let mut alpha_b = color_b_f32.w;
    if (alpha_a + alpha_b) >= 1.0 {
        alpha_b = 1.0 - alpha_a;
    }
    // alphas
    print!("{} {}\n", alpha_a, alpha_b);

    let x = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).x;
    let y = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).y;
    let z = ((color_a_f32 * alpha_a + color_b_f32 * alpha_b) * 255.0).z;
    let w = (alpha_a + alpha_b) * 255.0;
    print!("{} {} {} {}\n", x, y, z, w);
    let result = Color::new(
        x as u8,
        y as u8,
        z as u8,
        w as u8
    );

    result
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
        assert_eq!(result, Color::new(120, 134, 0, 255));
    }
}