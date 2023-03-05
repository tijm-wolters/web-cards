
use std::collections::HashMap;

use actix::{prelude::{Actor, Context}, Handler, Recipient};
use uuid::Uuid;

use crate::message::{WsMessage, GameMessage, Disconnect, Connect};

type Socket = Recipient<WsMessage>;

#[derive(Default)]
pub struct Game {
  connections: HashMap<Uuid, Socket>,
}

impl Game {
  fn send_message(&self, message: &str, recipient_id: &Uuid) {
    if let Some(socket_recipient) = self.connections.get(recipient_id) {
      socket_recipient.do_send(WsMessage(message.to_owned()));
    } else {
      println!("Couldn't find connection id: '{id}' when sending message", id=recipient_id);
    }
  }

  fn broadcast_message(&self, message: &str) {
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
    }
}

impl Handler<Connect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
      self.connections.insert(msg.client_id, msg.client_addr);

      self.connections.keys().filter(|&&conn_id: &&Uuid| conn_id != msg.client_id)
      .for_each(|conn_id: &Uuid| self.send_message(&format!("{} connected", msg.client_id), conn_id));

      self.send_message(&format!("connected as {}", msg.client_id), &msg.client_id);
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
