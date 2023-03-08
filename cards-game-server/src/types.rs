use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::games;

//
// ------------------ WebSocket Messages ------------------
//
// These messages are used for the actual messages being
// sent over the WebSocket connection, or messages that
// are used for internal communication.
//

#[derive(Message)]
#[rtype(result = "()")]
pub struct SimpleMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectMessage {
  pub client_addr: Recipient<SimpleMessage>,
  pub player: Player,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DisconnectMessage {
  pub player_like: PlayerLike,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IncomingMessage {
  pub player_like: PlayerLike,
  pub json: String,
}

//
// ------------------ JSON Messages ------------------
//
// These are the message types that are used to define
// how incoming and outgoing messages are structured,
// it is implied these will be converted to a JSON
// string when being sent over the WebSocket connection.
//

#[derive(Serialize, Deserialize)]
pub enum Type {
  //
  // ------------------ Generics ------------------
  // 

  // Will be sent to one client when it has connected.
  ConnectionSuccess,
  // Will be sent to all clients when a new client has connected to the game.
  ClientConnected,
  // Will be sent to all clients when a client has disconnected from the game.
  ClientDisconnected,
  
  //
  // ------------------ TicTacToe ------------------
  //

  TicTacToeStarted,
  TicTacToeMove,
  TicTacToeMoveIllegal,
  TicTacToeWinner,
  TicTacToeDraw,
}

#[derive(Serialize, Deserialize)]
pub enum Data {
  Player(JsonPlayer),
  PlayerLike(JsonPlayerLike),
  TicTacToeStarted(JsonTicTacToeStarted),
  TicTacToeMove(JsonTicTacToeMove),
  TicTacToeMoveSuccess(JsonTicTacToeMoveSuccess),
}

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
  pub r#type: Type,
  pub data: Data,
}

//
// ------------------ Generics ------------------
//
// These are the types used across the entire
// application for various purposes.
//
// We have JSON Types and regular types, this is
// because the regular types can include types
// that don't meet the criteria for
// serde::Serialize or serde::Deserialize.
//

#[derive(Serialize, Deserialize)]
pub struct JsonPlayer {
  pub client_uuid: String,
  pub name: String,
}

pub struct Player {
  pub client_uuid: Uuid,
  pub name: String,
}

// Used when you only communicating the identifier, happens mostly internally.
#[derive(Serialize, Deserialize)]
pub struct JsonPlayerLike {
  pub client_uuid: String,
}

pub struct PlayerLike {
  pub client_uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct JsonTicTacToeStarted {
  pub player_o: String,
  pub player_x: String,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct JsonTicTacToeMove {
  pub x: usize,
  pub y: usize,
}

#[derive(Serialize, Deserialize)]
pub enum JsonTicTacToePlayer {
  X,
  O,
}

#[derive(Serialize, Deserialize)]
pub struct JsonTicTacToeMoveSuccess {
  pub x: usize,
  pub y: usize,
  pub player: JsonTicTacToePlayer,
}

pub enum GameType {
  TicTacToe(games::tictactoe::TicTacToe),
}
