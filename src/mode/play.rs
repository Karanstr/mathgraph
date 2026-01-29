use super::common::*;

pub struct Play { 
  allow_clamping: bool,
}
impl super::Mode for Play {

  fn create(_program: &mut GraphProgram) -> Self {
    Self {
      allow_clamping: true,
    }
  }

  fn ui(&mut self, _program: &mut GraphProgram, ui: &mut Ui) {
    ui.checkbox(hash!(), "Allow Clamping", &mut self.allow_clamping);
  }

  fn interactions(&mut self, program: &mut GraphProgram) {
    let delta = 
      if is_mouse_button_pressed(MouseButton::Left) { 1 }
      else if is_mouse_button_pressed(MouseButton::Right) { -1 }
      else { return } as i8
    ;
    if   let Some(node) = program.get_hovering()
      && let Some(state_space) = &program.state_space
        && let Some(state) = state_space.splash_state(
          program.loaded_state,
          node,
          delta,
          self.allow_clamping
        )
    {
      program.desired_state = state;
    }
  }

}
