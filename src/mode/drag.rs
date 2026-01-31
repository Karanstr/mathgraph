use super::common::*;

pub struct Drag {
  selected: Option<usize> 
}
impl super::Mode for Drag {

  fn create(_program: &mut crate::GraphProgram) -> Self {
    Self { selected: None }
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {
    let Some(pos) = response.hover_pos() else { self.selected = None; return; };
    if response.drag_started_by(PointerButton::Primary) { self.selected = program.get_node_at(pos); }
    if response.drag_stopped_by(PointerButton::Primary) { self.selected = None; }

    if let Some(dragging) = self.selected {
      program.graph.nodes.get_mut(dragging).unwrap().position = pos;
    }
  }

}
