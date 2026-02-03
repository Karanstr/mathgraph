use super::common::*;
use eframe::egui::{Color32, Event, LayerId, Order, Painter, Stroke};

pub struct Blueprint {
  selected: Option<usize>,
  action_cd: usize,
  can_drag: bool,
}
impl super::Mode for Blueprint {

  fn create(_program: &GraphProgram) -> Self {
    Self {
      selected: None,
      action_cd: 0,
      can_drag: false,
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {
    
    ui.horizontal(|ui| {
      if ui.button("Save").clicked() {
        // there is certainly a cheaper solution, but atm not my problem
        program.graph.contiguize_and_trim();
        ui.ctx().copy_text( to_graph6( program.graph.get_neighbors() ) );
        self.action_cd = 300;
      }
      if self.action_cd > 0 {
        ui.label("Copied to Clipboard!");
      }
    });
  
    ui.checkbox(&mut self.can_drag, "Drag (Space to Toggle)");

  }

  fn tick(&mut self, _program: &mut GraphProgram) {
    self.action_cd = self.action_cd.saturating_sub(1);
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {

    let Some(pos) = response.hover_pos() else { return };
    let hovering = program.get_node_at(pos);

    response.ctx.input(|input| {

      // Toggle drag on space
      for event in &input.events {
        let Event::Key { key: Key::Space, pressed: true, repeat: false, ..} = event else { continue; };
        self.can_drag = !self.can_drag;
      }

      // Delete hovering on right click
      if input.pointer.secondary_down() {
        if let Some(remove) = hovering { 
          program.graph.remove(remove);
          program.graph_changed = true;
        }
      }

      // Select/Create on left click
      if input.pointer.primary_pressed() {
        // Either we select the node we're hovering
        self.selected = hovering;
        // Or we create a node
        if self.selected.is_none() {
          self.selected = Some(program.graph.add_node(pos));
          program.graph_changed = true;
        }
        // Or do nothing if we're not touching it but too close to make one??
      }

    }); 
 
    if self.can_drag {
      if response.dragged_by(PointerButton::Primary) && let Some(node) = self.selected {
        program.graph.nodes.get_mut(node).unwrap().position = pos;
      }
    } else {

      // Draw line from selected node to mouse
      if response.dragged_by(PointerButton::Primary) && let Some(node) = self.selected {
        let line_color = if let Some(hovering) = hovering 
          && program.graph.has_connection(node, hovering) 
        { Color32::RED } else { Color32:: WHITE };
        let origin = program.graph.nodes.get(node).unwrap().position;
        let lines = Painter::new(response.ctx.clone(), LayerId::new(Order::Background, Id::new("Lines")), response.interact_rect);
        lines.line_segment([pos, origin], Stroke::new(4., line_color));
      }
      
      if response.drag_stopped_by(PointerButton::Primary) {
        if let Some(node1) = self.selected && let Some(node2) = hovering && node1 != node2 {
          // If we fail to add unique connection, remove the existing one.
          if !program.graph.attempt_unique_connection(node1, node2) {
            program.graph.remove_connection(node1, node2);

          };
          program.graph_changed = true;
        }
        self.selected = None;
      }

    }
  }

}


/// Serialize an undirected simple graph into graph6 format.
/// 
/// `adj[i]` contains the neighbors of vertex `i`.
/// Assumes:
/// - vertices are 0..n-1
/// - no self-loops
/// - undirected (i in adj[j] iff j in adj[i])

// Visualizer which allows graph6 imports
// https://houseofgraphs.org/draw_graph
pub fn to_graph6(adj: Vec< Vec<usize> >) -> String {
  let n = adj.len();
  assert!(n <= 62, "This implementation supports n <= 62");

  // Encode number of vertices
  let mut output = String::new();
  output.push((n as u8 + 63) as char);

  // Build adjacency lookup for fast edge testing
  let mut has_edge = vec![vec![false; n]; n];
  for (u, neighbors) in adj.iter().enumerate() {
    for &v in neighbors {
      assert!(u != v, "Self-loops are not allowed");
      has_edge[u][v] = true;
      has_edge[v][u] = true;
    }
  }

  // Collect upper-triangle bits in graph6 order
  let mut bits: Vec<u8> = Vec::new();
  for j in 1..n {
    for i in 0..j {
      bits.push(if has_edge[i][j] { 1 } else { 0 });
    }
  }

  // Pad with zeros to multiple of 6
  while bits.len() % 6 != 0 {
    bits.push(0);
  }

  // Encode bits in chunks of 6
  for chunk in bits.chunks(6) {
    let mut value = 0u8;
    for &bit in chunk {
      value = (value << 1) | bit;
    }
    output.push((value + 63) as char);
  }

  output
}
