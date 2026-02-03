mod graph; mod state; mod utilities; mod mode;

use std::mem::take;
use eframe::App;
use eframe::egui::{Align2, CentralPanel, Color32, ComboBox, Context, FontId, Id, LayerId, Order, Painter, Pos2, Sense, Stroke, Ui, Window};
use graph::Graph;
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
  
  fn get_node_at(&self, pos: Pos2) -> Option<usize> {
    self.graph.node_at(pos, NODE_RADIUS)
  }
}
impl GraphProgram {

  fn settings_window(&mut self, ctx: &Context) {
    Window::new("Settings").show(ctx, |ui| {
      self.handle_max(ui);
      self.set_mode(ui);
      self.handle_mode_ui(ui);
    });
  }

  // I don't like directly touching the graph like this, but if I don't then max can't be changed
  // during add/remove (or I need edgecases)
  fn handle_max(&mut self, ui: &mut Ui) {
    ui.text_edit_singleline(self.max.string_mut());
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
    ComboBox::from_label("Mode").selected_text(format!("{}", self.mode))
      .show_ui(ui, |ui| {
        ui.selectable_value(&mut new_mode, 0, "Add/Remove");
        ui.selectable_value(&mut new_mode, 1, "Drag");
        ui.selectable_value(&mut new_mode, 2, "Play");
        ui.selectable_value(&mut new_mode, 3, "Set");
        ui.selectable_value(&mut new_mode, 4, "Analyze");
        ui.selectable_value(&mut new_mode, 5, "Bubbles");
      })
    ;

    if self.mode.as_int() == new_mode { return; }
    let mode = Modes::new(&self, new_mode);
    self.mode = mode;

    if !matches!(&self.mode, Modes::AddRemove(_)) && self.state_space.is_none() {
      
      self.state_space = StateData::new(&mut self.graph, self.max.val());
      if let Some(state_space) = &self.state_space {
        self.loaded_state = state_space.parse_vec(self.graph.export_state());
        self.desired_state = self.loaded_state;
        self.graph_changed = true;
      }

    } else if matches!(&self.mode, Modes::AddRemove(_)) && self.state_space.is_some() {
      self.state_space = None;
      self.graph_changed = true;
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
      ui.label(display);
    }

  }

  fn color_nodes(&mut self) {
    if let Some(state_space) = &self.state_space {
      for (idx, node) in self.graph.nodes.iter_mut() {
        let (has_zero, has_max) = state_space.neighborhood_zero_or_max(self.loaded_state, idx);
        node.color = 
          if has_zero && has_max { Color32::RED } // Can't move
          else if has_zero { Color32::ORANGE } // Can go up
          else if has_max { Color32::DARK_BLUE } // Can go down
          else { Color32::DARK_GREEN } // Free
        ;
      }
    } else {
      for (_, node) in self.graph.nodes.iter_mut() {
        node.color = Color32::RED;
      }
    }

  }

  fn draw_graph(&self, ui: &mut Ui) {
    let lines = Painter::new(ui.ctx().clone(), LayerId::new(Order::Background, Id::new("Lines")), ui.clip_rect());
    let nodes = Painter::new(ui.ctx().clone(), LayerId::new(Order::Middle, Id::new("Nodes")), ui.clip_rect());
    for (current, node) in self.graph.nodes.iter() {
      nodes.circle_filled(node.position, NODE_RADIUS, node.color);
      nodes.text(
        node.position,
        Align2::CENTER_CENTER,
        &format!("{}", node.value),
        FontId::new(NODE_RADIUS, eframe::egui::FontFamily::Proportional),
        Color32::WHITE
      );

      for neighbor in &node.neighbors {
        if *neighbor > current {
          lines.line_segment(
            [node.position, self.graph.nodes.get(*neighbor).unwrap().position],
            Stroke::new(4., Color32::WHITE)
          );
        }
      }
    }
  }

  fn main_frame(&mut self, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
      let response = ui.allocate_rect(ui.clip_rect(), Sense::click_and_drag());

      let mut mode = take(&mut self.mode);
      mode.interactions(self, response);
      self.mode = mode;

      if self.desired_state != self.loaded_state
        && let Some(state_space) = &self.state_space
      {
        self.graph.load_state(state_space.parse_state(self.desired_state));
        self.loaded_state = self.desired_state;
        self.graph_changed = true;
      }

      if self.graph_changed { self.color_nodes(); self.graph_changed = false; }
      self.draw_graph(ui);
    });
  }

}
impl App for GraphProgram {
  fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
    let mut mode = take(&mut self.mode);
    mode.tick(self);
    self.mode = mode;

    self.settings_window(ctx);

    self.main_frame(ctx);

  }
}

fn main() {
  let mut native_options = eframe::NativeOptions::default();
  native_options.viewport = native_options.viewport.with_title("Graph Application v3.0.0");
  let _ = eframe::run_native(
    "GraphAnalysis",
    native_options,
    Box::new(|_cc| Ok(Box::new(GraphProgram::new())))
  );
}
