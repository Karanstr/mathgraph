use super::common::*;

pub struct Set {
  value: StrType<u8> 
}
impl super::Mode for Set {

  fn create(_program: &mut GraphProgram) -> Self {
    Self { value: StrType::new(0) }
  }

  fn ui(&mut self, _program: &mut GraphProgram, ui: &mut Ui) {
    ui.input_text(hash!(), "Value", self.value.string_mut());
    self.value.parse();
  }

  fn interactions(&mut self, program: &mut GraphProgram) {

    if let Some(key) = get_last_key_pressed() {
      let val = match key {
        KeyCode::Key0 => 0,
        KeyCode::Key1 => 1,
        KeyCode::Key2 => 2,
        KeyCode::Key3 => 3,
        KeyCode::Key4 => 4,
        KeyCode::Key5 => 5,
        KeyCode::Key6 => 6,
        KeyCode::Key7 => 7,
        KeyCode::Key8 => 8,
        KeyCode::Key9 => 9,
        _ => self.value.val(),
      };
      self.value.assign(val);
    }
    
    if let Some(node) = program.get_hovering()
      && self.value.val() <= program.max.val()
      && is_mouse_button_pressed(MouseButton::Left)
      && let Some(state_space) = &program.state_space
    {
      program.desired_state = state_space.set_packed(program.loaded_state, node, self.value.val());
    }

  }
  
}

