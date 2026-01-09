mod graph; mod state;
use std::mem::discriminant;

use macroquad::prelude::*;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;
use state::*;

const NODE_RADIUS: f32 = 40.;

enum UserMode {
  AddRemove { selected: Option<usize> },
  Drag { selected: Option<usize> },
  Play,
  Set { value: u8 },
  Analyze {
    viewing_type: usize,
    viewing_idx: usize,
    idx_string: String,
    parsed_analysis: Vec<Vec<u32>>,
  }
}
impl UserMode {
  fn as_int(&self) -> usize {
    match self {
      Self::AddRemove {..} => 0,
      Self::Drag {..} => 1,
      Self::Play => 2,
      Self::Set {..} => 3,
      Self::Analyze {..} => 4,
    }
  }

  fn from_int(val: usize) -> Self {
    match val {
      0 => UserMode::AddRemove { selected: None },
      1 => UserMode::Drag { selected: None },
      2 => UserMode::Play,
      3 => UserMode::Set { value: 0 },
      4 => UserMode::Analyze { 
        viewing_type: 0,
        viewing_idx: 1,
        idx_string: "1".to_string(),
        parsed_analysis: Vec::new()
      },
      _ => unreachable!()
    }
  }
}

struct GraphProgram {
  data: Option<StateData>,
  graph: Graph,
  mode: UserMode,
  max: u8,
}
impl GraphProgram {
  pub fn new() -> Self {
    Self {
      data: None,
      graph: Graph::new(),
      mode: UserMode::AddRemove { selected: None },
      max: 2,
    }
  }

  pub async fn run(mut self) {
    loop {
      widgets::Window::new(hash!(), vec2(0., 0.), vec2(250., 150.)).label("Settings")
        .ui(&mut *root_ui(), |ui| {
          
          let mut max_str = self.max.to_string();
          ui.input_text(hash!(), "Max", &mut max_str);

          let mut cur_mode = self.mode.as_int();
          ui.combo_box(hash!(), "Mode", &[
            "Add/Remove",
            "Drag",
            "Play",
            "Set",
            "Analyze",
          ], &mut cur_mode);

          let potential_mode = UserMode::from_int(cur_mode);
          if discriminant(&self.mode) != discriminant(&potential_mode) { self.mode = potential_mode };

          if matches!(self.mode, UserMode::Analyze { .. } ) { self.handle_analysis(ui); }

          // We can only interact with the canvas when we aren't hovering ui
          if !ui.is_mouse_over(mouse_position().into()) { self.handle_interactions(ui); }

        });

      self.graph.render(NODE_RADIUS);

      next_frame().await
    }
  }

  // I don't really wanna make separate functions but I probably will.
  fn handle_interactions(&mut self, ui: &mut Ui) {
    // Silly borrow issue
    let hovering = self.get_hovering();
    
    match &mut self.mode {
      UserMode::AddRemove { selected } => {

        // If we're changing the structure of the graph our statespace shifts.
        self.data = None;
        let mouse_pos = Self::get_mouse_pos();

        // Delete hovering on right click
        if is_mouse_button_down(MouseButton::Right) {
          if let Some(remove) = hovering { 
            self.graph.remove(remove);
          }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
          // Either we select the node we're hovering
          *selected = hovering;
          // Or we create a node
          if selected.is_none() {
            *selected = Some(self.graph.add_node(mouse_pos));
          }
          // Or do nothing if we're not touching it but too close to make one??
        }

        // Draw line from selected node to mouse
        if is_mouse_button_down(MouseButton::Left) && let Some(node) = selected {
          let origin = self.graph.nodes.get(*node).unwrap().position;
          draw_line(
            mouse_pos.x as f32, mouse_pos.y as f32,
            origin.x as f32, origin.y as f32,
            4., WHITE
          );
        }

        if is_mouse_button_released(MouseButton::Left) {
          if let Some(node1) = *selected && let Some(node2) = hovering && node1 != node2 {
            self.graph.attempt_unique_connection(node1, node2);
          }
          *selected = None;
        }

      },
      UserMode::Drag { selected} => {

        if is_mouse_button_pressed(MouseButton::Left) {
          *selected = hovering;
        }
        if is_mouse_button_released(MouseButton::Left) {
          *selected = None;
        }
        
        if let Some(dragging) = *selected {
          self.graph.nodes.get_mut(dragging).unwrap().position = Self::get_mouse_pos();
        }

      },
      UserMode::Play => {

        let delta = 
          if is_mouse_button_pressed(MouseButton::Left) { 1 }
          else if is_mouse_button_pressed(MouseButton::Right) { -1 }
          else { 0 } as i8
        ;

        if let Some(node) = self.get_hovering() {
          self.graph.clamped_update(node, delta, self.max);
        }

      },
      UserMode::Set { value } => {

        let mut set_val = value.to_string();
        ui.input_text(hash!(), "Value", &mut set_val);

        if let Ok(value) = set_val.parse::<u8>()
          && let Some(node) = self.get_hovering()
          && value <= self.max
        { self.graph.nodes.get_mut(node).unwrap().value = value; }

      },
      UserMode::Analyze { .. } => { }
    }
  }

  fn handle_analysis(&mut self, ui: &mut Ui) {
    
    let UserMode::Analyze { 
      ref mut viewing_type,
      ref mut viewing_idx,
      ref mut idx_string,
      ref mut parsed_analysis
    } = self.mode else { return; };
      
    // Compute new state data
    let just_generated = ui.button(Vec2::new(5., 110.), "GO");
    if just_generated { self.data = StateData::new(&mut self.graph, self.max + 1); }
    // If we have valid states, display relevant ui elements
    if let Some(state_space) = &mut self.data {

      let total = (state_space.base as usize).pow(state_space.length as u32);
      ui.label(Vec2::new(30., 110.), &format!("{total} Total"));

      // Identify view type
      let old_type = *viewing_type;
      ui.combo_box(hash!(), "Mode", &[
        "All Invalid",    // 0
        "Bad States",     // 1
        "NotBad States",  // 2
        "All Valid",      // 3
      ], &mut *viewing_type);
      let focused_states = match *viewing_type {
        0 => &combine(
          state_space.get_list(Classification::InvalidOther), 
          state_space.get_list(Classification::InvalidT1)
        ),
        1 => state_space.get_list(Classification::InvalidT1),
        2 => state_space.get_list(Classification::InvalidOther),
        3 => state_space.get_list(Classification::Valid),
        _ => unreachable!()
      };
      
      // Identify view idx
      ui.input_text(
        hash!(),
        &format!("/{} Viewed States", focused_states.len()),
        idx_string
      );
      *viewing_idx = idx_string.parse().unwrap_or(*viewing_idx);

      // Load view
      if old_type != *viewing_type || just_generated {
        let analysis = frequency_analysis(focused_states, self.graph.node_count(), self.max);
        *parsed_analysis = parse_analysis(analysis, self.max, self.graph.nodes.len() as u8);
      }

      // Load current viewing state
      if let Some(state) = focused_states.get(viewing_idx.saturating_sub(1)) {
        let new_state = state_space.parse_state(*state);
        for idx in 0 .. self.graph.nodes.len() {
          self.graph.nodes.get_mut(idx).unwrap().value = new_state[idx];
        }
      }
    }
    


    // Analysis Window
    widgets::Window::new(hash!(), vec2(0., 150.), vec2(250., 200.))
      .label("Analysis")
      .ui(ui, |ui| {
        let mut y = 0.;
        for (value, values) in parsed_analysis.iter().enumerate() {
          for (node_count, state_count) in values.iter().enumerate() {
            ui.label(Vec2::new(0., y),
            &format!("{state_count} {} {} {value}{}",
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

  fn get_mouse_pos() -> IVec2 { Vec2::from(mouse_position()).as_ivec2() }

  fn get_hovering(&self) -> Option<usize> {
    self.graph.node_at(Self::get_mouse_pos(), NODE_RADIUS)
  }

}



// Returns a count of how many of each node value each state has
// Per state, how many nodes have a value
// result[state][value] = node_count
fn frequency_analysis(states: &Vec<PackedState>, length: usize, max: u8) -> Vec<Vec<u32>> {
  if states.is_empty() { return Vec::new() }
  let mut result = Vec::new();
  let base = max as usize + 1;
  for state in states {
    let mut count = vec![0; base];
    for idx in 0 .. length { 
      count[StateOps::get(*state, idx, base as u8, length) as usize] += 1;
    }
    result.push(count);
  }
  result
}

// How many states have 3 nodes with value 1, etc
// result[value][node_count] = state_count
fn parse_analysis(analysis: Vec<Vec<u32>>, max: u8, node_count: u8) -> Vec<Vec<u32>> {
  if analysis.is_empty() { return Vec::new(); }
  let mut result = vec![vec![0; node_count as usize + 1]; max as usize + 1];

  for state in analysis {
    for (value, node_count) in state.iter().enumerate() {
      result[value][*node_count as usize] += 1; 
    }
  }
  result
}

#[macroquad::main("Graph Visualizer")]
async fn main() { GraphProgram::new().run().await; }
