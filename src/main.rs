mod graph; mod state; mod utilities;

use std::mem::discriminant;
use macroquad::prelude::*;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;
use state::*;
use utilities::*;

const NODE_RADIUS: f32 = 40.;

struct GraphProgram {
  state_space: Option<StateData>,
  graph: Graph,
  mode: UserMode,
  max: StrType<u8>,
  current_state: PackedState,
}
impl GraphProgram {
  pub fn new() -> Self {
    Self {
      state_space: None,
      graph: Graph::new(),
      mode: UserMode::AddRemove { selected: None },
      max: StrType::new(2),
      current_state: 0
    }
  }

  pub async fn run(mut self) {
    loop {

      widgets::Window::new(hash!(), vec2(0., 0.), vec2(250., 150.))
        .label("Settings")
        .ui(&mut *root_ui(), |ui| 
      {

        self.handle_max(ui);

        self.set_mode(ui);

        self.handle_mode_ui(ui);

        // We can only interact with the canvas when we aren't hovering ui
        if !ui.is_mouse_over(mouse_position().into()) { self.handle_interactions(); }

      });

      let mut is_good = true;
      if let Some(state_space) = &self.state_space {
        if let Some(metadata) = state_space.meta.get(&self.current_state) {
          if discriminant(&metadata.classification()) != discriminant(&Classification::Valid) {
            is_good = false;
          }
        }

      }

      self.graph.render(NODE_RADIUS, is_good);
      
      next_frame().await
    }
  }

  fn handle_max(&mut self, ui: &mut Ui) {
    ui.input_text(hash!(), "Max", self.max.string_mut());
    self.graph.correct_max(self.max.parse());
  }

  fn set_mode(&mut self, ui: &mut Ui) {
    let mut cur_mode = self.mode.as_int();
    ui.combo_box(hash!(), "Mode", &[
      "Add/Remove",
      "Drag",
      "Play",
      "Set",
      "Analyze",
      "Bubbles",
    ], &mut cur_mode);

    let potential_mode = UserMode::from_int(cur_mode);
    if discriminant(&self.mode) != discriminant(&potential_mode) { self.mode = potential_mode };
  }

  // We're doing extra work here by reloading current state every frame, ideally we could extract
  // this code into an update_graph function, but for now it's not enough for me to care
  fn handle_mode_ui(&mut self, ui: &mut Ui) {
    match &mut self.mode {
      UserMode::Set { value, val_str} => {

        ui.input_text(hash!(), "Value", val_str);
        *value = val_str.parse().unwrap_or(*value);

      }
      UserMode::Analyze {
        viewing_type,
        viewing_length,
        viewing,
        parsed_analysis
      } => {

        if self.graph.node_count() == 0 { return }

        if self.state_space.is_none() {
          self.state_space = StateData::new(&mut self.graph, self.max.val() + 1);
        }
        let state_space = self.state_space.as_ref().unwrap();

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

        *viewing_length = focused_states.len();
        
        // Identify view idx
        ui.input_text(
          hash!(),
          &format!("/{} Viewed States", focused_states.len()),
          viewing.string_mut()
        );
        viewing.parse();

        if parsed_analysis.is_empty() || old_type != *viewing_type {
          let analysis = frequency_analysis(focused_states, self.graph.node_count(), self.max.val());
          *parsed_analysis = parse_analysis(analysis, self.max.val(), self.graph.nodes.len() as u8);
        }

        // Load current viewing state
        if let Some(state) = focused_states.get(viewing.val().saturating_sub(1)) {
          self.graph.load_state(state_space.parse_state(*state));
          self.current_state = *state;
        }
      
        self.draw_analysis_window(ui);

      },
      UserMode::Bubbles {
        bubble,
        state,
        state_length,
      } => {
        
        if self.graph.node_count() == 0 { return }

        if self.state_space.is_none() {
          self.state_space = StateData::new(&mut self.graph, self.max.val() + 1);
        }
        let state_space = self.state_space.as_ref().unwrap();

        // Identify bubble
        ui.input_text(
          hash!(),
          &format!("/{} Viewed Bubbles", state_space.bubbles.len()),
          bubble.string_mut()
        );

        let old_bubble_idx = bubble.val();
        bubble.parse();
        if bubble.val() != old_bubble_idx { state.assign(1); }
        
        let Some(bubble_vec) = state_space.bubbles.get(bubble.val().saturating_sub(1)) else {
          return;
        };
        *state_length = bubble_vec.len();

        // Identify view idx
        ui.input_text(
          hash!(),
          &format!("/{} Viewed States", bubble_vec.len()),
          state.string_mut()
        );
        state.parse();

        if bubble.val() == state_space.bubbles.len() {
          ui.label(Vec2::new(0., 100.), "Bubble of Size 1 Bubbles");
        }

        // Load current viewing state
        if let Some(state) = bubble_vec.get(state.val().saturating_sub(1)) {
          self.graph.load_state(state_space.parse_state(*state));
          self.current_state = *state;
        }
      
      },
      _ => {}
    }
  }

  fn draw_analysis_window(&self, ui: &mut Ui) {
    if let UserMode::Analyze { parsed_analysis, .. } = &self.mode {
      widgets::Window::new(hash!(), vec2(0., 150.), vec2(250., 200.))
        .label("Analysis")
        .ui(ui, |ui| 
      {
        let mut y = 0.;

        for (value, values) in parsed_analysis.iter().enumerate() {
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

  fn handle_interactions(&mut self) {
    // Silly borrow issue
    let hovering = self.get_hovering();
    
    match &mut self.mode {
      UserMode::AddRemove { selected } => {

        // If we're changing the structure of the graph our statespace shifts.
        self.state_space = None;
        self.current_state = 0;
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

        if is_mouse_button_pressed(MouseButton::Left) { *selected = hovering; }
        if is_mouse_button_released(MouseButton::Left) { *selected = None; }

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

        if let Some(node) = hovering && delta != 0 {
          self.graph.clamped_update(node, delta, self.max.val());
        }

      },
      UserMode::Set { value, .. } => {

        if let Some(node) = hovering 
          && *value <= self.max.val()
          && is_mouse_button_pressed(MouseButton::Left)
        {
          self.graph.nodes.get_mut(node).unwrap().value = *value;
        }

      },
      UserMode::Analyze {
        viewing,
        viewing_length,
        ..
      } => { 

        viewing.step_strnum(*viewing_length, 1);

      },
      UserMode::Bubbles {
        state,
        state_length,
        ..
      } => {

        state.step_strnum(*state_length, 1);

      }
      _ => {}
    }
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
