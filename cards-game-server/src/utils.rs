use serde::Serialize;
use uuid::Uuid;

use crate::{error, games::game::Game};

// Converts the message to a JSON string and sends it over the WebSocket connection.
pub fn send_json_message<T>(
    game: &&mut Game,
    message: &T,
    recipient_id: Option<&Uuid>,
) -> Result<(), error::GameError>
where
    T: Serialize,
{
    let message = serde_json::to_string(message)?;
    // {
    //     Ok(message) => message,
    //     Err(e) => panic!("Something went wrong: {}", e),
    // };

    if let Some(id) = recipient_id {
        game.send_message(&message, id);
    } else {
        game.broadcast_message(&message);
    }

    Ok(())
}
