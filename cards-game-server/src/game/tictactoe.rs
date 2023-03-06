use serde::{Deserialize, Serialize};
use serde_json;

use crate::types;

#[derive(Clone, Copy)]
enum NodeState {
  Empty,
  O,
  X,
}

#[derive(Serialize, Deserialize)]
struct Move {
  x: i8,
  y: i8,
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
  pub fn handle_join(&mut self) {
    todo!()
  }

  pub fn handle_message(&mut self, msg: &types::IncomingMessage) -> Result<String, String> {
    todo!()
  }
}