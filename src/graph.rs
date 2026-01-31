use eframe::egui::{Color32, Pos2};
use lilypads::Pond;

pub struct Graph { 
  pub nodes: Pond<Node>,
}
impl Graph {
  pub fn new() -> Self { Self { nodes: Pond::new() } }
  
  pub fn add_node(&mut self, position: Pos2) -> usize {
    self.nodes.insert(Node::new(position))
  }

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

  /// Assumes the graph is contiguized, if it isn't this might spit out nonsense
  pub fn export_state(&self) -> Vec<u8> {
    let mut output = vec![0; self.nodes.len()];
    for (idx, node) in self.nodes.iter() {
      output[idx] = node.value;
    }
    output
  }

  pub fn remove(&mut self, removed: usize) {
    let removed_node = self.nodes.free(removed).unwrap();
    for neighbor in removed_node.neighbors {
      self.nodes.get_mut(neighbor).unwrap().neighbors.retain(|search| { *search != removed });
    }
  }

  pub fn node_at(&self, point: Pos2, radius: f32) -> Option<usize> {
    for (idx, node) in self.nodes.safe_data().iter().enumerate() {
      if let Some(node) = *node {
        if node.position.distance_sq(point) < (radius as i32).pow(2) as f32 {
          return Some(idx)
        }
      }
    }
    None
  }

}
impl Graph {

  pub fn correct_max(&mut self, max: u8) {
    for (_, node) in self.nodes.iter_mut() {
      node.value = node.value.min(max);
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

#[derive(Debug)]
pub struct Node {
  pub position: Pos2,
  pub neighbors: Vec<usize>,
  pub value: u8,
  pub color: Color32,
}
impl Node {
  pub fn new(position: Pos2) -> Self {
    Self {
      position,
      neighbors: Vec::with_capacity(0),
      value: 0,
      color: Color32::RED,
    }
  }

  pub fn add_unique_neighbor(&mut self, neighbor: usize) {
    if !self.neighbors.contains(&neighbor) {
      self.neighbors.push(neighbor);
    }
  }

}

