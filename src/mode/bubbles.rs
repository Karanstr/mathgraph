use eframe::egui::Event;

use super::common::*;
use crate::state::PackedState;

pub struct Bubbles {
  bubble: StrType<usize>,
  bubble_length: usize,
  state: StrType<usize>,
  state_length: usize,
}
impl Bubbles {
  pub fn assign_self(&mut self, program: &mut GraphProgram, state: PackedState) {
    let (bubble_idx, state_idx) = if let Some(state_space) = &program.state_space {
      state_space.bubble_data(state)
    } else { (0, 0) };
    self.bubble.assign(bubble_idx + 1);
    self.state.assign(state_idx + 1);
  }
}
impl super::Mode for Bubbles {

  fn create(program: &GraphProgram) -> Self {
    
    let (bubble_idx, state_idx) = if let Some(state_space) = &program.state_space {
      state_space.bubble_data(program.loaded_state)
    } else { (0, 0) };

    Self {
      bubble: StrType::new(bubble_idx + 1),
      bubble_length: 0,
      state: StrType::new(state_idx + 1),
      state_length: 0,
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {

    let Some(state_space) = program.state_space.as_ref() else { return };

    let old_bubble_idx = self.bubble.val();
    ui.horizontal(|ui| {
      TextEdit::singleline(self.bubble.string_mut())
        .id(Id::new("Bubble"))
        .show(ui)
      ;
      ui.label(format!("/{} Viewed Bubbles", self.bubble_length));
    });
    self.bubble.parse();
    if self.bubble.val() != old_bubble_idx { self.state.assign(1); }
    self.bubble_length = state_space.bubbles.len();

    let Some(bubble_vec) = state_space.bubbles.get(self.bubble.val().saturating_sub(1)) else {
      return;
    };
    self.state_length = bubble_vec.len();

    ui.horizontal(|ui| {
      TextEdit::singleline(self.state.string_mut())
        .id(Id::new("State"))
        .show(ui)
      ;
      ui.label(format!("/{} Viewed States", self.state_length));
    });
    self.state.parse();
    if self.bubble.val() == state_space.bubbles.len() {
      ui.label("Bubble of Size 1 Bubbles");
    }

    // Load current viewing state
    if let Some(state) = bubble_vec.get(self.state.val().saturating_sub(1)) {
      program.desired_state = *state;
    }
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {

    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut left_pressed = false;
    let mut right_pressed = false;
    response.ctx.input(|input| {
      for event in &input.events {
        match event {
          Event::Key { key, pressed: true, repeat: false, .. } => {
            match *key {
              Key::ArrowDown => down_pressed = true,
              Key::ArrowUp => up_pressed = true,
              Key::ArrowLeft => left_pressed = true,
              Key::ArrowRight => right_pressed = true,
              _ => (),
            }
          }
          _ => {}
        }
      }

      // Play the game with only reversible actions
      let delta = 
        if input.pointer.primary_pressed() { 1 }
        else if input.pointer.secondary_pressed() { -1 }
        else { return } as i8
      ;
      if   let Some(node) = program.get_node_at(input.pointer.interact_pos().unwrap())
        && let Some(state_space) = &program.state_space
        && let Some(state) = state_space.splash_state(
          program.loaded_state,
          node,
          delta,
          false
        )
      {
        self.assign_self(program, state);
      }

    });
    
    if self.state_length != 0 && (right_pressed || left_pressed) {
      self.state.step_strnum(self.state_length, 1, right_pressed);
    }
    if self.bubble_length != 0 && (up_pressed || down_pressed) {
      self.bubble.step_strnum(self.bubble_length, 1, up_pressed);
    }

  }

}
