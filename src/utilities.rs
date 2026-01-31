use std::str::FromStr;
use std::ops::{Add, Rem, Sub};

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
  pub fn step_strnum(&mut self, max: T, step_size: T, increase: bool) 
  {
    if increase {
      self.assign((self.val % max) + step_size);
    } else {
      let new_val = 
        if self.val == step_size { max }
        else if self.val > max { max }
        else { self.val - step_size }
      ;
      self.assign(new_val);
    }
  }

}

