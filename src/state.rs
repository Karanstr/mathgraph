use ahash::AHashMap;

use crate::graph::Graph;
// Instead of wasting SOOOO much space per box we can just limit
// our support such that state can be stored in a single u128
// Also consider dumping the hashmap in exchange for just a sorted vec
pub type State = Box<[u8]>;

// struct State(u128);
// impl State { }

#[derive(Clone, Copy)]
pub enum Classification {
  Valid,
  InvalidT1,
  InvalidOther,
}
pub struct StateData {
  meta: AHashMap<State, Classification>,
  pub states: [Vec<State>; 3] // One vec per Classification
}
impl StateData {
  pub fn new() -> Self {
    Self {
      meta: AHashMap::new(),
      states: [Vec::new(), Vec::new(), Vec::new()],
    }
  }

  pub fn clear(&mut self) {
    self.meta.clear();
    self.states = [Vec::new(), Vec::new(), Vec::new()];
  }

  pub fn get_list(&self, classification: Classification) -> &Vec<State> {
    &self.states[classification as usize]
  }

  pub fn initialize(&mut self, graph: &mut Graph, max: u8) {
    self.clear();
    graph.contiguize();
    let neighbors = graph.get_neighbors();
    if neighbors.is_empty() { return; }
    self.generate_valid(&neighbors, max);

    let invalid = self.generate_invalid(max);
    self.classify_invalid(invalid, &neighbors, max);

    // Consider sorting all states for simplicity
  }

  fn track_unique_state(&mut self, state: State, classification: Classification) -> bool {
    if self.meta.contains_key(&state) { return false; }
    self.meta.insert(state.clone(), classification);
    self.states[classification as usize].push(state);
    true
  }

}

impl StateData {
  fn generate_valid(&mut self, neighbors: &Vec<Vec<usize>>, max: u8) {
    let count = neighbors.len();

    let initial_state = vec![0u8; count].into_boxed_slice();
    self.track_unique_state(initial_state.clone(), Classification::Valid);
    let mut stack = vec![(initial_state, 0u8)];
    
    while let Some((mut state, op_idx)) = stack.pop() {
      if (op_idx + 1) >> 1 < count as u8 { stack.push((state.clone(), op_idx + 1)) }
      let idx = (op_idx >> 1) as usize;
      // We want to move apply a value of -1 if op_idx & 1 == 0 and 1 if op_idx & 1 == 1
      let operation = -1 + 2 * (op_idx & 0b1) as i8;

      state[idx as usize] = state[idx].saturating_add_signed(operation).min(max);
      for neighbor in &neighbors[idx] {
        state[*neighbor] = state[*neighbor].saturating_add_signed(operation).min(max);
      }

      if self.track_unique_state(state.clone(), Classification::Valid) {
        stack.push((state, 0));
      }
    }
  }

  // Compute the not of the valid set
  fn generate_invalid(&self, max: u8) -> Vec<State> {
    if self.states.is_empty() { return Vec::new() }
    let mut missing = Vec::new();
    
    let n = self.meta.iter().next().unwrap().0.len(); // length of each state
    let base = (max as usize) + 1;
    let total_states = base.pow(n as u32);
 
    // Buffer reuse
    let mut state = vec![0; n];
    for num in 0..total_states {
      let mut rem = num;

      // Convert number to base-(max_value+1)
      for digit in &mut state {
        *digit = (rem % base) as u8;
        rem /= base;
      }
      
      let boxed = state.clone().into_boxed_slice();
      if !self.meta.contains_key(&boxed) { missing.push(boxed); }
    }

    missing
  }

  // Ideally we could just generate all states, then classify all states
  // But the entire point of this research is to see if p = np, whether we can trivially confirm
  // whether a state is valid or invalid.
  // Right now, we can only define the set of invalid states as the not of the valid set.
  fn classify_invalid(&mut self, invalid_list: Vec<State>, neighbors: &Vec<Vec<usize>>, max: u8) {
    if invalid_list.is_empty() { return; }
    for state in invalid_list {
      if self.is_invalid_theorem_1(&state, neighbors, max) {
        self.track_unique_state(state, Classification::InvalidT1);
      } else {
        self.track_unique_state(state, Classification::InvalidOther);
      }
    }
  }

  // If every node's closed neighborhood contains a min and a max,
  // this state is a theorem one invalid
  fn is_invalid_theorem_1(&mut self, state: &State, neighbors: &Vec<Vec<usize>>, max: u8) -> bool {
    'node: for (central_node, neighbor_indices) in state.iter().zip(neighbors) {
      let mut has_zero = false;
      let mut has_max = false;

      for node in neighbor_indices.iter().map(|idx| { &state[*idx] }).chain([central_node]) {
        has_zero |= *node == 0;
        has_max |= *node == max;
        if has_zero & has_max { continue 'node }
      }

      return false;
    }
    return true;
  }

}
