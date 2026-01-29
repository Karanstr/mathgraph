mod addremove;
mod drag;
mod play;
mod set;
mod analyze;
mod bubbles;

use crate::GraphProgram;
use macroquad::ui::Ui;

trait Mode where Self: Sized {

  fn create(_program: &mut GraphProgram) -> Self;
  fn ui(&mut self, _program: &mut GraphProgram, _ui: &mut Ui) {}
  fn tick(&mut self, _program: &mut GraphProgram) {}
  fn interactions(&mut self, _program: &mut GraphProgram) {}

}
pub enum Modes {
  AddRemove(addremove::AddRemove),
  Drag(drag::Drag),
  Play(play::Play),
  Set(set::Set),
  Analyze(analyze::Analyze),
  Bubbles(bubbles::Bubbles),
  SwapState,
}
impl Default for Modes { fn default() -> Self { Self::SwapState } }
impl Modes {

  pub fn new(program: &mut GraphProgram, int: usize) -> Self {
    match int {
      0 => Self::AddRemove(addremove::AddRemove::create(program)),
      1 => Self::Drag(drag::Drag::create(program)),
      2 => Self::Play(play::Play::create(program)),
      3 => Self::Set(set::Set::create(program)),
      4 => Self::Analyze(analyze::Analyze::create(program)),
      5 => Self::Bubbles(bubbles::Bubbles::create(program)),
      _ => unreachable!()
    }
  }
  
  pub fn as_int(&self) -> usize {
    match self {
      Self::AddRemove(_) => 0,
      Self::Drag(_) => 1,
      Self::Play(_)  => 2,
      Self::Set(_) => 3,
      Self::Analyze(_) => 4,
      Self::Bubbles(_) => 5,
      Self::SwapState => unreachable!(),
    }
  }

  pub fn list_modes() -> &'static[&'static str] {
    &[
      "Add/Remove",
      "Drag",
      "Play",
      "Set",
      "Analyze",
      "Bubbles"
    ]
  }


}
impl Modes {

  pub fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {
    match self {
      Self::AddRemove(inside) => inside.ui(program, ui),
      Self::Drag(inside) => inside.ui(program, ui),
      Self::Play(inside) => inside.ui(program, ui),
      Self::Set(inside) => inside.ui(program, ui),
      Self::Analyze(inside) => inside.ui(program, ui),
      Self::Bubbles(inside) => inside.ui(program, ui),
      Self::SwapState => unreachable!(),
    }
  }

  pub fn tick(&mut self, program: &mut GraphProgram) {
    match self {
      Self::AddRemove(inside) => inside.tick(program),
      Self::Drag(inside) => inside.tick(program),
      Self::Play(inside) => inside.tick(program),
      Self::Set(inside) => inside.tick(program),
      Self::Analyze(inside) => inside.tick(program),
      Self::Bubbles(inside) => inside.tick(program),
      Self::SwapState => unreachable!(),
    }
  }

  pub fn interactions(&mut self, program: &mut GraphProgram) {
    match self {
      Self::AddRemove(inside) => inside.interactions(program),
      Self::Drag(inside) => inside.interactions(program),
      Self::Play(inside) => inside.interactions(program),
      Self::Set(inside) => inside.interactions(program),
      Self::Analyze(inside) => inside.interactions(program),
      Self::Bubbles(inside) => inside.interactions(program),
      Self::SwapState => unreachable!(),
    }
  }

}

mod common {
  pub(crate) use crate::GraphProgram;
  pub use macroquad::ui::Ui;
  pub use macroquad::ui::hash;
  pub use macroquad::prelude::*;
  pub use crate::utilities::StrType;
}
