use std::ops::RangeInclusive;

use eframe::egui::{ComboBox, Context, Event, Pos2, Window};
use num2words::{Num2Words, Lang::English};

use super::common::*;
use crate::state::{PackedState, Classification, frequency_analysis, parse_analysis};

#[derive(Debug)]
pub struct Analyze {
  viewing_type: usize,
  viewing_length: usize,
  viewing: usize,
  parsed_analysis: Vec<Vec<u32>>,
}
impl Analyze {
  fn draw_analysis_window(&self, ctx: &Context) {
    Window::new("Analysis")
      .default_pos(Pos2::new(15., 200.))
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
    } else { (3, 0) };

    Self {
      viewing_type,
      viewing_length: 0,
      // Wow I hate that I did this off by one nonsense
      viewing: idx + 1,
      parsed_analysis: Vec::new(),
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {

    let Some(state_space) = program.state_space.as_ref() else { return };

    // Identify view type
    let old_type = self.viewing_type;
    let names = [
      "All Valid",
      "Lonely States",
      "Other Invalid",
      "All Invalid",
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
      0 => state_space.get_list(Classification::Valid),
      1 => state_space.get_list(Classification::InvalidT1),
      2 => state_space.get_list(Classification::InvalidOther),
      3 => &combine(
        state_space.get_list(Classification::InvalidOther), 
        state_space.get_list(Classification::InvalidT1)
      ),
      _ => unreachable!()
    };

    self.viewing_length = focused_states.len();

    // Identify view idx
    ui.horizontal(|ui| {
      DragValue::new(&mut self.viewing)
        .range(RangeInclusive::new(1, self.viewing_length))
        .speed(0.1)
        .ui(ui)
      ;
      ui.label(format!("/{} Viewed States", self.viewing_length));
    });

    if self.parsed_analysis.is_empty() || old_type != self.viewing_type {
      let analysis = frequency_analysis(focused_states, state_space.length(), program.max);
      self.parsed_analysis = parse_analysis(analysis, program.max, state_space.length() as u8);
    }

    // Load current viewing state
    if let Some(state) = focused_states.get(self.viewing - 1) {
      program.desired_state = *state;
    }

    let total = (state_space.base as usize).pow(state_space.length() as u32);
    ui.label(&format!("{total} Total State Count"));

    self.draw_analysis_window(ui.ctx());
  }

  fn interactions(&mut self, _program: &mut GraphProgram, response: Response) {

    let mut up_pressed = false;
    let mut down_pressed = false;
    let mut left_pressed = false;
    let mut right_pressed = false;
    response.ctx.input(|input| {
      for event in &input.events {
        let Event::Key { key, pressed: true, repeat: false, ..} = event else { continue; };
        match *key {
          Key::ArrowDown => down_pressed = true,
          Key::ArrowUp => up_pressed = true,
          Key::ArrowLeft => left_pressed = true,
          Key::ArrowRight => right_pressed = true,
          _ => (),
        }
      }
    });
  
    if self.viewing_length != 0 {
      if right_pressed {
        self.viewing = self.viewing % self.viewing_length + 1;
      }
      if left_pressed {
        self.viewing = 
          if self.viewing == 1 || self.viewing > self.viewing_length { 
            self.viewing_length
          }
          else { self.viewing - 1 }
        ;
      }

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

