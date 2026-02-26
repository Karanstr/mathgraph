use super::common::*;

#[derive(Debug)]
pub struct Play { 
  reversible_only: bool,
}
impl super::Mode for Play {

  fn create(_program: &GraphProgram) -> Self {
    Self {
      reversible_only: true,
    }
  }

  fn ui(&mut self, _program: &mut GraphProgram, ui: &mut Ui) {
    ui.checkbox(&mut self.reversible_only, "Reversible Only");
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {
    let mut delta: i8 = 0;
    response.ctx.input(|input| {
      delta = if input.pointer.primary_pressed() { 1 }
      else if input.pointer.secondary_pressed() { -1 }
      else { return };
    });
    if let Some(node) = program.get_node_at(response.ctx.pointer_interact_pos().unwrap())
      && let Some(state_space) = &program.state_space
      && let Some(state) = state_space.splash_state(
        program.loaded_state,
        node,
        delta,
        self.reversible_only
      )
    {
      program.desired_state = state;
    }
  }

}
