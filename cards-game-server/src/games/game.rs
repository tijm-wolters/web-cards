use std::{collections::HashMap, sync::Arc};

use actix::{
    prelude::{Actor, Context},
    Handler, Recipient,
};
use rand::Rng;
use uuid::Uuid;

use crate::{error, types, utils};

type Client = Recipient<types::SimpleMessage>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeState {
    Empty,
    O,
    X,
}

#[derive(Clone, Debug)]
pub struct Game {
    connections: HashMap<Uuid, Client>,

    max_players: i8,

    // Game state
    board: [[NodeState; 3]; 3],

    // You have a maximum of 9 moves in TicTacToe.
    nth_move: i8,

    player_o: Option<Uuid>,
    player_x: Option<Uuid>,
}

impl Default for Game {
    fn default() -> Game {
        Game {
            connections: HashMap::new(),

            max_players: 2,

            board: [[NodeState::Empty; 3]; 3],

            nth_move: 0,

            player_o: None,
            player_x: None,
        }
    }
}

impl Game {
    pub fn send_message(&self, message: &str, recipient_id: &Uuid) {
        if let Some(socket_recipient) = self.connections.get(recipient_id) {
            socket_recipient.do_send(types::SimpleMessage(message.to_owned()));
        } else {
            println!(
                "Couldn't find connection id: '{}' when sending message",
                recipient_id
            );
        }
    }

    pub fn broadcast_message(&self, message: &str) {
        self.connections
            .values()
            .for_each(|client_addr: &Recipient<types::SimpleMessage>| {
                client_addr.do_send(types::SimpleMessage(message.to_owned()))
            });
    }

    fn check_notify(&mut self) -> Result<(), error::GameError> {
        if self.connections.len() >= self.max_players.try_into().unwrap() {
            let mut rng = rand::thread_rng();
            let n: usize = rng.gen_range(0..2);

            for (idx, conn) in self.connections.keys().enumerate() {
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

            utils::send_json_message(&self, &message, None)?;
        }

        Ok(())
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl Handler<types::DisconnectMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: types::DisconnectMessage, _: &mut Self::Context) -> Self::Result {
        let broadcast_connect_message: Arc<types::JsonMessage> = Arc::new(types::JsonMessage {
            r#type: types::Type::ClientDisconnected,
            data: types::Data::PlayerLike(types::JsonPlayerLike {
                client_uuid: msg.player_like.client_uuid.to_string(),
            }),
        });

        self.connections.remove(&msg.player_like.client_uuid);

        self.connections.keys().for_each(|conn_id: &Uuid| {
            if let Err(error) =
                utils::send_json_message(&self, &broadcast_connect_message.as_ref(), Some(conn_id))
            {
                eprintln!(
                    "Broadcasting message {:?} failed: {}",
                    broadcast_connect_message, error,
                );
            }
        })
    }
}

impl Handler<types::ConnectMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: types::ConnectMessage, _: &mut Self::Context) -> Self::Result {
        let broadcast_connect_message: Arc<types::JsonMessage> = Arc::new(types::JsonMessage {
            r#type: types::Type::ClientConnected,
            data: types::Data::Player(types::JsonPlayer {
                client_uuid: msg.player.client_uuid.to_string(),
                name: msg.player.name.to_owned(),
            }),
        });

        let client_connect_message: types::JsonMessage = types::JsonMessage {
            r#type: types::Type::ConnectionSuccess,
            data: types::Data::Player(types::JsonPlayer {
                client_uuid: msg.player.client_uuid.to_string(),
                name: msg.player.name.to_owned(),
            }),
        };

        self.connections
            .insert(msg.player.client_uuid, msg.client_addr);

        // Send ClientConnect to everyone but the client
        self.connections
            .keys()
            .filter(|&&conn_id: &&Uuid| conn_id != msg.player.client_uuid)
            .for_each(|conn_id: &Uuid| {
                if let Err(error) = utils::send_json_message(
                    &self,
                    &broadcast_connect_message.as_ref(),
                    Some(conn_id),
                ) {
                    eprintln!("Something went wrong: {}", error);
                }
            });

        // Send ConnectionSuccess to the client
        if let Err(error) = utils::send_json_message(
            &self,
            &client_connect_message,
            Some(&msg.player.client_uuid),
        ) {
            eprintln!("Something went wrong: {}", error);
        }

        // Check if we should initialize the game
        if let Err(error) = self.check_notify() {
            eprintln!("Something went wrong: {}", error);
        }
    }
}

impl Handler<types::IncomingMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: types::IncomingMessage, _: &mut Self::Context) -> Self::Result {
        // TODO: Don't just unwrap, return error to the client if json is malformed.
        let r#move: types::JsonTicTacToeMove = serde_json::from_str(&msg.json).unwrap();

        let illegal_move_message = types::JsonMessage {
            r#type: types::Type::TicTacToeMoveIllegal,
            data: { types::Data::TicTacToeMove(r#move) },
        };

        // Player is moving out of turn
        if self.nth_move % 2 == 0 {
            if self.player_o == Some(msg.player_like.client_uuid) {
                utils::send_json_message(&self, &illegal_move_message, self.player_o.as_ref());
                return;
            };
        } else if self.player_x == Some(msg.player_like.client_uuid) {
            utils::send_json_message(&self, &illegal_move_message, self.player_x.as_ref());
            return;
        };

        // Player is moving outside the 3x3 board
        if r#move.x > 2 || r#move.y > 2
      // Square is already occupied
      || self.board[r#move.x][r#move.y] != NodeState::Empty
        {
            utils::send_json_message(
                &self,
                &illegal_move_message,
                if self.nth_move % 2 == 0 {
                    self.player_x.as_ref()
                } else {
                    self.player_o.as_ref()
                },
            );
            return;
        };

        // Process the move
        self.board[r#move.x][r#move.y] = if self.nth_move % 2 == 0 {
            NodeState::X
        } else {
            NodeState::O
        };
        self.nth_move += 1;

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

        utils::send_json_message(&self, &move_message, None);

        println!("{:?}", self.board);

        // TODO: Check if the move was winning
        // TODO: Broadcast the winning player
    }
}
