use std::collections::HashMap;
use std::time::Instant;
use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{Button, Color32, ColorImage, PointerButton, Pos2, Rect, Sense, Vec2};
use crate::paint_app::app::{Canvas, LineTool, PaintTool, PixelPencil};
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
    tool_button_started: bool,
    primary_button: bool,
    canvas: Canvas,
    global_params: GlobalParams,
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
            tool_button_started: false,
            primary_button: false,
            canvas: Canvas::new(w, h),
            global_params: GlobalParams::new(),
            paint_tools: HashMap::new(),
            selected_paint_tool: 2
        };
        app.paint_tools.insert(1, Box::new(PixelPencil::new()));
        app.paint_tools.insert(2, Box::new(LineTool::new()));

        app
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

    fn draw_panel_left(&mut self, ctx: &egui::Context, take_input: &mut bool) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Tools");

            let mut popup_open = false;
            ui.memory(|mem|{popup_open = mem.any_popup_open()});
            if popup_open {
                *take_input = false;
            }

            if ui.small_button("Pencil").clicked() {
                self.selected_paint_tool = 1;
            }
            if ui.small_button("Line").clicked() {
                self.selected_paint_tool = 2;
            }

            let mut color_primary = self.global_params.primary_color.to_color32();
            ui.color_edit_button_srgba(&mut color_primary);
            self.global_params.primary_color = Color::from_color32(&color_primary);

            let mut color_secondary = self.global_params.secondary_color.to_color32();
            ui.color_edit_button_srgba(&mut color_secondary);
            self.global_params.secondary_color = Color::from_color32(&color_secondary);
        });
    }

    fn draw_panel_right(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            if ui.button("Undo").clicked() {
                self.canvas.undo();
            };

            if ui.button("Redo").clicked() {
                self.canvas.redo();
            };
        });
    }

    fn draw_center(&mut self, ctx: &egui::Context, take_input: bool) -> egui::InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");

            let fps = 0f32;//self.get_fps();
            ui.label(format!("FPS: {:.2}", fps));

            let size = self.canvas.get_size();

            let slice: &[Color] = &self.canvas.get_draw_layer().get_data();

            // Unsafe conversion from &[Color] to &[u8]
            let byte_slice: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    slice.as_ptr() as *const u8,
                    slice.len() * std::mem::size_of::<Color>(),
                )
            };

            let image = ColorImage::from_rgba_premultiplied([size.0 as usize, size.1 as usize], &byte_slice);

            let texture = ui.ctx().load_texture("aa", image, TextureOptions::NEAREST);


            let mut middle_button = false;
            let mut current = Pos2::new(0f32, 0f32);
            let mut ctrl_key = false;
            let mut z_key = false;
            let mut y_key = false;

            ctx.input(|s| {
                middle_button = s.pointer.button_down(PointerButton::Middle);
                self.primary_button = s.pointer.button_down(PointerButton::Primary);
                current = s.pointer.latest_pos().unwrap_or_default();
                ctrl_key = s.modifiers.ctrl;
                z_key = s.key_pressed(egui::Key::Z);
                y_key = s.key_pressed(egui::Key::Y);
            });

            let mut image_rect: Rect = Rect::from_two_pos(Pos2::new(0f32, 0f32), Pos2::new(0f32, 0f32));
            let scroll_area = egui::ScrollArea::both().drag_to_scroll(middle_button).show(ui, |ui| {
                image_rect = ui.image(&texture).rect;
            });
            self.global_params.cursor_in_canvas = scroll_area.inner_rect.contains(current);
            current.x -= image_rect.min.x;
            current.y -= image_rect.min.y;

            if take_input {
                let pixel = PixelPos { x: current.x as u32, y: current.y as u32 };
                self.global_params.current_pixel = match self.global_params.cursor_in_canvas {
                    true => Some(pixel),
                    false => None
                };
            }

            if ctrl_key && z_key {
                self.canvas.undo();
            }
            if ctrl_key && y_key {
                self.canvas.redo();
            }

            //ui.label(format!("drawing:{} origin:{},{} current:{},{}", drawing, origin.x, origin.y, current.x, current.y));
        })
    }

    fn handle_tool_events(&mut self) {
        match self.paint_tools.get_mut(&self.selected_paint_tool) {
            Some(value) => {
                let contains = self.global_params.cursor_in_canvas;
                if contains && !self.tool_button_started && self.primary_button {
                    self.canvas.stroke_start(&self.global_params, value.as_mut());
                    self.tool_button_started = true;
                } else {
                    if self.tool_button_started {
                        if contains && self.primary_button {
                            self.canvas.stroke_update(&self.global_params, value.as_mut());
                        } else {
                            self.canvas.stroke_end(&self.global_params, value.as_mut());
                            self.tool_button_started = false;
                        }
                    }
                }

                
            }
            None => {}
        }
    }

    fn draw_panel_bottom(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let pos = self.global_params.current_pixel;
            match pos {
                Some(value) => {
                    ui.label(format!("{} x {}", value.x, value.y));
                }
                None => {}
        
            }
        });
    }
}

impl eframe::App for AppContext {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        let mut take_input: bool = true;

        self.draw_panel_left(ctx, &mut take_input);

        self.draw_panel_right(ctx);

        self.draw_panel_bottom(ctx);
        
        self.draw_center(ctx, take_input);

        self.handle_tool_events();

        //ctx.request_repaint();
    }
}