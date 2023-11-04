use std::collections::HashMap;
use std::time::Instant;
use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{Button, Color32, ColorImage, PointerButton, Pos2, Rect, Sense, Vec2, menu, WidgetText};
use paint_app::size_window::SizeWindow;
use crate::paint_app::canvas::{Canvas, CanvasLayerEntry, LayerConfig, LayerId, LineTool, PaintTool, PixelPencil};
use crate::paint_app::canvas_layer::{CanvasLayer, FlatCanvasLayer};
use crate::paint_app::data_types::*;
use egui_dnd::*;


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
    selected_paint_tool: u32,
    size_dialog: SizeWindow
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
            selected_paint_tool: 1,
            size_dialog: SizeWindow::new()
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

            //Fill tool box
            for (key, value) in self.paint_tools.iter() {
                if ui.selectable_label(self.selected_paint_tool == *key, value.get_name()).clicked() {
                    self.selected_paint_tool = *key;
                }
            }

            ui.separator();

            ui.heading("Color");
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
            let mut checked = false;
            ui.vertical(|ui| {
                ui.heading("Tool");
                ui.label("Current tool extra settings");
                ui.spacing();
                ui.separator();
                
                ui.heading("Layers");

                let mut canvas_layers_config = self.canvas.get_canvas_layers_config();
                let layers = &mut canvas_layers_config.entries;
                let active_layer_id =  &mut canvas_layers_config.active_layer_id;

                ui.vertical(|ui| {
                    dnd(ui, "2dnd_example2")
                        .show_vec(layers, |ui, item, handle, _state| {
                            handle.ui(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("layer: {}", item.id.0));
                                        ui.checkbox(&mut item.visible, "visible");
                                        // tickbox
                                        let mut active = *active_layer_id == item.id;
                                        ui.checkbox(&mut active, "active");
                                        if active {
                                            *active_layer_id = item.id;
                                        }
                                    });
                                    //ui.image(&texture_id);
                                });
                            });
                        });
                });

                self.canvas.set_canvas_layers_config(canvas_layers_config);


                //TODO: add real layer list
                //let mut checked = false;
                //ui.checkbox(&mut checked, "Layer 1");
                //ui.checkbox(&mut checked, "Layer 2");
                
            });
        });
    }

    fn draw_center(&mut self, ctx: &egui::Context, take_input: bool) -> egui::InnerResponse<()> {
        egui::CentralPanel::default().show(ctx, |ui| {

            let mut input = take_input;
            let mut a_dialog_opened = false;
            a_dialog_opened = self.handle_dialogs(ctx);
            if a_dialog_opened {
                input = false;
            }
            ui.set_enabled(!a_dialog_opened);

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

            if input {
                ctx.input(|s| {
                    middle_button = s.pointer.button_down(PointerButton::Middle);
                    self.primary_button = s.pointer.button_down(PointerButton::Primary);
                    current = s.pointer.latest_pos().unwrap_or_default();
                    ctrl_key = s.modifiers.ctrl;
                    z_key = s.key_pressed(egui::Key::Z);
                    y_key = s.key_pressed(egui::Key::Y);
                });
            }

            let mut image_rect: Rect = Rect::from_two_pos(Pos2::new(0f32, 0f32), Pos2::new(0f32, 0f32));
            let scroll_area = egui::ScrollArea::both().drag_to_scroll(middle_button).show(ui, |ui| {
                image_rect = ui.image(&texture).rect;
            });
            self.global_params.cursor_in_canvas = scroll_area.inner_rect.contains(current);
            current.x -= image_rect.min.x;
            current.y -= image_rect.min.y;

            if input {
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

    fn handle_dialogs(&mut self, ctx: &egui::Context) -> bool {
        let mut dialog_opened = false;

        if self.size_dialog.open {
            dialog_opened = true;
            self.size_dialog.show_size_window(ctx);
        }

        dialog_opened
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

    fn draw_panel_top(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open (TODO)").clicked() {
                        ui.close_menu();
                        //TODO: open file
                        //This will be handled differently for web and native
                    }
                    if ui.button("Save (TODO)").clicked() {
                        ui.close_menu();
                        //TODO: save file
                        //This will be handled differently for web and native
                    }
                    if ui.button("Size...").clicked() {
                        ui.close_menu();
                        self.size_dialog.width = self.canvas.get_size().0;
                        self.size_dialog.height = self.canvas.get_size().1;
                        self.size_dialog.open = true;
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        ui.close_menu();
                        self.canvas.undo();
                    }
                    if ui.button("Redo").clicked() {
                        ui.close_menu();
                        self.canvas.redo();
                    }

                    ui.separator();

                    if ui.button("Canvas size... (TODO)").clicked() {
                        ui.close_menu();
                        //TODO: open canvas size dialog
                    }

                    if ui.button("Resize... (TODO)").clicked() {
                        ui.close_menu();
                        //TODO: open resize dialog
                    }
                });
            });
        });
    }
}

impl eframe::App for AppContext {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        let mut take_input: bool = true;

        self.draw_panel_top(ctx);

        self.draw_panel_left(ctx, &mut take_input);

        self.draw_panel_right(ctx);

        self.draw_panel_bottom(ctx);
        
        self.draw_center(ctx, take_input);

        self.handle_tool_events();

        //ctx.request_repaint();
    }
}

//impl Into<WidgetText> for LayerConfig {
//    fn into(self) -> WidgetText {
//        let layer_str = self.id.0.to_string();
//
//        // layer : layer_str
//        let final_str = format!("layer : {}", layer_str);
//        WidgetText::from(final_str)
//    }
//}

impl Into<WidgetText> for CanvasLayerEntry {
    fn into(self) -> WidgetText {
        let layer_str = self.id.0.to_string();

        // layer : layer_str
        let final_str = format!("layer : {}", layer_str);
        WidgetText::from(final_str)
    }
}
