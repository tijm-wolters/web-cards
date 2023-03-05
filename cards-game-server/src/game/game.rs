
use std::collections::HashMap;

use actix::{prelude::{Actor, Context}, Handler, Recipient};
use uuid::Uuid;

use crate::{message::{WsMessage, GameMessage, Disconnect, Connect}, game::json, utils};

type Socket = Recipient<WsMessage>;

#[derive(Default)]
pub struct Game {
  connections: HashMap<Uuid, Socket>,
}

impl Game {
  pub fn send_message(&self, message: &str, recipient_id: &Uuid) {
    if let Some(socket_recipient) = self.connections.get(recipient_id) {
      socket_recipient.do_send(WsMessage(message.to_owned()));
    } else {
      println!("Couldn't find connection id: '{}' when sending message", recipient_id);
    }
  }

  pub fn broadcast_message(&self, message: &str) {
    self.connections.values().for_each(
      |client_addr: &Recipient<WsMessage>| client_addr.do_send(WsMessage(message.to_owned())));
  }
}

impl Actor for Game {
  type Context = Context<Self>;
}

impl Handler<Disconnect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
      self.connections.remove(&msg.client_id);

      self.connections.keys().for_each(|conn_id: &Uuid| {
        // For now we do this probably expensive operation to define broadcast_connect_message every iteration,
        // this should be moved outside of scope if possible and used for each iteration.
        let broadcast_connect_message: json::JsonMessage = json::JsonMessage {
          r#type: json::Type::ClientDisconnected,
          data: Some(json::Data::ConnectionData { client_uuid: Some(msg.client_id.to_string()) }),
        };
        utils::send_json_message(&self, broadcast_connect_message, &conn_id)
      })
    }
}

impl Handler<Connect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
      let client_connect_message: json::JsonMessage = json::JsonMessage {
        r#type: json::Type::ConnectionSuccess,
        data: Some(json::Data::ConnectionData { client_uuid: Some(msg.client_id.to_string()) }),
      };
      
      self.connections.insert(msg.client_id, msg.client_addr);
      
      // Send ClientConnect to everyone but the client
      self.connections.keys().filter(|&&conn_id: &&Uuid| conn_id != msg.client_id)
      .for_each(|conn_id: &Uuid| {
        // For now we do this probably expensive operation to define broadcast_connect_message every iteration,
        // this should be moved outside of scope if possible and used for each iteration.
        let broadcast_connect_message: json::JsonMessage = json::JsonMessage {
          r#type: json::Type::ClientConnected,
          data: Some(json::Data::ConnectionData { client_uuid: Some(msg.client_id.to_string()) }),
        };
        utils::send_json_message(&self, broadcast_connect_message, &conn_id)
      });

      // Send ConnectionSuccess to the client
      utils::send_json_message(&self, client_connect_message, &msg.client_id);
    }
}

impl Handler<GameMessage> for Game {
  type Result = ();

  fn handle(&mut self, msg: GameMessage, _: &mut Self::Context) -> Self::Result {
    // Perform game logic
    // Send response to all clients

    // one to many impl for right now.
    self.broadcast_message(&format!("Broadcast from the server: {}", msg.json));
  }
}
