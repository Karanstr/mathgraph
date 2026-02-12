mod blueprint;
mod play;
mod set;
mod analyze;
mod bubbles;

use eframe::egui::{Response, Ui};

use crate::{GraphProgram};

trait Mode where Self: Sized {

  fn create(program: &GraphProgram) -> Self;
  fn ui(&mut self, _program: &mut GraphProgram, _ui: &mut Ui) {}
  fn tick(&mut self, _program: &mut GraphProgram) {}
  fn interactions(&mut self, _program: &mut GraphProgram, _response: Response) {}

}
pub enum Modes {
  Blueprint(blueprint::Blueprint),
  Play(play::Play),
  Set(set::Set),
  Analyze(analyze::Analyze),
  Bubbles(bubbles::Bubbles),
  SwapState,
}
impl Default for Modes { fn default() -> Self { Self::SwapState } }
impl std::fmt::Display for Modes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let str = match self {
      Self::Blueprint(_) => "Blueprint",
      Self::Play(_) => "Play",
      Self::Set(_) => "Set",
      Self::Analyze(_) => "Analyze",
      Self::Bubbles(_) => "Bubbles",
      Self::SwapState => "Invalid Mode!!"
    };
    write!(f, "{}", str)
  }
}
impl Modes {

  pub fn new(program: &GraphProgram, int: usize) -> Self {
    match int {
      0 => Self::Blueprint(blueprint::Blueprint::create(program)),
      1 => Self::Play(play::Play::create(program)),
      2 => Self::Set(set::Set::create(program)),
      3 => Self::Analyze(analyze::Analyze::create(program)),
      4 => Self::Bubbles(bubbles::Bubbles::create(program)),
      _ => unreachable!()
    }
  }
  
  pub fn as_int(&self) -> usize {
    match self {
      Self::Blueprint(_) => 0,
      Self::Play(_)  => 1,
      Self::Set(_) => 2,
      Self::Analyze(_) => 3,
      Self::Bubbles(_) => 4,
      Self::SwapState => unreachable!(),
    }
  }

}
impl Modes {

  pub fn ui(&mut self, program: &mut GraphProgram, ui: &mut Ui) {
    match self {
      Self::Blueprint(inside) => inside.ui(program, ui),
      Self::Play(inside) => inside.ui(program, ui),
      Self::Set(inside) => inside.ui(program, ui),
      Self::Analyze(inside) => inside.ui(program, ui),
      Self::Bubbles(inside) => inside.ui(program, ui),
      Self::SwapState => unreachable!(),
    }
  }

  pub fn tick(&mut self, program: &mut GraphProgram) {
    match self {
      Self::Blueprint(inside) => inside.tick(program),
      Self::Play(inside) => inside.tick(program),
      Self::Set(inside) => inside.tick(program),
      Self::Analyze(inside) => inside.tick(program),
      Self::Bubbles(inside) => inside.tick(program),
      Self::SwapState => unreachable!(),
    }
  }

  pub fn interactions(&mut self, program: &mut GraphProgram, response: Response) {
    match self {
      Self::Blueprint(inside) => inside.interactions(program, response),
      Self::Play(inside) => inside.interactions(program, response),
      Self::Set(inside) => inside.interactions(program, response),
      Self::Analyze(inside) => inside.interactions(program, response),
      Self::Bubbles(inside) => inside.interactions(program, response),
      Self::SwapState => unreachable!(),
    }
  }

}

mod common {
  pub(crate) use crate::GraphProgram;
  pub use eframe::egui::Ui;
  pub use eframe::egui::{Key, PointerButton, Response, DragValue, Widget, Id};
}
