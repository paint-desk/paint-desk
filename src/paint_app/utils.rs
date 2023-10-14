use super::data_types::*;

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