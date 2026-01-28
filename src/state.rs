use ahash::{AHashMap, AHashSet};

use crate::graph::Graph;

pub type PackedState = u128;
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Classification {
  Valid,
  InvalidT1,
  InvalidOther,
}

struct Metadata {
  classification: Option<(Classification, usize)>,
  bubble: Option<(usize, usize)>,
}
impl Metadata {
  fn set_bubble(&mut self, bubble_idx: usize, state_idx: usize) { self.bubble = Some((bubble_idx, state_idx)); }
  fn classification(&self) -> (Classification, usize) { self.classification.unwrap() }
  fn bubble(&self) -> (usize, usize) { self.bubble.unwrap() }
}

pub struct StateData {
  meta: AHashMap<PackedState, Metadata>,
  pub bubbles: Vec< Vec<PackedState> >,
  states: [Vec<PackedState>; 3], // One vec per Classification
  pub base: u8,
  neighbors: Vec< Vec<usize> >
}
impl StateData {
  pub fn new(graph: &mut Graph, max: u8) -> Option<Self> {
    graph.contiguize_and_trim();

    let neighbors = graph.get_neighbors();
    if neighbors.is_empty() { return None; }

    let length = neighbors.len();
    let base = max + 1;
    assert!(length * StateOps::bits_per_digit(base) <= 128);

    let mut data = Self {
      meta: AHashMap::new(),
      bubbles: Vec::new(),
      states: [Vec::new(), Vec::new(), Vec::new()],
      base,
      neighbors: graph.get_neighbors(),
    };

    data.generate_valid();
    let invalid = data.generate_invalid();
    data.classify_invalid(invalid);
    
    data.identify_bubbles();

    Some(data)
  }

  pub fn get_list(&self, classification: Classification) -> &Vec<PackedState> {
    &self.states[classification as usize]
  }

  fn track_unique_state(&mut self, state: PackedState, classification: Classification) -> bool {
    if self.meta.contains_key(&state) { return false; }
    let state_vec = &mut self.states[classification as usize];
    let metadata = Metadata {
      classification: Some((classification, state_vec.len())),
      bubble: None
    };
    self.meta.insert(state.clone(), metadata);
    state_vec.push(state);
    true
  }

  pub fn length(&self) -> usize { self.neighbors.len() }

  pub fn parse_state(&self, state: PackedState) -> Vec<u8> {
    StateOps::to_vec(state, self.base, self.length())
  }

  pub fn parse_vec(&self, vec: Vec<u8>) -> PackedState {
    StateOps::from_vec(vec, self.base)
  }

  pub fn set_packed(&self, state: PackedState, idx: usize, value: u8) -> PackedState {
    StateOps::set(state, idx, value, self.base, self.length())
  }

  pub fn classification_data(&self, state: PackedState) -> (Classification, usize) {
    self.meta.get(&state).unwrap().classification()
  }

  /// Returns (bubble_idx, state_idx)
  pub fn bubble_data(&self, state: PackedState) -> (usize, usize) {
    self.meta.get(&state).unwrap().bubble()
  }

}
impl StateData {
  fn generate_valid(&mut self) {
    for state in self.dfs(0, true) {
      self.track_unique_state(state, Classification::Valid);
    }
  }

  // Compute the not of the valid set
  fn generate_invalid(&self) -> Vec<PackedState> {
    if self.states.is_empty() { return Vec::new() }
    let mut missing = Vec::new();
    
    // We know the zero state is always valid, which is why we're allowed to increment immediately
    let mut cur_state = 0;
    while let Some(state) = StateOps::increment(cur_state, self.base, self.length()) {
      cur_state = state;
      if !self.meta.contains_key(&cur_state) { missing.push(cur_state); }
    }
  
    missing
  }

  // Ideally we could just generate all states, then classify all states
  // But the entire point of this research is to see if p = np, whether we can trivially confirm
  // whether a state is valid or invalid.
  // Right now, we can only define the set of invalid states as the not of the valid set.
  fn classify_invalid(&mut self, invalid_list: Vec<PackedState>) {
    if invalid_list.is_empty() { return; }
    for state in invalid_list {
      if self.is_invalid_theorem_1(state) {
        self.track_unique_state(state, Classification::InvalidT1);
      } else {
        self.track_unique_state(state, Classification::InvalidOther);
      }
    }
  }

  // If every node's closed neighborhood contains a min and a max,
  // this state is a theorem one invalid
  fn is_invalid_theorem_1(&self, state: PackedState) -> bool {
    for center in 0 .. self.length() {
      let (has_zero, has_max) = self.neighborhood_zero_or_max(state, center);
      if !(has_zero && has_max) { return false }
    }
    return true;
  }

  pub fn neighborhood_zero_or_max(&self, state: PackedState, node: usize) -> (bool, bool) {
    let mut has_zero = false;
    let mut has_max = false;

    for node in self.neighbors[node].iter().chain(&[node])
      .map( |idx| { StateOps::get(state, *idx, self.base, self.length()) } )
    {
      has_zero |= node == 0;
      has_max |= node == self.base - 1;
      if has_zero && has_max { break }
    }

    (has_zero, has_max)
  }

  fn identify_bubbles(&mut self) {
    let mut seen_states = AHashSet::<PackedState>::new();

    let mut smol_bubbles = Vec::new();

    for initial_state in self.meta.keys() {
      if seen_states.contains(initial_state) { continue }
      let mut bubble = self.dfs(*initial_state, false);
      if bubble.len() == 1 { 
        smol_bubbles.extend(bubble.drain());
        continue;
      }
      let mut bubble_vec = Vec::with_capacity(bubble.len());
      for state in bubble {
        seen_states.insert(state);
        bubble_vec.push(state);
      }
      self.bubbles.push(bubble_vec);
    }

    for (bubble_vec, bubble) in self.bubbles.iter().enumerate() {
      for (bubble_idx, state) in bubble.iter().enumerate() {
        self.meta.get_mut(state).unwrap().set_bubble(bubble_vec, bubble_idx);
      }
    }

    let single_bubbles = self.bubbles.len();
    for (idx, state) in smol_bubbles.iter().enumerate() {
      self.meta.get_mut(state).unwrap().set_bubble(single_bubbles, idx);
    }
    self.bubbles.push(smol_bubbles);
  
  }

  fn dfs(&self, initial_state: PackedState, saturate: bool) -> AHashSet<PackedState> {
    let count = self.neighbors.len();
    let mut stack = vec![(initial_state, 0u8)];
    let mut found_states = AHashSet::new();
    found_states.insert(initial_state);
    
    'search: while let Some((state, op_idx)) = stack.pop() {
      if (op_idx + 1) >> 1 < count as u8 { stack.push((state.clone(), op_idx + 1)) }
      let center_idx = (op_idx >> 1) as usize;
      // We want to apply a value of -1 if op_idx & 1 == 0 and 1 if op_idx & 1 == 1
      let operation = -1 + (op_idx & 0b1) as i8 * 2;
      
      if let Some(new_state) = self.splash_state(state, center_idx, operation, saturate) {
        if found_states.insert(new_state.clone()) { stack.push((new_state, 0)) }
      } else { continue 'search }

    }

    found_states
  }
  
  pub fn splash_state(
    &self,
    mut state: PackedState,
    center: usize,
    operation: i8,
    allow_clamping: bool,
  ) -> Option<PackedState> {
    for idx in self.neighbors[center].iter().chain(&[center]) {
      let old_node = StateOps::get(state, *idx, self.base, self.length());
      let new_node = old_node.saturating_add_signed(operation).min(self.base - 1);
      if old_node == new_node && !allow_clamping { return None }
      state = StateOps::set(state, *idx, new_node, self.base, self.length());
    }
    Some(state)
  }

}

struct StateOps;
impl StateOps {
 
  const fn bits_per_digit(base: u8) -> usize {
    let mut bits = 0;
    let mut v = base - 1;
    while v > 0 {
      bits += 1;
      v >>= 1;
    }
    bits
  }

  const fn digit_mask(base: u8) -> PackedState {
    (1u128 << Self::bits_per_digit(base)) - 1
  }

  pub fn get(state: PackedState, idx: usize, base: u8, length: usize) -> u8 {
    debug_assert!(idx < length);
    let shift = idx * Self::bits_per_digit(base);
    ((state >> shift) & Self::digit_mask(base)) as u8
  }
  
  pub fn set(state: PackedState, idx: usize, value: u8, base: u8, length: usize) -> PackedState {
    debug_assert!(idx < length);
    debug_assert!(value < base);

    let bits = Self::bits_per_digit(base);
    let shift = idx * bits;
    let mask = Self::digit_mask(base) << shift;

    (state & !mask) | ((value as u128) << shift)
  }

  pub fn to_vec(state: PackedState, base: u8, length: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(length);
    for i in 0 .. length {
      vec.push(Self::get(state, i, base, length));
    }
    vec
  }

  pub fn from_vec(vec: Vec<u8>, base: u8) -> PackedState {
    let mut state = 0;
    for (idx, val) in vec.iter().enumerate() {
      state = Self::set(state, idx, *val, base, vec.len());
    }
    state
  }

  pub fn increment(mut state: PackedState, base: u8, length: usize) -> Option<PackedState> {
    let bits = Self::bits_per_digit(base);
    let mask = Self::digit_mask(base);

    let mut i = 0;
    while i < length {
      let shift = i * bits;
      let digit = (state >> shift) & mask;

      if digit + 1 < base as u128 {
        state += 1u128 << shift;
        return Some(state);
      } else {
        // reset digit to 0
        state &= !(mask << shift);
      }

      i += 1;
    }

    None
  }
}

// Returns a count of how many of each node value each state has
// Per state, how many nodes have a value
// result[state][value] = node_count
pub fn frequency_analysis(states: &Vec<PackedState>, length: usize, max: u8) -> Vec<Vec<u32>> {
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
pub fn parse_analysis(analysis: Vec<Vec<u32>>, max: u8, node_count: u8) -> Vec<Vec<u32>> {
  if analysis.is_empty() { return Vec::new(); }
  let mut result = vec![vec![0; node_count as usize + 1]; max as usize + 1];

  for state in analysis {
    for (value, node_count) in state.iter().enumerate() {
      result[value][*node_count as usize] += 1; 
    }
  }
  result
}

pub fn combine<'a>(a: &'a [PackedState], b: &'a [PackedState]) -> Vec<PackedState> {
  let mut out = Vec::with_capacity(a.len() + b.len());
  out.extend_from_slice(a);
  out.extend_from_slice(b);
  out
}

