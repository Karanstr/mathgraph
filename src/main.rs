mod graph; mod state;
use macroquad::prelude::*;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;

mod edit_functions;
use edit_functions::*;

use crate::state::{Classification, StateData, PackedState};
fn combine<'a>(a: &'a [PackedState], b: &'a [PackedState]) -> Vec<PackedState> {
  let mut out = Vec::with_capacity(a.len() + b.len());
  out.extend_from_slice(a);
  out.extend_from_slice(b);
  out
}

struct Nodes {
  hovering: Option<usize>,
  selected: Option<usize> 
}

// Implement all UI based functions in this ui struct
struct UiData {
  radius_str: String,
  radius: f32,
  edit_mode: usize,
  modify_str: String,
  max_str: String,
}
impl UiData {
  pub fn new() -> Self {
    Self {
      radius_str: "40.0".to_string(),
      radius: 40.,
      edit_mode: 0,
      modify_str: "1".to_string(),
      max_str: "2".to_string(),
    }
  }

  pub fn parse_radius(&mut self) {
    if let Ok(radius) = self.radius_str.parse::<f32>() { self.radius = radius };
  }

}

#[macroquad::main("Graph Visualizer")]
async fn main() {

  let mut ui_data = UiData::new();

  let mut state_data = None;

  let mut graph = Graph::new();
  let mut nodes = Nodes { hovering: None, selected: None};

  let mut current_view_str = "1".to_string();
  let mut viewing = 0;
  // let mut parsed_analysis = Vec::new();

  loop {
    // Controls Window
    widgets::Window::new(hash!(), vec2(0., 0.), vec2(250., 150.))
      .label("Settings")
      .ui(&mut *root_ui(), |ui| {
        ui.input_text(hash!(), "Radius", &mut ui_data.radius_str);
        ui.input_text(hash!(), "Max", &mut ui_data.max_str);
        ui.combo_box(hash!(), "Mode", &[
          "Add/Connect", // 0
          "Remove",      // 1
          "Move",        // 2
          "Modify",      // 3
          "Set",         // 4
          "Analyze",      // 5
        ], &mut ui_data.edit_mode);
        if ui_data.edit_mode == 0 || ui_data.edit_mode == 1 {
          state_data = None;
          // parsed_analysis.clear();
          current_view_str = "1".to_string();
        }
        if ui_data.edit_mode == 3 { ui.input_text(hash!(), "Delta", &mut ui_data.modify_str); }
        if ui_data.edit_mode == 4 { ui.input_text(hash!(), "Value", &mut ui_data.modify_str); }
        if ui_data.edit_mode == 5 {

          let max = ui_data.max_str.parse::<u8>().unwrap_or_default();

          // Compute new state data
          let mut just_pressed = false;
          if ui.button(Vec2::new(5., 110.), "GO") {
            state_data = StateData::new(&mut graph, max + 1);
            just_pressed = true;
          }
          if let Some(state_space) = &mut state_data {

            let total = (state_space.base as usize).pow(state_space.length as u32);
            ui.label(Vec2::new(30., 110.), &format!("{total} Total"));

            let old_viewing = viewing;
            ui.combo_box(hash!(), "Mode", &[
              "All Invalid",    // 0
              "Bad States",     // 1
              "NotBad States",  // 2
              "All Valid",      // 3
            ], &mut viewing);
            let focused_states = match viewing {
              0 => &combine(
                state_space.get_list(Classification::InvalidOther), 
                state_space.get_list(Classification::InvalidT1)
              ),
              1 => state_space.get_list(Classification::InvalidT1),
              2 => state_space.get_list(Classification::InvalidOther),
              3 => state_space.get_list(Classification::Valid),
              _ => unreachable!()
            };
            // Load first view
            if old_viewing != viewing || just_pressed {
              if !focused_states.is_empty() {
                let current_viewed = current_view_str.parse::<usize>().unwrap_or_default();
                if let Some(state) = focused_states.get(current_viewed.saturating_sub(1)) {
                  let new_state = state_space.parse_state(*state);
                  for idx in 0 .. graph.nodes.len() {
                    graph.nodes.get_mut(idx).unwrap().value = new_state[idx];
                  }
                }
              }
              // let analysis = frequency_analysis(focused_states, max);
              // parsed_analysis = parse_analysis(analysis, max, graph.nodes.len() as u8);
            }

            ui.input_text(
              hash!(),
              &format!("/{} Viewed States", focused_states.len()),
              &mut current_view_str
            );

            // Load current viewing state
            if let Ok(idx) = current_view_str.parse::<usize>()
            && let Some(state) = focused_states.get(idx.saturating_sub(1)) {
              let new_state = state_space.parse_state(*state);
              for idx in 0 .. graph.nodes.len() {
                graph.nodes.get_mut(idx).unwrap().value = new_state[idx];
              }
            }
          }
          


          // Analysis Window
          // widgets::Window::new(hash!(), vec2(0., 150.), vec2(250., 200.))
            // .label("Analysis")
          //   .ui(ui, |ui| {
          //     let mut y = 0.;
          //     for (value, values) in parsed_analysis.iter().enumerate() {
          //       for (node_count, state_count) in values.iter().enumerate() {
          //         ui.label(Vec2::new(0., y),
          //         &format!("{state_count} {} {} {value}{}",
          //           if *state_count == 1 {"state has"} else {"states have"},
          //           Num2Words::new(node_count as f32).lang(English).to_words().unwrap(),
          //           if node_count == 1 { "" } else {"s"}
          //         )
          //       );
          //       y += 10.;
          //
          //       }
          //     }
          //
          // });

        }
    });

    ui_data.parse_radius();

    if !root_ui().is_mouse_over(mouse_position().into()) {
      let mouse_pos = Vec2::from(mouse_position()).as_ivec2();
      nodes.hovering = graph.node_at(mouse_pos, ui_data.radius);
      match ui_data.edit_mode {
        0 => create_nodes_neighbors(&mut graph, &mut nodes, mouse_pos, ui_data.radius),
        1 => remove_node(&mut graph, &mut nodes),
        2 => drag_nodes(&mut graph, &mut nodes, mouse_pos),
        3 => modify(&mut graph, &mut nodes, &ui_data.modify_str, &ui_data.max_str),
        4 => set(&mut graph, &mut nodes, &ui_data.modify_str, &ui_data.max_str),
        _ => (),
      }
    }

    graph.render(ui_data.radius);

    next_frame().await
  }
}


// Returns a count of how many of each node value each state has
// Per state, how many nodes have a value
// result[state][value] = node_count
fn frequency_analysis(states: &Vec<Vec<u8>>, max: u8) -> Vec<Vec<u32>> {
  if states.is_empty() { return Vec::new() }
  let mut result = Vec::new();
  for state in states {
    let mut count = vec![0; max as usize + 1];
    for node in 0 .. state.len() { count[state[node] as usize] += 1; }
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

