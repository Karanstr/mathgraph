use std::ops::RangeInclusive;

use crate::{NODE_RADIUS, graph::Graph};

use super::common::*;
use eframe::egui::{Align2, Area, Color32, Context, Event, FontId, LayerId, Order, Painter, Pos2, Rect, RichText, Stroke, Vec2, Window};

pub struct Blueprint {
  selected: Option<usize>,
  action_cd: usize,
  action: usize, // 1 is save, 2 is load
  can_drag: bool,

  loading_screen: bool,
  load_n: usize,
}
impl Blueprint {
  fn load_menu(&mut self, program: &mut GraphProgram, ctx: &Context) {
    Area::new(Id::new("Dimming"))
      .order(Order::Foreground)
      .interactable(true)
      .show(ctx, |ui| {
        let screen_rect = ctx.content_rect();

        ui.painter().rect_filled(
          screen_rect,
          0.0,
          Color32::from_black_alpha(150),
        );
      })
    ;

    Window::new("Load")
      .anchor(Align2::CENTER_CENTER, [0.0, -100.0])
      .collapsible(false)
      .resizable(false)
      .order(Order::Tooltip)
      .title_bar(false)
      .show(ctx, |ui| {
        ui.style_mut().override_font_id = Some(FontId::proportional(20.));
        ui.vertical_centered(|ui| {
          ui.label(RichText::new("Load Options").size(30.).strong());
          
          DragValue::new(&mut self.load_n)
            .range(RangeInclusive::new(1, 15))
            .speed(0.1)
            .ui(ui)
          ;

          if ui.button("Path").clicked() {
            program.graph = GraphType::Path(self.load_n).new(ctx.content_rect());
            program.graph_changed = true;
            self.loading_screen = false;
          }

          if ui.button("Cycle").clicked() {
            program.graph = GraphType::Cycle(self.load_n).new(ctx.content_rect());
            program.graph_changed = true;
            self.loading_screen = false;
          }

          if ui.button("Complete").clicked() {
            program.graph = GraphType::Complete(self.load_n).new(ctx.content_rect());
            program.graph_changed = true;
            self.loading_screen = false;
          }

          if ui.button("Wheel").clicked() {
            program.graph = GraphType::Wheel(self.load_n).new(ctx.content_rect());
            program.graph_changed = true;
            self.loading_screen = false;
          }

          if ui.button("Star").clicked() {
            program.graph = GraphType::Star(self.load_n).new(ctx.content_rect());
            program.graph_changed = true;
            self.loading_screen = false;
          }

          if ui.button("Cancel").clicked() {
            self.loading_screen = false;
          }

        });
        

      })
    ;

  }

}
impl super::Mode for Blueprint {

  fn create(_program: &GraphProgram) -> Self {
    Self {
      selected: None,
      action_cd: 0,
      action: 0,
      can_drag: false,

      loading_screen: false,
      load_n: 1,
    }
  }

  fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {
    if self.loading_screen { self.load_menu(program, ui.ctx()); return } 
    
    ui.checkbox(&mut self.can_drag, "Drag (Space to Toggle)");

    ui.horizontal(|ui| {
      if ui.button("Save").clicked() {
        // there is certainly a cheaper solution, but atm not my problem
        program.graph.contiguize_and_trim();
        ui.ctx().copy_text( to_graph6( program.graph.get_neighbors() ) );
        self.action_cd = 300;
        self.action = 1;
      }
      if ui.button("Load").clicked() { self.loading_screen = true }
    });
    
    if self.action_cd > 0 {
      let message = match self.action {
        1 => "Copied to Clipboard!",
        2 => "Loaded!",
        _ => unimplemented!()
      };
      ui.label(message);
    } else { self.action = 0; }

  }

  fn tick(&mut self, _program: &mut GraphProgram) {
    self.action_cd = self.action_cd.saturating_sub(1);
  }

  fn interactions(&mut self, program: &mut GraphProgram, response: Response) {
    if self.loading_screen { return }

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

enum GraphType {
  Path(usize),
  Cycle(usize),
  Complete(usize),
  Wheel(usize),
  Star(usize),
}
impl GraphType {
  // Add a max size to prevent from going off screen
  fn new(self, space: Rect) -> Graph {
    let mut graph = Graph::new();
    match self {
      Self::Path(n) => {
        let node_size = NODE_RADIUS * 3.;
        let step = Vec2::new(node_size, 0.);
        let start= space.center() - Vec2::new((node_size * n as f32 - 1.) / 2., 0.);
        let mut cur_pos = start;

        graph.add_node(cur_pos);
        if n == 1 { return graph }
        graph.unchecked_directed_connection(0, 1);

        cur_pos += step;
        for i in 1 .. n - 1 {
          graph.add_node(cur_pos);
          graph.unchecked_directed_connection(i, i + 1);
          graph.unchecked_directed_connection(i, i - 1);
          cur_pos += step;
        }
        graph.add_node(cur_pos);
        graph.unchecked_directed_connection(n - 1, 0);
      }
      Self::Cycle(n) => {
        let big_radius = NODE_RADIUS * n as f32;
        let points = points_on_circle(n, space.center(), big_radius);
        for point in points {
          graph.add_node(point);
        }
        for i in 1 .. n - 1 {
          graph.attempt_unique_connection(i, (i + 1) % n);
          graph.attempt_unique_connection(i, i - 1);
        }
        graph.attempt_unique_connection(0, n - 1);
      }
      Self::Complete(n) => {
        let big_radius = NODE_RADIUS * n as f32;
        let points = points_on_circle(n, space.center(), big_radius);
        for point in points {
          graph.add_node(point);
        }
        for i in 0 .. n {
          for j in 0 .. n {
            // Not optimal but fast :sunglasses:
            graph.attempt_unique_connection(i, j);
          }
        }
        graph.attempt_unique_connection(0, n - 1);
      }
      Self::Wheel(n) => {
        graph.add_node(space.center());
        let big_radius = (NODE_RADIUS * n as f32).max(200.);
        let points = points_on_circle(n, space.center(), big_radius);
        for point in points {
          graph.add_node(point);
        }
        // This logic feels wrong but it works
        for i in 1 .. n + 1 {
          graph.attempt_unique_connection(i, (i + 1) % n);
          graph.attempt_unique_connection(i, i - 1);
          graph.attempt_unique_connection(0, i);
        }
        graph.attempt_unique_connection(1, n);
      }
      Self::Star(n) => {
        graph.add_node(space.center());
        let big_radius = (NODE_RADIUS * n as f32).max(200.);
        let points = points_on_circle(n, space.center(), big_radius);
        for point in points { graph.add_node(point); }
        for i in 1 .. n + 1 {
          graph.attempt_unique_connection(0, i);
        }
      }
    };
    graph
  }
}

use std::f32::consts::TAU;

fn points_on_circle( n: usize, center: Pos2, radius: f32) -> Vec<Pos2> {
  (0..n).map(|i| {
    let theta = TAU * i as f32 / n as f32;
    Pos2::new(
      center.x + radius * theta.cos(),
      center.y + radius * theta.sin(),
    )
  }).collect()
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
