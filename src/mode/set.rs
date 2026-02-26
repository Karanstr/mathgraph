use super::common::*;

#[derive(Debug)]
pub struct Set {
  value: u8 
}
impl super::Mode for Set {

  fn create(_program: &GraphProgram) -> Self {
    Self { value: 0 }
  }

  fn ui(&mut self, _program: &mut GraphProgram, ui: &mut Ui) {
    DragValue::new(&mut self.value).ui(ui);
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {

    response.ctx.input(|input| {
      if input.key_pressed(Key::Num0) { self.value = 0; }
      if input.key_pressed(Key::Num1) { self.value = 1; }
      if input.key_pressed(Key::Num2) { self.value = 2; }
      if input.key_pressed(Key::Num3) { self.value = 3; }
      if input.key_pressed(Key::Num4) { self.value = 4; }
      if input.key_pressed(Key::Num5) { self.value = 5; }
      if input.key_pressed(Key::Num6) { self.value = 6; }
      if input.key_pressed(Key::Num7) { self.value = 7; }
      if input.key_pressed(Key::Num8) { self.value = 8; }
      if input.key_pressed(Key::Num9) { self.value = 9; }
      

      if let Some(pos) = input.pointer.hover_pos()
        && let Some(node) = program.get_node_at(pos)
        && self.value <= program.max
        && input.pointer.primary_pressed()
        && let Some(state_space) = &program.state_space
      {
        program.desired_state = state_space.set_packed(program.loaded_state, node, self.value);
      }
    });
    

  }
  
}

