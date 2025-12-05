mod graph;
use ahash::AHashSet;
use macroquad::prelude::*;
use graph::Graph;
use macroquad::ui::*;
use num2words::Lang::English;
use num2words::Num2Words;

mod edit_functions;
use edit_functions::*;

struct Nodes {
  hovering: Option<usize>,
  selected: Option<usize> 
}

#[macroquad::main("Graph Visualizer")]
async fn main() {
  let mut radius_str = "40.0".to_string();
  let mut radius = 40.;
  let mut graph = Graph::new();
  let mut nodes = Nodes { hovering: None, selected: None};

  let mut edit_mode = 0;
  let mut modify_val = "1".to_string();
  let mut max_str = "2".to_string();

  let mut state_table = AHashSet::new();
  let mut state_list = Vec::new();
  let mut invalid_states = Vec::new();
  let mut current_view_str = "1".to_string();
  let mut theorem_1_states = Vec::new();
  let mut not_t1_states = Vec::new();
  let mut viewing = 0;
  let mut parsed_analysis = Vec::new();

  loop {
    // Controls Window
    widgets::Window::new(hash!(), vec2(0., 0.), vec2(250., 150.))
      .label("Settings")
      .ui(&mut *root_ui(), |ui| {
        ui.input_text(hash!(), "Radius", &mut radius_str);
        ui.input_text(hash!(), "Max", &mut max_str);
        ui.combo_box(hash!(), "Mode", &[
          "Add/Connect", // 0
          "Remove",      // 1
          "Move",        // 2
          "Modify",      // 3
          "Set",         // 4
          "Analyze",      // 5
        ], &mut edit_mode);
        if edit_mode == 0 || edit_mode == 1 {
          state_table.clear();
          state_list.clear();
          invalid_states.clear();
          theorem_1_states.clear();
          not_t1_states.clear();
          parsed_analysis.clear();
          current_view_str = "1".to_string();
        }
        if edit_mode == 3 { ui.input_text(hash!(), "Delta", &mut modify_val); }
        if edit_mode == 4 { ui.input_text(hash!(), "Value", &mut modify_val); }
        if edit_mode == 5 {

          let max = max_str.parse::<u8>().unwrap_or_default();

          // Compute new state data
          let mut just_pressed = false;
          if ui.button(Vec2::new(5., 110.), "GO") {
            state_table = graph.all_possible_states(max);
            state_list = state_table.clone().into_iter().collect();
            state_list.sort();
            invalid_states = find_invalid(&state_table, max);
            (theorem_1_states, not_t1_states) = find_theorem_1(&graph, &invalid_states, max);
            just_pressed = true;
          }

          let total = ((max + 1) as u32).pow(graph.nodes.len() as u32);
          ui.label(Vec2::new(30., 110.), &format!("{total} Total"));

          let old_viewing = viewing;
          ui.combo_box(hash!(), "Mode", &[
            "All Invalid",    // 0
            "Bad States",     // 1
            "NotBad States",  // 2
            "All Valid",      // 3
          ], &mut viewing);
          let focused_states = match viewing {
            0 => &invalid_states,
            1 => &theorem_1_states,
            2 => &not_t1_states,
            3 => &state_list,
            _ => unimplemented!()
          };
          if old_viewing != viewing || just_pressed {
            if !focused_states.is_empty() {
              let current_viewed = current_view_str.parse::<usize>().unwrap_or_default();
              if let Some(state) = focused_states.get(current_viewed.saturating_sub(1)) {
                for idx in 0 .. graph.nodes.len() {
                  graph.nodes.get_mut(idx).unwrap().value = state[idx];
                }
              }
            }
            let analysis = frequency_analysis(focused_states, max);
            parsed_analysis = parse_analysis(analysis, max, graph.nodes.len() as u8);
          }
          

          ui.input_text(
            hash!(),
            &format!("/{} Viewed States", focused_states.len()),
            &mut current_view_str
          );

          // Load current viewing state
          if let Ok(idx) = current_view_str.parse::<usize>()
          && let Some(state) = focused_states.get(idx.saturating_sub(1)) {
            for idx in 0 .. graph.nodes.len() {
              graph.nodes.get_mut(idx).unwrap().value = state[idx];
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
    });

    radius = if let Ok(radius) = radius_str.parse::<f32>() { radius } else { radius };

    if !root_ui().is_mouse_over(mouse_position().into()) {
      let mouse_pos = Vec2::from(mouse_position()).as_ivec2();
      nodes.hovering = graph.node_at(mouse_pos, radius);
      match edit_mode {
        0 => create_nodes_neighbors(&mut graph, &mut nodes, mouse_pos, radius),
        1 => remove_node(&mut graph, &mut nodes),
        2 => drag_nodes(&mut graph, &mut nodes, mouse_pos),
        3 => modify(&mut graph, &mut nodes, &modify_val, &max_str),
        4 => set(&mut graph, &mut nodes, &modify_val, &max_str),
        _ => (),
      }
    }

    graph.render(radius);

    next_frame().await
  }
}


fn find_theorem_1(graph: &Graph, invalid_states: &Vec<Vec<u8>>, max: u8) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
  if invalid_states.is_empty() { return (Vec::new(), Vec::new()) }
  let mut theorem_1_states = Vec::new();
  let mut not_t1_states = Vec::new();
  let node_count = invalid_states[0].len();

  'state: for state in invalid_states {
    // Check each closed neighborhood
    'neighborhood: for node in 0 .. node_count {
      let neighbors = &graph.nodes.get(node).unwrap().neighbors;
      let mut has_zero = false;
      let mut has_max = false;
      has_zero |= state[node] == 0;
      has_max |= state[node] == max;

      for neighbor in neighbors {
        has_zero |= state[*neighbor] == 0;
        has_max |= state[*neighbor] == max;
        if has_zero & has_max {
          continue 'neighborhood
        }
      }
      not_t1_states.push(state.clone());
      continue 'state;
    }
    theorem_1_states.push(state.clone());
  }

  (theorem_1_states, not_t1_states)
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

// Written by ai, don't trust
fn find_invalid(valid_states: &AHashSet<Vec<u8>>, max_value: u8) -> Vec<Vec<u8>> {
  if valid_states.is_empty() { return Vec::new() }
  let n = valid_states.iter().next().unwrap().len(); // length of each vector
  let base = (max_value as usize) + 1;
  let total_states = base.pow(n as u32);

  let mut missing = Vec::new();

  for num in 0..total_states {
    let mut state = Vec::with_capacity(n);
    let mut rem = num;

    // Convert number to base-(max_value+1)
    for _ in 0..n {
      state.push((rem % base) as u8);
      rem /= base;
    }

    if !valid_states.contains(&state) {
      missing.push(state);
    }
  }

  missing.sort();
  missing
}


