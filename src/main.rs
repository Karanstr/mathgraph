mod graph; mod state; mod utilities;

use std::mem::discriminant;
use macroquad::prelude::*;
use macroquad::input::KeyCode as RKeyCode;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;
use state::*;
use utilities::*;

const NODE_RADIUS: f32 = 40.;

struct GraphProgram {
  graph: Graph,
  state_space: Option<StateData>,
  mode: UserMode,
  max: StrType<u8>,

  loaded_state: PackedState,
  desired_state: PackedState,
}
impl GraphProgram {
  pub fn new() -> Self {
    Self {
      state_space: None,
      graph: Graph::new(),
      mode: UserMode::AddRemove { selected: None },
      max: StrType::new(2),
      loaded_state: 0,
      desired_state: 0,
    }
  }

  pub async fn run(mut self) { loop {

    self.settings_window();
    
    if self.desired_state != self.loaded_state
      && let Some(state_space) = &self.state_space
    {
      self.graph.load_state(state_space.parse_state(self.desired_state));
      self.loaded_state = self.desired_state;
    }

    // We can only interact with the canvas when we aren't hovering ui
    if !root_ui().is_mouse_over(mouse_position().into()) { self.handle_interactions(); }

    if matches!(self.mode, UserMode::Analyze {..}) {
      self.draw_analysis_window(&mut root_ui());
    }

    self.graph.render(NODE_RADIUS, RED);
    
    next_frame().await
  } }

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
    if self.state_space.is_some() {
      self.state_space = StateData::new(&mut self.graph, self.max.val());
      if let Some(state_space) = &self.state_space {
        self.loaded_state = state_space.parse_vec(self.graph.export_state());
        self.desired_state = self.loaded_state;
      }
    }
  }

  fn set_mode(&mut self, ui: &mut Ui) {
    let mut new_mode = self.mode.as_int();
    ui.combo_box(hash!(), "Mode", &[
      "Add/Remove",
      "Drag",
      "Play",
      "Set",
      "Analyze",
      "Bubbles",
    ], &mut new_mode);

    let potential_mode = UserMode::from_int(new_mode);
    if discriminant(&self.mode) != discriminant(&potential_mode) { 
      self.mode = potential_mode
    };

    if matches!(&self.mode, UserMode::AddRemove { .. }) {
      self.state_space = None;
    } else {
      if self.state_space.is_none() && self.graph.node_count() != 0 {
        self.state_space = StateData::new(&mut self.graph, self.max.val());
        if let Some(state_space) = &self.state_space {
          self.loaded_state = state_space.parse_vec(self.graph.export_state());
          self.desired_state = self.loaded_state;
        }
      }
    }

  }

  fn handle_mode_ui(&mut self, ui: &mut Ui) {
    'mode: {match &mut self.mode {
      UserMode::Set { value} => {

        ui.input_text(hash!(), "Value", value.string_mut());
        value.parse();

      }
      UserMode::Play { allow_clamping} => {

        ui.checkbox(hash!(), "Allow Clamping", allow_clamping);
        
      }
      UserMode::Analyze {
        viewing_type,
        viewing_length,
        viewing,
        parsed_analysis
      } => {

        let Some(state_space) = self.state_space.as_ref() else { break 'mode };

        let total = (state_space.base as usize).pow(state_space.length() as u32);
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
          let analysis = frequency_analysis(focused_states, self.graph.nodes.len(), self.max.val());
          *parsed_analysis = parse_analysis(analysis, self.max.val(), self.graph.nodes.len() as u8);
        }

        // Load current viewing state
        if let Some(state) = focused_states.get(viewing.val().saturating_sub(1)) {
          self.desired_state = *state;
        }

      },
      UserMode::Bubbles {
        bubble,
        bubble_length,
        state,
        state_length,
      } => {
        
        let Some(state_space) = self.state_space.as_ref() else { break 'mode };

        let old_bubble_idx = bubble.val();
        // Identify bubble
        ui.input_text(
          hash!(),
          &format!("/{} Viewed Bubbles", state_space.bubbles.len()),
          bubble.string_mut()
        );
        bubble.parse();
        if bubble.val() != old_bubble_idx { state.assign(1); }
        *bubble_length = state_space.bubbles.len();
        
        let Some(bubble_vec) = state_space.bubbles.get(bubble.val().saturating_sub(1)) else {
          break 'mode;
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
          self.desired_state = *state;
        }
      
      },
      _ => {}
    }}

    if let Some(state_space) = &self.state_space {
      if let Some(metadata) = state_space.meta.get(&self.loaded_state) {
        let display = match metadata.classification() {
          Classification::Valid => { "Valid" },
          Classification::InvalidT1 => { "Invalid, Theorem 1" },
          Classification::InvalidOther => { "Invalid, Unknown Theorem" },
        };
        ui.label(Vec2::new(0., 85.), display);
      }
    }

  }

  fn draw_analysis_window(&self, ui: &mut Ui) {
    if let UserMode::Analyze { parsed_analysis, .. } = &self.mode {
      widgets::Window::new(hash!("Analysis"), vec2(0., 150.), vec2(250., 200.))
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
      UserMode::Play { allow_clamping } => {

        let delta = 
          if is_mouse_button_pressed(MouseButton::Left) { 1 }
          else if is_mouse_button_pressed(MouseButton::Right) { -1 }
          else { return } as i8
        ;
        
        if   let Some(node) = hovering
          && let Some(state_space) = &self.state_space
          && let Some(state) = state_space.splash_state(
            self.loaded_state,
            node,
            delta,
            *allow_clamping
          )
        {
          self.desired_state = state;
        }

      },
      UserMode::Set { value, .. } => {

        if let Some(node) = hovering 
          && value.val() <= self.max.val()
          && is_mouse_button_pressed(MouseButton::Left)
          && let Some(state_space) = &self.state_space
        {
          self.desired_state = state_space.set_packed(self.loaded_state, node, value.val());
        }

      },
      UserMode::Analyze {
        viewing,
        viewing_length,
        ..
      } => { 
        
        if *viewing_length != 0 {
          viewing.step_strnum(*viewing_length, 1, RKeyCode::Right, RKeyCode::Left);
        }

      },
      UserMode::Bubbles {
        state,
        state_length,
        bubble,
        bubble_length,
      } => {

        if *state_length != 0 {
          state.step_strnum(*state_length, 1, RKeyCode::Right, RKeyCode::Left);
        }

        if *bubble_length != 0 {
          bubble.step_strnum(*bubble_length, 1, RKeyCode::Up, RKeyCode::Down);
        }

      }
    }
  }

  fn get_mouse_pos() -> IVec2 { Vec2::from(mouse_position()).as_ivec2() }

  fn get_hovering(&self) -> Option<usize> {
    self.graph.node_at(Self::get_mouse_pos(), NODE_RADIUS)
  }

}

#[macroquad::main("Graph Visualizer")]
async fn main() { GraphProgram::new().run().await; }
