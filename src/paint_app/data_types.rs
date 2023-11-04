pub enum SideHorizontal {
    center,
    left,
    right
}

pub enum SideVertical {
    center,
    top,
    bottom
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: a
        }
    }

    // don use float
    pub fn interpolate(&self, other: &Color, t: u8) -> Color {
        let t = t as u16;
        let t_inv = 255 - t;
        Color {
            red: ((self.red as u16 * t + other.red as u16 * t_inv) / 255) as u8,
            green: ((self.green as u16 * t + other.green as u16 * t_inv) / 255) as u8,
            blue: ((self.blue as u16 * t + other.blue as u16 * t_inv) / 255) as u8,
            alpha: ((self.alpha as u16 * t + other.alpha as u16 * t_inv) / 255) as u8
        }
    }

    pub fn white() -> Color {
        Color::new(255, 255, 255, 255)
    }

    pub fn black() -> Color {
        Color::new(0, 0, 0, 255)
    }

    pub fn to(&self) -> u32 {
        unsafe {
            std::mem::transmute(*self)
        }
    }

    pub fn from(value: u32) -> Color {
        unsafe {
            std::mem::transmute(value)
        }
    }

    pub fn to_color32(&self) -> egui::ecolor::Color32 {
        egui::ecolor::Color32::from_rgba_unmultiplied(self.red, self.green, self.blue, self.alpha)
    }

    pub fn from_color32(color32: &egui::ecolor::Color32) -> Color {
        let tuple = color32.to_tuple();
        Color::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PixelPos {
    pub x: u32,
    pub y: u32
}

pub struct GlobalParams {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub cursor_in_canvas: bool,
    pub current_pixel: Option<PixelPos>
}

impl GlobalParams {
    pub fn new() -> GlobalParams {
        GlobalParams {
            primary_color: Color::new(0, 0, 0, 255),
            secondary_color: Color::new(255, 255, 255, 255),
            cursor_in_canvas: false,
            current_pixel: None
        }
    }
}