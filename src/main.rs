use std::time::Instant;
use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{ColorImage, PointerButton, Pos2, Rect, Sense, Vec2};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Texture Drawer", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc, 1024, 768))));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(MyEguiApp::new(cc, 1024, 768))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[derive(Debug, Clone, Copy)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8
}

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: a
        }
    }
}

//#[derive(Default)]
struct MyEguiApp {
    width: u32,
    height: u32,
    data: Vec<Color>,
    start_time: f32,
    frame_times: Vec<f32>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, w:u32, h:u32) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        MyEguiApp {
            width: w,
            height: h,
            data: vec!(Color::new(0, 255, 0, 0); (w * h) as usize),
            start_time: 0f32,
            frame_times: Vec::new()
        }
    }


    fn fill(&mut self)
    {
        for i in 0..self.width as usize * self.height as usize {
            //self.data[i] = Color::new(255, 0, 0, 0);//rand::random::<u32>();
            self.data[i].green = 0;
            self.data[i].blue = 255;
            self.data[i].alpha = 255;
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

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");

            let fps = 0f32;//self.get_fps();
            ui.label(format!("FPS: {:.2}", fps));

            let slice: &[Color] = &self.data;

            // Unsafe conversion from &[u32] to &[u8]
            let byte_slice: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    slice.as_ptr() as *const u8,
                    slice.len() * std::mem::size_of::<Color>(),
                )
            };

            let image = ColorImage::from_rgba_premultiplied([self.width as usize, self.height as usize], &byte_slice);

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

            ui.label(format!("drawing:{} origin:{},{} current:{},{}", drawing, origin.x, origin.y, current.x, current.y));
        });

        self.fill();
        ctx.request_repaint();
    }
}