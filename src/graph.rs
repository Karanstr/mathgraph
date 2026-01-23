use ahash::AHashMap;
use macroquad::math::IVec2;
use lilypads::Pond;
use macroquad::shapes::*;
use macroquad::color::*;
use macroquad::text::draw_text;

#[derive(Debug)]
pub struct Node {
  pub position: IVec2,
  pub neighbors: Vec<usize>,
  pub value: u8
}
impl Node {
  pub fn new(position: IVec2) -> Self {
    Self {
      position,
      neighbors: Vec::with_capacity(0),
      value: 0,
    }
  }

  pub fn add_unique_neighbor(&mut self, neighbor: usize) {
    if !self.neighbors.contains(&neighbor) {
      self.neighbors.push(neighbor);
    }
  }

}

pub struct Graph { 
  pub nodes: Pond<Node>,
}
// Boring management stuff
impl Graph {
  pub fn new() -> Self { Self { nodes: Pond::new() } }
  
  pub fn add_node(&mut self, position: IVec2) -> usize {
    self.nodes.insert(Node::new(position))
  }

  pub fn node_count(&self) -> usize { self.nodes.iter().count() }

  /// Adds neighbors if not neighbors already, otherwise does nothing
  pub fn attempt_unique_connection(&mut self, node1: usize, node2: usize) -> bool {
    if self.nodes.is_occupied(node1) && self.nodes.is_occupied(node2) {
      self.nodes.get_mut(node1).unwrap().add_unique_neighbor(node2);
      self.nodes.get_mut(node2).unwrap().add_unique_neighbor(node1);
      true
    } else { false }
  }

  pub fn load_state(&mut self, state: Vec<u8>) {
    for (idx, node) in self.nodes.iter_mut() {
      node.value = state[idx];
    }
  }

  pub fn remove(&mut self, removed: usize) {
    let removed_node = self.nodes.free(removed).unwrap();
    for neighbor in removed_node.neighbors {
      self.nodes.get_mut(neighbor).unwrap().neighbors.retain(|search| { *search != removed });
    }
  }

  pub fn node_at(&self, point: IVec2, radius: f32) -> Option<usize> {
    for (idx, node) in self.nodes.safe_data().iter().enumerate() {
      if let Some(node) = *node {
        if node.position.distance_squared(point) < (radius as i32).pow(2) {
          return Some(idx)
        }
      }
    }
    None
  }

  pub fn render(&self, radius: f32) {
    let mut circles: Vec<(IVec2, u8)> = Vec::new();
    for node in self.nodes.safe_data() {
      if let Some(node) = node {
        circles.push((node.position, node.value));
        for neighbor in &node.neighbors {
          let other_node = self.nodes.get(*neighbor).unwrap();
          draw_line(
            node.position.x as f32, node.position.y as f32,
            other_node.position.x as f32, other_node.position.y as f32,
            4., WHITE
          );
        }
      }
    }
    
    let font_size = radius;
    for (pos, value) in circles {
      draw_circle(
        pos.x as f32,
        pos.y as f32,
        radius, RED
      );
      draw_text(
        &format!("{value}"),
        pos.x as f32 - font_size/4. * (1 + value.max(1).ilog10()) as f32,
        pos.y as f32 + font_size/4.,
        font_size, WHITE
      );
    }
  }

}

impl Graph {

  pub fn correct_max(&mut self, max: u8) {
    for (_, node) in self.nodes.iter_mut() {
      node.value = node.value.min(max);
    }
  }

  pub fn clamped_update(&mut self, node_idx: usize, delta: i8, max: u8) {
    let neighbors = if let Some(node) = self.nodes.get_mut(node_idx) {
      node.value = node.value.saturating_add_signed(delta).min(max);
      node.neighbors.clone()
    } else { return };
    for neighbor in neighbors {
      let node = self.nodes.get_mut(neighbor).unwrap();
      node.value = node.value.saturating_add_signed(delta).min(max);
    }
  }

  // I can probably do this better, but for now this works
  pub fn restricted_update(&mut self, node_idx: usize, delta: i8, max: u8) {
    let mut new_vals = AHashMap::new();
    let neighbors = if let Some(node) = self.nodes.get_mut(node_idx) {
      // No clamping
      let new_val = node.value.saturating_add_signed(delta).min(max);
      if new_val == node.value { return }
      new_vals.insert(node_idx, new_val);

      node.neighbors.clone()
    } else { return };
    for neighbor in neighbors {
      let node = self.nodes.get_mut(neighbor).unwrap();
      
      // No clamping
      let new_val = node.value.saturating_add_signed(delta).min(max);
      if new_val == node.value { return }
      new_vals.insert(neighbor, new_val);

    }

    for (idx, val) in new_vals {
      self.nodes.get_mut(idx).unwrap().value = val;
    }

  }

  pub fn contiguize_and_trim(&mut self) {
    let fix = self.nodes.trim();
    for idx in 0 .. self.nodes.len() {
      let node = self.nodes.get_mut(idx).unwrap();
      let mut new_neighbors = Vec::with_capacity(node.neighbors.len());
      for neighbor in &node.neighbors {
        new_neighbors.push(*fix.get(neighbor).unwrap_or(neighbor))
      }
      node.neighbors = new_neighbors;
    }
  }

  /// Assumes graph has already been contiguized by [Self::contiguize]
  pub fn get_neighbors(&self) -> Vec<Vec<usize>> {
    let mut neigbors = Vec::new();
    for (_, node) in self.nodes.iter() {
      neigbors.push(node.neighbors.clone());
    }
    neigbors
  }

}

