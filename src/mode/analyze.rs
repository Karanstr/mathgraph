use super::common::*;
use crate::state::{PackedState, Classification, frequency_analysis, parse_analysis};

pub struct Analyze {
  viewing_type: usize,
  viewing_length: usize,
  viewing: StrType<usize>,
  parsed_analysis: Vec<Vec<u32>>,
}
impl Analyze {
  pub fn get_analysis(&self) -> &Vec<Vec<u32>> { &self.parsed_analysis }
}
impl super::Mode for Analyze {

  fn create(program: &mut GraphProgram) -> Self {

    let (viewing_type, idx) = if let Some(state_space) = &program.state_space {
      let (classification, idx) = state_space.classification_data(program.loaded_state);
      (classification as usize, idx)
    } else { (0, 1) };

    Self {
      viewing_type,
      viewing_length: 0,
      // Wow I hate that I did this off by one nonsense
      viewing: StrType::new(idx + 1),
      parsed_analysis: Vec::new(),
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {

    let Some(state_space) = program.state_space.as_ref() else { return };

    let total = (state_space.base as usize).pow(state_space.length() as u32);
    ui.label(Vec2::new(30., 110.), &format!("{total} Total"));

    // Identify view type
    let old_type = self.viewing_type;
    ui.combo_box(hash!(), "Mode", &[
      "All Invalid",    // 0
      "Bad States",     // 1
      "NotBad States",  // 2
      "All Valid",      // 3
    ], &mut self.viewing_type);
    let focused_states = match self.viewing_type {
      0 => &combine(
        state_space.get_list(Classification::InvalidOther), 
        state_space.get_list(Classification::InvalidT1)
      ),
      1 => state_space.get_list(Classification::InvalidT1),
      2 => state_space.get_list(Classification::InvalidOther),
      3 => state_space.get_list(Classification::Valid),
      _ => unreachable!()
    };

    self.viewing_length = focused_states.len();

    // Identify view idx
    ui.input_text(
      hash!(),
      &format!("/{} Viewed States", self.viewing_length),
      self.viewing.string_mut()
    );
    self.viewing.parse();

    if self.parsed_analysis.is_empty() || old_type != self.viewing_type {
      let analysis = frequency_analysis(focused_states, state_space.length(), program.max.val());
      self.parsed_analysis = parse_analysis(analysis, program.max.val(), state_space.length() as u8);
    }

    // Load current viewing state
    if let Some(state) = focused_states.get(self.viewing.val().saturating_sub(1)) {
      program.desired_state = *state;
    }
  }

  fn interactions(&mut self, _program: &mut GraphProgram) {

    if self.viewing_length != 0 {
      self.viewing.step_strnum(self.viewing_length, 1, KeyCode::Right, KeyCode::Left);
    }

    if is_key_pressed(KeyCode::Up) {
      self.viewing_type = if self.viewing_type == 0 { 3 } else { self.viewing_type - 1 };
    }
    if is_key_pressed(KeyCode::Down) {
      self.viewing_type = (self.viewing_type + 1) % 4;
    }
  }

}


fn combine<'a>(a: &'a [PackedState], b: &'a [PackedState]) -> Vec<PackedState> {
  let mut out = Vec::with_capacity(a.len() + b.len());
  out.extend_from_slice(a);
  out.extend_from_slice(b);
  out
}


