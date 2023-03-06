use crate::types::IncomingMessage;

#[derive(Clone, Copy)]
enum NodeState {
  Empty,
  O,
  X,
}

#[derive(Clone, Copy)]
pub struct TicTacToe {
  board: [[NodeState; 3]; 3],
}

impl Default for TicTacToe {
  fn default() -> TicTacToe {
    TicTacToe {
      board: [[NodeState::Empty; 3]; 3],
    }
  }
}

impl TicTacToe {
  pub fn handle(&mut self, msg: IncomingMessage) {
    todo!();
  }
}