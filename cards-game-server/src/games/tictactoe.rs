use rand::Rng;
use serde_json;
use uuid::Uuid;

use crate::{error, types, utils};

use crate::games::game::Game;

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeState {
    Empty,
    O,
    X,
}

#[derive(Clone, Copy, Debug)]
pub struct TicTacToe {
    // Configuration for outside use
    pub min_players: i8,
    pub max_players: i8,

    // Game state
    board: [[NodeState; 3]; 3],

    // You have a maximum of 9 moves in TicTacToe.
    nth_move: i8,

    player_o: Option<Uuid>,
    player_x: Option<Uuid>,
}

impl Default for TicTacToe {
    fn default() -> TicTacToe {
        TicTacToe {
            max_players: 2,
            min_players: 2,

            board: [[NodeState::Empty; 3]; 3],

            nth_move: 1,

            player_o: None,
            player_x: None,
        }
    }
}

impl TicTacToe {
    // There is no message saying which player can begin this is always player X.
    pub fn init(&mut self, game: &mut Game) -> Result<(), error::GameError> {
        let mut rng = rand::thread_rng();
        let n: usize = rng.gen_range(0..2);

        for (idx, conn) in game.connections.keys().enumerate() {
            if idx == n {
                self.player_o = Some(conn.to_owned());
            } else {
                self.player_x = Some(conn.to_owned());
            }
        }

        // Surely there is a better way to do this...
        let player_o = match self.player_o {
            Some(p) => p,
            None => panic!("player_o should exist, but doesn't."),
        };

        let player_x = match self.player_x {
            Some(p) => p,
            None => panic!("player_o should exist, but doesn't."),
        };

        let message: types::JsonMessage = types::JsonMessage {
            r#type: types::Type::TicTacToeStarted,
            data: types::Data::TicTacToeStarted(types::JsonTicTacToeStarted {
                player_o: player_o.to_string(),
                player_x: player_x.to_string(),
            }),
        };

        utils::send_json_message(&game, &message, None)?;

        Ok(())
    }

    pub fn handle(
        &mut self,
        game: &mut Game,
        msg: &types::IncomingMessage,
    ) -> Result<(), error::GameError> {
        let r#move: types::JsonTicTacToeMove = serde_json::from_str(&msg.json)?;

        let illegal_move_message = types::JsonMessage {
            r#type: types::Type::TicTacToeMoveIllegal,
            data: { types::Data::TicTacToeMove(r#move) },
        };

        // Player is moving out of turn
        if self.nth_move % 2 == 0 {
            if self.player_o == Some(msg.player_like.client_uuid) {
                utils::send_json_message(&game, &illegal_move_message, self.player_o.as_ref())?;
                return Ok(());
            };
        } else if self.player_x == Some(msg.player_like.client_uuid) {
            utils::send_json_message(&game, &illegal_move_message, self.player_x.as_ref())?;
            return Ok(());
        };

        // Player is moving outside the 3x3 board
        if r#move.x > 2 || r#move.y > 2
      // Square is already occupied
      || self.board[r#move.x][r#move.y] != NodeState::Empty
        {
            utils::send_json_message(
                &game,
                &illegal_move_message,
                if self.nth_move % 2 == 0 {
                    self.player_x.as_ref()
                } else {
                    self.player_o.as_ref()
                },
            )?;
            return Ok(());
        };

        // Process the move
        self.nth_move += 1;
        self.board[r#move.x][r#move.y] = if self.nth_move % 2 == 0 {
            NodeState::X
        } else {
            NodeState::O
        };

        let move_message = types::JsonMessage {
            r#type: types::Type::TicTacToeMove,
            data: {
                types::Data::TicTacToeMoveSuccess(types::JsonTicTacToeMoveSuccess {
                    x: r#move.x,
                    y: r#move.y,
                    player: if self.nth_move % 2 == 0 {
                        types::JsonTicTacToePlayer::X
                    } else {
                        types::JsonTicTacToePlayer::O
                    },
                })
            },
        };

        utils::send_json_message(&game, &move_message, None)?;

        println!("{:?}", self.board);

        Ok(())
        // TODO: Check if the move was winning
        // TODO: Broadcast the winning player
    }
}
