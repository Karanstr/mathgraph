use macroquad::math::IVec2;
use lilypads::Pond;
use macroquad::shapes::*;
use macroquad::color::*;
use macroquad::text::draw_text;
use ahash::AHashSet;

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

  pub fn add_neighbor(&mut self, neighbor: usize) {
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

  pub fn add_connection(&mut self, node1: usize, node2: usize) -> bool {
    if self.nodes.is_occupied(node1) && self.nodes.is_occupied(node2) {
      self.nodes.get_mut(node1).unwrap().add_neighbor(node2);
      self.nodes.get_mut(node2).unwrap().add_neighbor(node1);
      true
    } else { false }
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

// Exciting logic stuff
impl Graph {

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

  pub fn contiguize(&mut self) {
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

  pub fn determine_state_space(&mut self, all_states: AHashSet<Vec<u8>>, max_value: u8) -> AHashSet<Vec<u8>> {
    self.contiguize();
    let neighbors = self.get_neighbors();

    let count = self.nodes.len();
    if count == 0 { return AHashSet::new(); }
    let mut initial_state = vec![0u8; count];
    for (idx, node) in self.nodes.safe_data().iter().enumerate() {
      // This is safe because contiguize promises only safe values exist
      initial_state[idx] = node.unwrap().value;
    }
    let mut states = AHashSet::from([initial_state.clone()]);
    let mut stack = vec![(initial_state, 0u8)];

    'state_space: while let Some((mut state, op_idx)) = stack.pop() {
      if op_idx + 1 >> 1 < count as u8 { stack.push((state.clone(), op_idx + 1)) }
      let idx = op_idx >> 1;
      // We want to move apply a value of -1 if op_idx & 1 == 0 and 1 if op_idx & 1 == 1
      let operation = -1 + 2 * (op_idx & 0b1) as i8;
      {
        let new_val = state[idx as usize] as i8 + operation;
        if new_val < 0 || new_val as u8 > max_value { continue 'state_space }
        state[idx as usize] = state[idx as usize].checked_add_signed(operation).unwrap();
      }
      for neighbor in &self.nodes.get(idx as usize).unwrap().neighbors {
        let new_val = state[*neighbor as usize] as i8 + operation;
        if new_val < 0 || new_val as u8 > max_value { continue 'state_space }
        state[idx as usize] = state[*neighbor as usize].checked_add_signed(operation).unwrap();
      }

      if states.insert(state.clone()) { stack.push((state, 0)) }
    }

    states
  }

}


