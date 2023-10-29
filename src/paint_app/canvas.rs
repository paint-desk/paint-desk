use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use itertools::Itertools;
use crate::paint_app::utils::{checkers_pattern, draw_rect, rasterize_line};
use super::data_types::*;
use super::canvas_layer::*;

pub struct Canvas {
    layers: CanvasLayers,
    undo_stack : Vec<EditCommand>,
    redo_stack : Vec<EditCommand>,

    tool_layer: HashMapCanvasLayer,
    draw_layer: FlatCanvasLayer,

    checkers_pattern_layer: FlatCanvasLayer,

    size: (u32, u32),
}

// supports equality comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasLayerConfig{
    pub id : LayerId,
    pub visible: bool,
}
// supports deep equality comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasLayersConfig{
    pub entries : Vec<CanvasLayerConfig>,
    pub active_layer_id : LayerId,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Canvas {
        let mut canvas = Canvas {
            layers: CanvasLayers {
                entries: Vec::new(),
                active_layer_id: LayerId(0),
            },
            undo_stack : Vec::new(),
            redo_stack : Vec::new(),

            checkers_pattern_layer: Canvas::create_checkers_pattern(w, h, 10),
            tool_layer: HashMapCanvasLayer::new(w, h),
            draw_layer: FlatCanvasLayer::new(w, h),
            size: (w, h),
        };
        let mut green_horizontal = FlatCanvasLayer::new(w, h);
        draw_rect(&mut green_horizontal, PixelPos{x: 100, y: 100}, PixelPos{x: 100 + 100, y: 100 + 10}, Color::new(0, 255, 0, 255));
        canvas.layers.entries.push(CanvasLayerEntry {
            id: LayerId(0),
            layer: green_horizontal,
            visible: true,
        });

        let mut red_vertical = FlatCanvasLayer::new(w, h);
        draw_rect(&mut red_vertical, PixelPos{x: 100, y: 100}, PixelPos{x: 100 + 10, y: 100 + 100}, Color::new(255, 0, 0, 255));
        canvas.layers.entries.push(CanvasLayerEntry {
            id: LayerId(1),
            layer: red_vertical,
            visible: true,
        });

        canvas.layers.active_layer_id = LayerId(0);
        //canvas.get_active_layer_mut().map(|active_canvas|{
        //    active_canvas.fill(Color::white());
        //});
        canvas.update_display_canvas();

        canvas
    }

    pub fn get_canvas_layers_config(&self) -> CanvasLayersConfig {
        CanvasLayersConfig{
            entries: self.layers.entries.iter().map(|entry| CanvasLayerConfig{
                id: entry.id,
                visible: entry.visible,
            }).collect_vec(),
            active_layer_id: self.layers.active_layer_id,
        }
    }

    pub fn set_canvas_layers_config(&mut self, config : CanvasLayersConfig){
        let changed = self.get_canvas_layers_config() != config;
        if changed {
            self.layers.active_layer_id = config.active_layer_id;


            //sort self.layers.entries based on LayerId looking at config.entries
            let mut id_to_order = HashMap::new();
            config.entries.iter().enumerate().for_each(|(i, entry)|{
                id_to_order.insert(entry.id, i);
            });
            self.layers.entries.sort_by_key(|entry| id_to_order.get(&entry.id).unwrap_or(&0));

            // set the "visible"
            self.layers.entries.iter_mut().for_each(|entry|{
                let new_entry = config.entries.iter().find(|new_entry| new_entry.id == entry.id);
                if let Some(new_entry) = new_entry {
                    entry.visible = new_entry.visible;
                }
            });

            self.update_display_canvas();
        }
    }


    pub fn get_draw_layer(&self) -> &FlatCanvasLayer {
        &self.draw_layer
    }
    pub fn get_active_layer(&self) -> Option<&FlatCanvasLayer>{
        self.layers.get_active_layer()
    }

    pub fn get_active_layer_mut(&mut self) -> Option<&mut FlatCanvasLayer>{
        self.layers.get_active_layer_mut()
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }
    fn apply_commands_handle_undo_redo(&mut self, commands : &Vec<EditCommand>){
        if commands.len() > 0 {
            self.redo_stack.clear();
        }
        commands.iter().for_each(|command|{
            self.apply_command_handle_undo_redo(command);
        });
    }

    fn apply_command_handle_undo_redo(&mut self, command : &EditCommand){
        self.layers.get_active_layer_mut().map(|active_canvas|{
            self.undo_stack.push(command.reverse(active_canvas));
            command.apply(active_canvas);
        });
    }

    pub fn stroke_start(&mut self, global_params: &GlobalParams, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_start(global_params, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands_handle_undo_redo(&commands);
    }

    pub fn stroke_update(&mut self, global_params: &GlobalParams, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_update(global_params, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands_handle_undo_redo(&commands);

        self.update_display_canvas();
    }

    pub fn stroke_end(&mut self, global_params: &GlobalParams, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_end(global_params, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands_handle_undo_redo(&commands);

        self.update_display_canvas();
    }

    pub fn undo(&mut self){
        if let Some(command) = self.undo_stack.pop(){
            if let Some(active_layer) = self.layers.get_active_layer_mut(){
                self.redo_stack.push(command.reverse(active_layer));
                command.apply(active_layer);
            }

            self.update_display_canvas();
            println!("undo");
            println!("undo_stack: {}", self.undo_stack.len());
            println!("redo_stack: {}", self.redo_stack.len());
        }
        else {
            println!("nothing to undo");
        }
    }

    pub fn redo(&mut self){
        if let Some(command) = self.redo_stack.pop(){
            self.undo_stack.push(command.reverse(self.get_active_layer().unwrap()));
            //self.apply_command(&command);
            if let Some(active_layer) = self.layers.get_active_layer_mut(){
                command.apply(active_layer);
            }
            self.update_display_canvas();

            println!("redo");
            println!("undo_stack: {}", self.undo_stack.len());
            println!("redo_stack: {}", self.redo_stack.len());
        }
    }

    fn update_display_canvas(&mut self){
        self.draw_layer.clear();

        let mut layers_to_apply = Vec::new();

        layers_to_apply.push(&self.checkers_pattern_layer as &dyn CanvasLayer);

        let canvas_layers = self.layers.entries.iter()
            .rev()
            .filter(|entry| entry.visible)
            .map(|entry| &entry.layer as &dyn CanvasLayer)
            .collect_vec();

        layers_to_apply.extend(canvas_layers);

        // make the tool_layer appear on top (you may want to apply it to correct layer instead)
        layers_to_apply.push(&self.tool_layer);


        apply_layers(layers_to_apply.into_iter(), &mut self.draw_layer);
    }

    fn create_checkers_pattern(w: u32, h: u32, grid_len : usize) -> FlatCanvasLayer {
        let mut result = FlatCanvasLayer::new(w, h);
        result.iter_pixels_mut().for_each(|(pos, color)|{
            *color = checkers_pattern(pos, grid_len, Color::new(225, 225, 225, 255), Color::new(200, 200, 200, 255));
        });
        result
    }
}

// applied in order (last one is on top)
pub fn apply_layers<'a>(canvases_iter : impl Iterator<Item = &'a dyn CanvasLayer>, target_canvas : &mut dyn CanvasLayer){
    canvases_iter.for_each(|canvas|{
        canvas.apply_to_canvas(target_canvas);
    });
}

pub struct CanvasLayers{
    pub entries: Vec<CanvasLayerEntry>,
    pub active_layer_id: LayerId,
}
pub struct CanvasLayerEntry{
    pub id: LayerId,
    pub layer: FlatCanvasLayer,
    pub visible: bool,
}
impl Hash for CanvasLayerEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl CanvasLayers {
    pub fn get_active_layer(&self) -> Option<&FlatCanvasLayer>{
        //self.entries.get(&self.active_layer_id).map(|canvas| &canvas.layer)
        self.entries.iter().find(|entry| entry.id == self.active_layer_id).map(|entry| &entry.layer)
    }

    pub fn get_active_layer_mut(&mut self) -> Option<&mut FlatCanvasLayer>{
        //self.entries.get_mut(&self.active_layer_id).map(|canvas| &mut canvas.layer)
        self.entries.iter_mut().find(|entry| entry.id == self.active_layer_id).map(|entry| &mut entry.layer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(pub usize);

impl Display for LayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasOrder(usize);

#[derive(Default, Clone)]
pub struct EditCommand {
    pub edits : Vec<(PixelPos, Color)>,
}

impl EditCommand {
    pub fn apply(&self, canvas : &mut dyn CanvasLayer){
        self.edits.iter().for_each(|(pos, color)|{
            canvas.set_pixel(*pos, *color);
        });
    }
    pub fn reverse(&self, canvas : &dyn CanvasLayer) -> EditCommand {
        let mut result = EditCommand::default();
        self.edits.iter().for_each(|(pos, _color)|{
            result.edits.push((*pos, canvas.get_pixel(*pos)));
        });
        result
    }
}

pub trait PaintTool {
    fn get_name(&self) -> &str;

    // pass a function to push commands to
    fn stroke_start(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));
    fn stroke_update(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));
    fn stroke_end(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));

}
pub struct PixelPencil {
    name: String,
    previous_point : Option<PixelPos>,
}
impl PixelPencil {
    pub fn new() -> PixelPencil {
        PixelPencil {
            name: "Pencil".to_string(),
            previous_point : None
        }
    }
}

impl PaintTool for PixelPencil {
    fn get_name(&self) -> &str {
        &self.name
    }

    // like that but push_command should be of type Action<EditCommand> in c#
    fn stroke_start(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        tool_canvas.clear();
        self.previous_point = None;
    }

    fn stroke_update(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        if let Some(previous_point) = self.previous_point {
            rasterize_line(previous_point, global_params.current_pixel.unwrap_or_default()).iter().for_each(|pos|{
                tool_canvas.set_pixel(*pos, global_params.primary_color);
            });
        }

        self.previous_point = global_params.current_pixel;
    }
    fn stroke_end(&mut self, global_params: &GlobalParams, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand)){
        let mut command = EditCommand::default();
        tool_canvas.pixels_iter().for_each(|(pos, color)|{
            command.edits.push((*pos, *color));
        });
        push_command(command);
        tool_canvas.clear();
        println!("command pushed");

        self.previous_point = None;
    }

}

pub struct LineTool {
    name: String,
    line_start_point : Option<PixelPos>,
}

impl LineTool {
    pub fn new() -> LineTool {
        LineTool {
            name: "Line".to_string(),
            line_start_point : None,
        }
    }
}

impl PaintTool for LineTool {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn stroke_start(&mut self, global_params: &GlobalParams, _tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        self.line_start_point = global_params.current_pixel;
    }

    fn stroke_update(&mut self, global_params: &GlobalParams, _tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        _tool_canvas.clear();
        rasterize_line(self.line_start_point.unwrap_or_default(), global_params.current_pixel.unwrap_or_default()).iter().for_each(|pos|{
            _tool_canvas.set_pixel(*pos, global_params.primary_color);
        });
    }
    fn stroke_end(&mut self, global_params: &GlobalParams, _tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand)){
        if let Some(line_start_point) = self.line_start_point {
            let mut command = EditCommand::default();
            rasterize_line(line_start_point, global_params.current_pixel.unwrap_or_default()).iter().for_each(|pos|{
                command.edits.push((*pos, global_params.primary_color));
            });
            push_command(command);
            println!("command pushed");
        }
        _tool_canvas.clear();
        self.line_start_point = None;
    }

}

#[derive(Debug,Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerConfig{
    pub id : LayerId,
    pub visible : bool,
}