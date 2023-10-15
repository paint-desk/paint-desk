use std::collections::HashMap;
use itertools::Itertools;
use super::data_types::*;
use super::canvas::*;

pub struct Canvas {
    layers: CanvasLayers,
    undo_stack : Vec<EditCommand>,
    redo_stack : Vec<EditCommand>,

    tool_layer: HashMapCanvasLayer,
    draw_layer: FlatCanvasLayer,

    size: (u32, u32),
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Canvas {
        let mut canvas = Canvas {
            layers: CanvasLayers {
                entries: HashMap::new(),
                active_layer_id: LayerId(0),
            },
            undo_stack : Vec::new(),
            redo_stack : Vec::new(),

            tool_layer: HashMapCanvasLayer::new(w, h),
            draw_layer: FlatCanvasLayer::new(w, h),
            size: (w, h),
        };
        canvas.layers.entries.insert(LayerId(0), CanvasLayerEntry {
            order: 0,
            layer: FlatCanvasLayer::new(w, h),
        });
        canvas.layers.active_layer_id = LayerId(0);
        canvas.get_active_layer_mut().map(|active_canvas|{
            active_canvas.clear();
            for wi in 0..w {
                for hi in 0..h {
                    if wi==hi {
                        active_canvas.set_pixel(PixelPos{x: wi, y: hi}, Color::new(255, 0, 0, 255));
                    }
                }
            }

        });
        canvas.update_display_canvas();

        canvas
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
    fn apply_commands(&mut self, commands : &Vec<EditCommand>){
        if commands.len() > 0 {
            self.redo_stack.clear();
        }
        commands.iter().for_each(|command|{
            self.apply_command(command);
        });
    }

    fn apply_command(&mut self, command : &EditCommand){
        self.layers.get_active_layer_mut().map(|active_canvas|{
            command.apply(active_canvas);
            self.undo_stack.push(command.reverse(active_canvas));
        });
    }

    pub fn stroke_start(&mut self, pixel_pos: PixelPos, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_start(pixel_pos, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands(&commands);
    }

    pub fn stroke_update(&mut self, pixel_pos: PixelPos, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_update(pixel_pos, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands(&commands);

        self.update_display_canvas();
    }

    pub fn stroke_end(&mut self, pixel_pos: PixelPos, tool : &mut dyn PaintTool){
        let mut commands = Vec::new();
        tool.stroke_end(pixel_pos, &mut self.tool_layer, &mut |command| commands.push(command));
        self.apply_commands(&commands);

        self.update_display_canvas();
    }

    pub fn undo(&mut self){
        if let Some(command) = self.undo_stack.pop(){
            self.apply_command(&command);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self){
        if let Some(command) = self.redo_stack.pop(){
            self.apply_command(&command);
            self.undo_stack.push(command);
        }
    }

    fn update_display_canvas(&mut self){
        self.draw_layer.clear();

        let mut layers_to_apply = self.layers.entries.values()
            .sorted_by(|a, b| a.order.cmp(&b.order))
            .map(|entry| &entry.layer as &dyn CanvasLayer)
            .collect_vec();

        // make the tool_layer appear on top (you may want to apply it to correct layer instead)
        layers_to_apply.push(&self.tool_layer);

        apply_layers(layers_to_apply.into_iter(), &mut self.draw_layer);
    }
}

// applied in order (last one is on top)
pub fn apply_layers<'a>(canvases_iter : impl Iterator<Item = &'a dyn CanvasLayer>, target_canvas : &mut dyn CanvasLayer){
    canvases_iter.for_each(|canvas|{
        canvas.apply_to_canvas(target_canvas);
    });
}

pub struct CanvasLayers{
    entries: HashMap<LayerId, CanvasLayerEntry>,
    active_layer_id: LayerId,
}
pub struct CanvasLayerEntry{
    pub order: i32,
    pub layer: FlatCanvasLayer,
}

impl CanvasLayers {
    pub fn get_active_layer(&self) -> Option<&FlatCanvasLayer>{
        self.entries.get(&self.active_layer_id).map(|canvas| &canvas.layer)
    }

    pub fn get_active_layer_mut(&mut self) -> Option<&mut FlatCanvasLayer>{
        self.entries.get_mut(&self.active_layer_id).map(|canvas| &mut canvas.layer)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(usize);
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

pub trait PaintTool{
    // pass a function to push commands to
    fn stroke_start(&mut self, pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));
    fn stroke_update(&mut self, pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));
    fn stroke_end(&mut self, pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand));

}
pub struct PixelPencil {
    color : Color,
}
impl PixelPencil {
    pub fn new(color : Color, _size : u32) -> PixelPencil {
        PixelPencil {
            color,
        }
    }
}

impl PaintTool for PixelPencil {
    //fn stroke_start(&mut self, pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvas, push_command : &mut Vec<EditCommand>){
    //    tool_canvas.clear();
    //}
    // like that but push_command should be of type Action<EditCommand> in c#
    fn stroke_start(&mut self, _pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        tool_canvas.clear();
    }

    fn stroke_update(&mut self, pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, _push_command : &mut dyn FnMut(EditCommand)){
        tool_canvas.set_pixel(pixel_pos, self.color);
    }
    fn stroke_end(&mut self, _pixel_pos: PixelPos, tool_canvas : &mut HashMapCanvasLayer, push_command : &mut dyn FnMut(EditCommand)){
        let mut command = EditCommand::default();
        tool_canvas.pixels_iter().for_each(|(pos, color)|{
            command.edits.push((*pos, *color));
        });
        push_command(command);
        tool_canvas.clear();
    }

}