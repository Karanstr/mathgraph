mod graph; mod state; mod utilities; mod mode;

use std::mem::take;
use macroquad::prelude::*;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;
use state::*;
use utilities::*;

use crate::mode::Modes;

const NODE_RADIUS: f32 = 40.;

struct GraphProgram {
  graph: Graph,
  state_space: Option<StateData>,
  mode: Modes,
  max: StrType<u8>,
  graph_changed: bool,

  loaded_state: PackedState,
  desired_state: PackedState,
}
impl GraphProgram {
  pub fn new() -> Self {
    let mut program = Self {
      state_space: None,
      graph: Graph::new(),
      mode: Modes::default(),
      max: StrType::new(2),
      graph_changed: false,

      loaded_state: 0,
      desired_state: 0,
    };
    let mode = Modes::new(&mut program, 0);
    program.mode = mode;
    program
  }
  
  fn get_mouse_pos() -> IVec2 { Vec2::from(mouse_position()).as_ivec2() }

  fn get_hovering(&self) -> Option<usize> {
    self.graph.node_at(Self::get_mouse_pos(), NODE_RADIUS)
  }
}
impl GraphProgram {

  pub async fn run(mut self) { loop {

    let mut mode = take(&mut self.mode);
    mode.tick(&mut self);
    self.mode = mode;

    self.settings_window();
    
    if self.desired_state != self.loaded_state
      && let Some(state_space) = &self.state_space
    {
      self.graph.load_state(state_space.parse_state(self.desired_state));
      self.loaded_state = self.desired_state;
      self.graph_changed = true;
    }

    // We can only interact with the canvas when we aren't hovering ui
    if !root_ui().is_mouse_over(mouse_position().into()) { self.handle_interactions(); }

    if matches!(self.mode, Modes::Analyze(_)) { self.draw_analysis_window(); }

    if self.graph_changed { self.color_nodes(); }
    self.graph_changed = false;
    self.graph.render(NODE_RADIUS);
    
    next_frame().await

  }}

  fn settings_window(&mut self) {
    widgets::Window::new(hash!("Settings"), vec2(0., 0.), vec2(250., 150.))
      .label("Settings")
      .ui(&mut *root_ui(), |ui| 
    {

      self.handle_max(ui);

      self.set_mode(ui);

      self.handle_mode_ui(ui);
    
    });
  }

  // I don't like directly touching the graph like this, but if I don't then max can't be changed
  // during add/remove (or I need edgecases)
  fn handle_max(&mut self, ui: &mut Ui) {
    ui.input_text(hash!(), "Max", self.max.string_mut());
    let old_max = self.max.val();
    if old_max == self.max.parse() { return; }
    
    self.graph.correct_max(self.max.val());
    if let Some(state_space) = &mut self.state_space {
      *state_space = StateData::new(&mut self.graph, self.max.val()).unwrap();
      self.loaded_state = state_space.parse_vec(self.graph.export_state());
      self.desired_state = self.loaded_state;
    }
    self.graph_changed = true;
  }

  fn set_mode(&mut self, ui: &mut Ui) {
    let mut new_mode = self.mode.as_int();
    ui.combo_box(hash!(), "Mode", Modes::list_modes(), &mut new_mode);

    if self.mode.as_int() == new_mode { return; }
    let mode = Modes::new(self, new_mode);
    self.mode = mode;

    if !matches!(&self.mode, Modes::AddRemove(_)) && self.state_space.is_none() {
      self.state_space = StateData::new(&mut self.graph, self.max.val());
     
      if let Some(state_space) = &self.state_space {
        self.loaded_state = state_space.parse_vec(self.graph.export_state());
        self.desired_state = self.loaded_state;
        self.graph_changed = true;
      }
    }

  }

  fn handle_mode_ui(&mut self, ui: &mut Ui) {

    let mut mode = take(&mut self.mode);
    mode.ui(self, ui);
    self.mode = mode;

    if let Some(state_space) = &self.state_space {
      let display = match state_space.classification_data(self.loaded_state).0 {
        Classification::Valid => { "Valid" },
        Classification::InvalidT1 => { "Invalid, Theorem 1" },
        Classification::InvalidOther => { "Invalid, Unknown Theorem" },
      };
      ui.label(Vec2::new(0., 85.), display);
    }

  }

  fn handle_interactions(&mut self) {
    let mut mode = take(&mut self.mode);
    mode.interactions(self);
    self.mode = mode;
  }


  fn color_nodes(&mut self) {

    if let Some(state_space) = &self.state_space {
      for (idx, node) in self.graph.nodes.iter_mut() {
        let (has_zero, has_max) = state_space.neighborhood_zero_or_max(self.loaded_state, idx);
        node.color = 
          if has_zero && has_max { RED } // Can't move
          else if has_zero { ORANGE } // Can go up
          else if has_max { DARKBLUE } // Can go down
          else { DARKGREEN } // Free
        ;
      }
    } else {
      for (_, node) in self.graph.nodes.iter_mut() {
        node.color = RED;
      }
    }

  }


  fn draw_analysis_window(&self) {
    if let Modes::Analyze(analyze) = &self.mode {
      widgets::Window::new(hash!("Analysis"), vec2(0., 150.), vec2(250., 200.))
        .label("Analysis")
        .ui(&mut root_ui(), |ui| 
      {
        let mut y = 0.;

        for (value, values) in analyze.get_analysis().iter().enumerate() {
          for (node_count, state_count) in values.iter().enumerate() {
            ui.label(Vec2::new(0., y),
              &format!
              (
                "{state_count} {} {} {value}{}",
                if *state_count == 1 {"state has"} else {"states have"},
                Num2Words::new(node_count as f32).lang(English).to_words().unwrap(),
                if node_count == 1 { "" } else {"s"}
              )
            );

            y += 10.;
          }
        }

      });
    }
  }

}

#[macroquad::main("Graph Visualizer")]
async fn main() { GraphProgram::new().run().await; }
