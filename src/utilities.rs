use std::str::FromStr;
use std::ops::{Add, Rem, Sub};
use macroquad::prelude::*;

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
  FromStr + Clone + Copy + ToString + Eq +
  Add<Output = T> + Sub<Output = T> + Rem<Output = T> 
{

  // I'm not a huge fan of this
  pub fn step_strnum(&mut self, max: T, step_size: T, increase: KeyCode, decrease: KeyCode) 
  {
    if is_key_pressed(decrease) {
      self.assign( if self.val == step_size { max } else { self.val - step_size } );
    } else if is_key_pressed(increase) {
      self.assign((self.val % max) + step_size);
    }
  }

}

pub enum UserMode {
  AddRemove { selected: Option<usize> },
  Drag { selected: Option<usize> },
  Play { allow_overflow: bool },
  Set { value: u8, val_str: String },
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
      2 => UserMode::Play { allow_overflow: true },
      3 => UserMode::Set { value: 0, val_str: "0".to_string() },
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
