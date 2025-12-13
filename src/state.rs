use ahash::AHashMap;

use crate::graph::Graph;
// Consider dumping the hashmap in exchange for just a sorted vec

pub type PackedState = u128;
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

  const fn digit_mask(base: u8) -> u128 {
    (1u128 << Self::bits_per_digit(base)) - 1
  }

  pub fn get(state: u128, idx: usize, base: u8, length: usize) -> u8 {
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

  pub fn increment(mut state: PackedState, base: u8, length: usize) -> Option<u128> {
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

#[derive(Clone, Copy)]
pub enum Classification {
  Valid,
  InvalidT1,
  InvalidOther,
}
pub struct StateData {
  meta: AHashMap<PackedState, Classification>,
  pub states: [Vec<PackedState>; 3], // One vec per Classification
  pub base: u8,
  pub length: usize,
}
impl StateData {
  pub fn new(graph: &mut Graph, base: u8) -> Option<Self> {
    graph.contiguize();

    let length = graph.node_count();
    assert!(length * StateOps::bits_per_digit(base) <= 128);

    let neighbors = graph.get_neighbors();
    if neighbors.is_empty() { return None; }

    let mut data = Self {
      meta: AHashMap::new(),
      states: [Vec::new(), Vec::new(), Vec::new()],
      base,
      length,
    };

    data.generate_valid(&neighbors);
    let invalid = data.generate_invalid();
    data.classify_invalid(invalid, &neighbors);
    
    Some(data)
  }

  pub fn get_list(&self, classification: Classification) -> &Vec<PackedState> {
    &self.states[classification as usize]
  }

  fn track_unique_state(&mut self, state: PackedState, classification: Classification) -> bool {
    if self.meta.contains_key(&state) { return false; }
    self.meta.insert(state.clone(), classification);
    self.states[classification as usize].push(state);
    true
  }

  pub fn parse_state(&self, state: PackedState) -> Vec<u8> {
    StateOps::to_vec(state, self.base, self.length)
  }

}

impl StateData {
  fn generate_valid(&mut self, neighbors: &Vec<Vec<usize>>) {
    let count = neighbors.len();

    let initial_state = 0;
    // let initial_state = vec![0u8; count].into_boxed_slice();
    self.track_unique_state(initial_state.clone(), Classification::Valid);
    let mut stack = vec![(initial_state, 0u8)];
    
    while let Some((mut state, op_idx)) = stack.pop() {
      if (op_idx + 1) >> 1 < count as u8 { stack.push((state.clone(), op_idx + 1)) }
      let center_idx = (op_idx >> 1) as usize;
      // We want to move apply a value of -1 if op_idx & 1 == 0 and 1 if op_idx & 1 == 1
      let operation = -1 + 2 * (op_idx & 0b1) as i8;
      
      // state[idx as usize] = state[idx].saturating_add_signed(operation).min(self.base);
      for idx in neighbors[center_idx].iter().chain(&[center_idx]) {
        let old_node = StateOps::get(state, *idx, self.base, self.length);
        let new_node = old_node.saturating_add_signed(operation).min(self.base - 1);
        state = StateOps::set(state, *idx, new_node, self.base, self.length);
      }

      if self.track_unique_state(state.clone(), Classification::Valid) {
        stack.push((state, 0));
      }
    }
  }

  // Compute the not of the valid set
  fn generate_invalid(&self) -> Vec<PackedState> {
    if self.states.is_empty() { return Vec::new() }
    let mut missing = Vec::new();
    
    // We know the zero state is always valid, which is why we're allowed to increment immediately
    let mut cur_state = 0;
    while let Some(state) = StateOps::increment(cur_state, self.base, self.length) {
      cur_state = state;
      if !self.meta.contains_key(&cur_state) { missing.push(cur_state); }
    }
  
    missing
  }

  // Ideally we could just generate all states, then classify all states
  // But the entire point of this research is to see if p = np, whether we can trivially confirm
  // whether a state is valid or invalid.
  // Right now, we can only define the set of invalid states as the not of the valid set.
  fn classify_invalid(&mut self, invalid_list: Vec<PackedState>, neighbors: &Vec<Vec<usize>>) {
    if invalid_list.is_empty() { return; }
    for state in invalid_list {
      if self.is_invalid_theorem_1(&state, neighbors) {
        self.track_unique_state(state, Classification::InvalidT1);
      } else {
        self.track_unique_state(state, Classification::InvalidOther);
      }
    }
  }

  // If every node's closed neighborhood contains a min and a max,
  // this state is a theorem one invalid
  fn is_invalid_theorem_1(&mut self, state: &PackedState, neighbors: &Vec<Vec<usize>>) -> bool {
    'node: for (central_idx, neighbor_indexes) in neighbors.iter().enumerate() {
      let mut has_zero = false;
      let mut has_max = false;

      for node in neighbor_indexes.iter().chain(&[central_idx])
        .map(|idx| { StateOps::get(*state, *idx, self.base, self.length) })
      {
        has_zero |= node == 0;
        has_max |= node == self.base - 1;
        if has_zero & has_max { continue 'node }
      }

      return false;
    }
    return true;
  }

}
