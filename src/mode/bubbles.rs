use super::common::*;

pub struct Bubbles {
  bubble: StrType<usize>,
  bubble_length: usize,
  state: StrType<usize>,
  state_length: usize,
}
impl super::Mode for Bubbles {

  fn create(program: &mut GraphProgram) -> Self {
    
    let (bubble_idx, state_idx) = if let Some(state_space) = &program.state_space {
      state_space.bubble_data(program.loaded_state)
    } else { (1,1) };

    Self {
      bubble: StrType::new(bubble_idx),
      bubble_length: 0,
      state: StrType::new(state_idx),
      state_length: 0,
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {

    let Some(state_space) = program.state_space.as_ref() else { return };

    let old_bubble_idx = self.bubble.val();
    // Identify bubble
    ui.input_text(
      hash!(),
      &format!("/{} Viewed Bubbles", state_space.bubbles.len()),
      self.bubble.string_mut()
    );
    self.bubble.parse();
    if self.bubble.val() != old_bubble_idx { self.state.assign(1); }
    self.bubble_length = state_space.bubbles.len();

    let Some(bubble_vec) = state_space.bubbles.get(self.bubble.val().saturating_sub(1)) else {
      return;
    };
    self.state_length = bubble_vec.len();

    // Identify view idx
    ui.input_text(
      hash!(),
      &format!("/{} Viewed States", bubble_vec.len()),
      self.state.string_mut()
    );
    self.state.parse();

    if self.bubble.val() == state_space.bubbles.len() {
      ui.label(Vec2::new(0., 100.), "Bubble of Size 1 Bubbles");
    }

    // Load current viewing state
    if let Some(state) = bubble_vec.get(self.state.val().saturating_sub(1)) {
      program.desired_state = *state;
    }
  }

  fn interactions(&mut self, _program: &mut GraphProgram) {

    if self.state_length != 0 {
      self.state.step_strnum(self.state_length, 1, KeyCode::Right, KeyCode::Left);
    }

    if self.bubble_length != 0 {
      self.bubble.step_strnum(self.bubble_length, 1, KeyCode::Up, KeyCode::Down);
    }

  }

}
