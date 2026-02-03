use eframe::egui::{ComboBox, Context, Event, Pos2, Window};
use num2words::{Num2Words, Lang::English};

use super::common::*;
use crate::state::{PackedState, Classification, frequency_analysis, parse_analysis};

pub struct Analyze {
  viewing_type: usize,
  viewing_length: usize,
  viewing: StrType<usize>,
  parsed_analysis: Vec<Vec<u32>>,
}
impl Analyze {
  fn draw_analysis_window(&self, ctx: &Context) {
    Window::new("Analysis")
      .default_pos(Pos2::new(0., 150.))
      .show(ctx, |ui| {
        for (value, values) in self.parsed_analysis.iter().enumerate() {
          for (node_count, state_count) in values.iter().enumerate() {
            ui.label(&format!(
              "{state_count} {} {} {value}{}",
              if *state_count == 1 {"state has"} else {"states have"},
              Num2Words::new(node_count as f32).lang(English).to_words().unwrap(),
              if node_count == 1 { "" } else {"s"}
            ));
          }
        }
    });
  }
}
impl super::Mode for Analyze {

  fn create(program: &GraphProgram) -> Self {

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
    ui.label(&format!("{total} Total"));

    // Identify view type
    let old_type = self.viewing_type;
    let names = [
      "All Invalid",
      "Bad States",
      "NotBad States",
      "All Valid",
    ];
    ComboBox::from_label("Type").selected_text(format!("{}", names[old_type]))
      .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.viewing_type, 0, names[0]);
        ui.selectable_value(&mut self.viewing_type, 1, names[1]);
        ui.selectable_value(&mut self.viewing_type, 2, names[2]);
        ui.selectable_value(&mut self.viewing_type, 3, names[3]);
      })
    ;
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
    ui.horizontal(|ui| {
      TextEdit::singleline(self.viewing.string_mut())
        .id(Id::new("Viewing"))
        .show(ui)
      ;
      self.viewing.parse();

      ui.label(format!("/{} Viewed States", self.viewing_length));
    });

    if self.parsed_analysis.is_empty() || old_type != self.viewing_type {
      let analysis = frequency_analysis(focused_states, state_space.length(), program.max.val());
      self.parsed_analysis = parse_analysis(analysis, program.max.val(), state_space.length() as u8);
    }

    // Load current viewing state
    if let Some(state) = focused_states.get(self.viewing.val().saturating_sub(1)) {
      program.desired_state = *state;
    }

    self.draw_analysis_window(ui.ctx());
  }

  fn interactions(&mut self, _program: &mut GraphProgram, response: Response) {

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
    });
  
    if self.viewing_length != 0 && (right_pressed || left_pressed) {
      self.viewing.step_strnum(self.viewing_length, 1, right_pressed);
    }
    
    if up_pressed {
      self.viewing_type = if self.viewing_type == 0 { 3 } else { self.viewing_type - 1 };
    }
    
    if down_pressed {
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

