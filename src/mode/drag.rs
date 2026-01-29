use super::common::*;

pub struct Drag {
  selected: Option<usize> 
}
impl super::Mode for Drag {

  fn create(_program: &mut crate::GraphProgram) -> Self {
    Self { selected: None }
  }

  fn interactions(&mut self, program: &mut GraphProgram) {
    if is_mouse_button_pressed(MouseButton::Left) { self.selected = program.get_hovering(); }
    if is_mouse_button_released(MouseButton::Left) { self.selected = None; }

    if let Some(dragging) = self.selected {
      program.graph.nodes.get_mut(dragging).unwrap().position = GraphProgram::get_mouse_pos();
    }
  }

}
