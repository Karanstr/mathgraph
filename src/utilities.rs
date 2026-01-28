use std::str::FromStr;
use std::ops::{Add, Rem, Sub};
use macroquad::prelude::*;

pub enum UserMode {
  AddRemove { selected: Option<usize> },
  Drag { selected: Option<usize> },
  Play { allow_clamping: bool },
  Set { value: StrType<u8> },
  Analyze {
    viewing_type: usize,
    viewing_length: usize,
    viewing: StrType<usize>,
    parsed_analysis: Vec<Vec<u32>>,
  },
  Bubbles {
    bubble: StrType<usize>,
    bubble_length: usize,
    state: StrType<usize>,
    state_length: usize,
  },
}
impl UserMode {
  pub fn as_int(&self) -> usize {
    match self {
      Self::AddRemove {..} => 0,
      Self::Drag {..} => 1,
      Self::Play {..} => 2,
      Self::Set {..} => 3,
      Self::Analyze {..} => 4,
      Self::Bubbles {..} => 5,
    }
  }

  pub fn from_int(val: usize) -> Self {
    match val {
      0 => UserMode::AddRemove { selected: None },
      1 => UserMode::Drag { selected: None },
      2 => UserMode::Play { allow_clamping: true },
      3 => UserMode::Set { value: StrType::new(0) },
      4 => UserMode::Analyze { 
        viewing_type: 0,
        viewing_length: 0,
        viewing: StrType::new(1),
        parsed_analysis: Vec::new(),
      },
      5 => UserMode::Bubbles {
        bubble: StrType::new(1),
        bubble_length: 0,
        state: StrType::new(1),
        state_length: 0,
      },
      _ => unreachable!()
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

pub struct StrType<T> where T: FromStr + Clone + ToString {
  string: String,
  val: T,
}
impl<T> StrType<T> where T: FromStr + Clone + ToString {

  pub fn new(initial: T) -> Self {
    Self {
      string: initial.clone().to_string(),
      val: initial
    }
  }

  pub fn parse(&mut self) -> T {
    if let Ok(val) = self.string.parse::<T>() { self.val = val }
    self.val.clone()
  }

  pub fn assign(&mut self, val: T) {
    self.val = val;
    self.string = self.val.to_string();
  }

  pub fn string_mut(&mut self) -> &mut String { &mut self.string }

  pub fn val(&self) -> T { self.val.clone() }

}
impl<T> StrType<T>
where T: 
  FromStr + Clone + Copy + ToString + Eq + Ord +
  Add<Output = T> + Sub<Output = T> + Rem<Output = T> 
{

  // I'm not a huge fan of this
  pub fn step_strnum(&mut self, max: T, step_size: T, increase: KeyCode, decrease: KeyCode) 
  {
    if is_key_pressed(decrease) {
      let new_val = 
        if self.val == step_size { max }
        else if self.val > max { max }
        else { self.val - step_size }
      ;
      self.assign(new_val);
    } else if is_key_pressed(increase) {
      self.assign((self.val % max) + step_size);
    }
  }

}

