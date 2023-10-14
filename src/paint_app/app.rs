use std::collections::HashMap;
use super::data_types::*;
use super::canvas::*;

pub struct Canvas {
    layers: CanvasLayers,
    undo_stack : Vec<EditCommand>,
    redo_stack : Vec<EditCommand>,

    tool_layer: HashMapCanvasLayer,
    draw_layer: FlatCanvasLayer,
}

impl Canvas {
    fn apply_commands(&mut self, commands : &Vec<EditCommand>){
        if commands.len() > 0 {
            self.redo_stack.clear();
        }
        commands.iter().for_each(|command|{
            self.apply_command(command);
        });
    }
    fn apply_command(&mut self, command : &EditCommand){
        //if let Some(active_canvas) = self.get_active_canvas_mut(){
        //    command.apply(active_canvas);
        //    self.undo_stack.push(command.reverse(active_canvas));
        //}
        //else { println!("no active canvas"); }

        // reverse if - else
        self.layers.get_active_canvas_mut().map(|active_canvas|{
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

    }
}

pub struct CanvasLayers{
    layers: HashMap<LayerId, FlatCanvasLayer>,
    active_layer_id: LayerId,

}

impl CanvasLayers {
    pub fn get_active_canvas(&self) -> Option<&dyn CanvasLayer>{
        self.layers.get(&self.active_layer_id).map(|canvas| canvas as &dyn CanvasLayer)
    }
    pub fn get_active_canvas_mut(&mut self) -> Option<&mut dyn CanvasLayer>{
        self.layers.get_mut(&self.active_layer_id).map(|canvas| canvas as &mut dyn CanvasLayer)
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