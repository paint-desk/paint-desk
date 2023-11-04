use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{Button, Color32, ColorImage, PointerButton, Pos2, Rect, Sense, Vec2, menu, WidgetText, Window, DragValue};
use crate::paint_app::canvas::Canvas;

use super::data_types::*;

pub struct SizeWindow {
    pub open: bool,
    pub width: u32,
    pub height: u32,
    pub keep_horizontal: SideHorizontal,
    pub keep_vertical: SideVertical,
}

impl SizeWindow {
    pub fn new() -> SizeWindow {
        SizeWindow {
            open: false,
            width: 0,
            height: 0,
            keep_horizontal: SideHorizontal::center,
            keep_vertical: SideVertical::center,
        }
    }

    pub fn show_size_window(&mut self, ctx: &egui::Context) {
    let mut egui_window = egui::Window::new("Size")
    .resizable(false)
    .collapsible(false);
    egui_window = egui_window.open(&mut self.open);
    egui_window.show(ctx, |ui| {
        ui.heading("Anchor");
        
        ui.horizontal(|ui| {
            ui.label("Width: ");
            ui.add(DragValue::new(&mut self.width).speed(1.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("Height: ");
            ui.add(DragValue::new(&mut self.height).speed(1.0));
        });

        ui.separator();

        ui.heading("Anchor");
        ui.horizontal(|ui| {
            //Top

            if ui.radio(self.keep_vertical == SideVertical::top && self.keep_horizontal == SideHorizontal::left, "").clicked() {
                self.keep_vertical = SideVertical::top;
                self.keep_horizontal = SideHorizontal::left;
            }
            if ui.radio(self.keep_vertical == SideVertical::top && self.keep_horizontal == SideHorizontal::center, "").clicked() {
                self.keep_vertical = SideVertical::top;
                self.keep_horizontal = SideHorizontal::center;
            }
            if ui.radio(self.keep_vertical == SideVertical::top && self.keep_horizontal == SideHorizontal::right, "").clicked() {
                self.keep_vertical = SideVertical::top;
                self.keep_horizontal = SideHorizontal::right;
            
            }
        });
        ui.horizontal(|ui| {
            //Middle

            if ui.radio(self.keep_vertical == SideVertical::center && self.keep_horizontal == SideHorizontal::left, "").clicked() {
                self.keep_vertical = SideVertical::center;
                self.keep_horizontal = SideHorizontal::left;
            }
            if ui.radio(self.keep_vertical == SideVertical::center && self.keep_horizontal == SideHorizontal::center, "").clicked() {
                self.keep_vertical = SideVertical::center;
                self.keep_horizontal = SideHorizontal::center;
            }
            if ui.radio(self.keep_vertical == SideVertical::center && self.keep_horizontal == SideHorizontal::right, "").clicked() {
                self.keep_vertical = SideVertical::center;
                self.keep_horizontal = SideHorizontal::right;
            }
        });
        ui.horizontal(|ui| {
            //Bottom

            if ui.radio(self.keep_vertical == SideVertical::bottom && self.keep_horizontal == SideHorizontal::left, "").clicked() {
                self.keep_vertical = SideVertical::bottom;
                self.keep_horizontal = SideHorizontal::left;
            }
            if ui.radio(self.keep_vertical == SideVertical::bottom && self.keep_horizontal == SideHorizontal::center, "").clicked() {
                self.keep_vertical = SideVertical::bottom;
                self.keep_horizontal = SideHorizontal::center;
            }
            if ui.radio(self.keep_vertical == SideVertical::bottom && self.keep_horizontal == SideHorizontal::right, "").clicked() {
                self.keep_vertical = SideVertical::bottom;
                self.keep_horizontal = SideHorizontal::right;
            
            }
        });
        
        ui.separator();


        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button("OK").clicked() {
            }
            if ui.button("Cancel").clicked() {
                
            }
        });
    });
}   
}