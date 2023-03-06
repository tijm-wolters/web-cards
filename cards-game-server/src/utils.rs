use serde::Serialize;
use uuid::Uuid;

use crate::game::game::Game;

// Converts the message to a JSON string and sends it over the WebSocket connection.
pub fn send_json_message<T>(
    game: &&mut Game,
    message: T,
    recipient_id: &Uuid,
  ) where T: Serialize {
  let message: String = match serde_json::to_string(&message) {
    Ok(message) => message,
    Err(e) => panic!("Something went wrong: {}", e),
  };
  
  game.send_message(&message, recipient_id);
}

