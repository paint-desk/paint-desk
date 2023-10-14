use std::collections::HashMap;
use std::time::Instant;
use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{ColorImage, PointerButton, Pos2, Rect, Sense, Vec2};
use crate::paint_app::app::{Canvas, PaintTool, PixelPencil};
use crate::paint_app::canvas::{CanvasLayer, FlatCanvasLayer};
use crate::paint_app::data_types::*;

mod paint_app;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Paint Desk", native_options, Box::new(|cc| Box::new(AppContext::new(cc, 1024, 768))));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(AppContext::new(cc, 1024, 768))),
            )
            .await
            .expect("failed to start eframe");
    });
}


//#[derive(Default)]
struct AppContext {
    start_time: f32,
    frame_times: Vec<f32>,

    canvas: FlatCanvasLayer,
    paint_tools: HashMap<u32, Box<dyn PaintTool>>,
    selected_paint_tool: u32
}

impl AppContext {
    fn new(cc: &eframe::CreationContext<'_>, w:u32, h:u32) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut app = AppContext {
            start_time: 0f32,
            frame_times: Vec::new(),

            canvas: FlatCanvasLayer::new(w, h),
            paint_tools: HashMap::new(),
            selected_paint_tool: 0
        };
        app.paint_tools.insert(1, Box::new(PixelPencil::new(Color::new(255, 0, 0, 255), 1)));
        app
    }


    fn fill(&mut self)
    {
        let size = self.canvas.get_size();
        for x in 0..size.0 {
            for y in 0..size.1 {
                self.canvas.set_pixel(PixelPos {x, y}, Color::new(255, 255, 0, 255));
            }
        }
    }

    fn get_fps(&mut self) -> f32 {
        //let now = Instant::now();
        let delta_time = 1f32;//now.duration_since(self.start_time).as_secs_f32();
        self.start_time = 0f32;//now;
        // Record frame time and calculate average FPS
        self.frame_times.push(1.0 / delta_time);
        if self.frame_times.len() > 10 {
            self.frame_times.remove(0);
        }
        let avg_fps = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        avg_fps
    }
}

impl eframe::App for AppContext {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");

            let fps = 0f32;//self.get_fps();
            ui.label(format!("FPS: {:.2}", fps));

            let size = self.canvas.get_size();

            //let slice: &[Color] = &self.data;
            let slice: &[Color] = &self.canvas.get_data();

            // Unsafe conversion from &[u32] to &[u8]
            let byte_slice: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    slice.as_ptr() as *const u8,
                    slice.len() * std::mem::size_of::<Color>(),
                )
            };

            let image = ColorImage::from_rgba_premultiplied([size.0 as usize, size.1 as usize], &byte_slice);

            let texture = ui.ctx().load_texture("aa", image, TextureOptions::NEAREST);


            let mut middle_button = false;
            let mut primary_button = false;
            let mut origin = Pos2::new(0f32, 0f32);
            let mut current = Pos2::new(0f32, 0f32);
            ctx.input(| s | {
                middle_button = s.pointer.button_down(PointerButton::Middle);
                primary_button = s.pointer.button_down(PointerButton::Primary);
                origin = s.pointer.press_origin().unwrap_or_default();
                current = s.pointer.latest_pos().unwrap_or_default();

            });


            let mut drawing = false;
            egui::ScrollArea::both().drag_to_scroll(middle_button).show(ui, |ui| {
                let rect = ui.image(&texture).rect;
                drawing = rect.contains(origin) && rect.contains(current) && primary_button;
                origin.x -= rect.min.x;
                origin.y -= rect.min.y;
                current.x -= rect.min.x;
                current.y -= rect.min.y;

            });

            //TODO:
            //let mut tool = self.paint_tools.get_mut(&self.selected_paint_tool).unwrap_or_default();


            ui.label(format!("drawing:{} origin:{},{} current:{},{}", drawing, origin.x, origin.y, current.x, current.y));
        });

        //self.fill();
        //ctx.request_repaint();
    }
}