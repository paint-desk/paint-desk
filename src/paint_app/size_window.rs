use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{Button, Color32, ColorImage, PointerButton, Pos2, Rect, Sense, Vec2, menu, WidgetText, Window};
use crate::paint_app::canvas::Canvas;

pub struct SizeWindow {
    pub open: bool
}

impl SizeWindow {
    pub fn new() -> SizeWindow {
        SizeWindow {
            open: false
        }
    }

    pub fn show_size_window(&mut self, ctx: &egui::Context, canvas: &mut Canvas) {
    let mut egui_window = egui::Window::new("Size").resizable(false);
    egui_window = egui_window.open(&mut self.open);
    egui_window.show(ctx, |ui| {
        ui.label(format!("Size: {}x{}", canvas.get_size().0, canvas.get_size().1));
    });
}   
}