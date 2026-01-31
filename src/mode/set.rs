use super::common::*;

pub struct Set {
  value: StrType<u8> 
}
impl super::Mode for Set {

  fn create(_program: &mut GraphProgram) -> Self {
    Self { value: StrType::new(0) }
  }

  fn ui(&mut self, _program: &mut GraphProgram, ui: &mut Ui) {
    ui.text_edit_singleline(self.value.string_mut());
    self.value.parse();
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {

    response.ctx.input(|input| {
      if input.key_pressed(Key::Num0) { self.value.assign(0); }
      if input.key_pressed(Key::Num1) { self.value.assign(1); }
      if input.key_pressed(Key::Num2) { self.value.assign(2); }
      if input.key_pressed(Key::Num3) { self.value.assign(3); }
      if input.key_pressed(Key::Num4) { self.value.assign(4); }
      if input.key_pressed(Key::Num5) { self.value.assign(5); }
      if input.key_pressed(Key::Num6) { self.value.assign(6); }
      if input.key_pressed(Key::Num7) { self.value.assign(7); }
      if input.key_pressed(Key::Num8) { self.value.assign(8); }
      if input.key_pressed(Key::Num9) { self.value.assign(9); }
      

      if let Some(pos) = input.pointer.hover_pos()
        && let Some(node) = program.get_node_at(pos)
        && self.value.val() <= program.max.val()
        && input.pointer.primary_pressed()
        && let Some(state_space) = &program.state_space
      {
        program.desired_state = state_space.set_packed(program.loaded_state, node, self.value.val());
      }
    });
    

  }
  
}

