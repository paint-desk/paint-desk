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

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PixelPos {
    pub x: u32,
    pub y: u32
}

pub struct GlobalParams {
    pub primary_color: Color,
    pub secondary_color: Color
}

impl GlobalParams {
    pub fn new() -> GlobalParams {
        GlobalParams {
            primary_color: Color::new(0, 0, 0, 255),
            secondary_color: Color::new(255, 255, 255, 255)
        }
    }
}