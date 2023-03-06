
use std::collections::HashMap;

use actix::{prelude::{Actor, Context}, Handler, Recipient};
use uuid::Uuid;

use crate::{utils, types};

type Client = Recipient<types::SimpleMessage>;

pub struct Game {
  connections: HashMap<Uuid, Client>,
  r#type: types::GameType,
}

impl Game {
  pub fn new(r#type: types::GameType) -> Game {
    Game {
      connections: HashMap::new(),
      r#type,
    }
  }

  pub fn send_message(&self, message: &str, recipient_id: &Uuid) {
    if let Some(socket_recipient) = self.connections.get(recipient_id) {
      socket_recipient.do_send(types::SimpleMessage(message.to_owned()));
    } else {
      println!("Couldn't find connection id: '{}' when sending message", recipient_id);
    }
  }

  pub fn broadcast_message(&self, message: &str) {
    self.connections.values().for_each(
      |client_addr: &Recipient<types::SimpleMessage>| client_addr.do_send(types::SimpleMessage(message.to_owned())));
  }
}

impl Actor for Game {
  type Context = Context<Self>;
}

impl Handler<types::DisconnectMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: types::DisconnectMessage, _: &mut Self::Context) -> Self::Result {
      self.connections.remove(&msg.player_like.client_uuid);

      self.connections.keys().for_each(|conn_id: &Uuid| {
        // For now we do this probably expensive operation to define broadcast_connect_message every iteration,
        // this should be moved outside of scope if possible and used for each iteration.
        let broadcast_connect_message: types::JsonMessage = types::JsonMessage {
          r#type: types::Type::ClientDisconnected,
          data: types::Data::PlayerLike(
            types::JsonPlayerLike { client_uuid: msg.player_like.client_uuid.to_string() }
          ),
        };
        utils::send_json_message(&self, broadcast_connect_message, &conn_id)
      })
    }
}

impl Handler<types::ConnectMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: types::ConnectMessage, _: &mut Self::Context) -> Self::Result {
      let client_connect_message: types::JsonMessage = types::JsonMessage {
        r#type: types::Type::ConnectionSuccess,
        data: types::Data::Player(
          types::JsonPlayer {
            client_uuid: msg.player.client_uuid.to_string(),
            name: msg.player.name.to_owned() }
        ),
      };
      
      self.connections.insert(msg.player.client_uuid, msg.client_addr);
      
      // Send ClientConnect to everyone but the client
      self.connections.keys().filter(|&&conn_id: &&Uuid| conn_id != msg.player.client_uuid)
      .for_each(|conn_id: &Uuid| {
        // For now we do this probably expensive operation to define broadcast_connect_message every iteration,
        // this should be moved outside of scope if possible and used for each iteration.
        let broadcast_connect_message: types::JsonMessage = types::JsonMessage {
          r#type: types::Type::ClientConnected,
          data: types::Data::Player(
            types::JsonPlayer {
              client_uuid: msg.player.client_uuid.to_string(),
              name: msg.player.name.to_owned() }
          ),
        };
        utils::send_json_message(&self, broadcast_connect_message, &conn_id)
      });

      // Send ConnectionSuccess to the client
      utils::send_json_message(&self, client_connect_message, &msg.player.client_uuid);
    }
}

impl Handler<types::IncomingMessage> for Game {
  type Result = ();

  fn handle(&mut self, msg: types::IncomingMessage, _: &mut Self::Context) -> Self::Result {
    // Check if incoming message is supported by game otherwise return some error
    // let the game handle the request
    
    match self.r#type {
      types::GameType::TicTacToe(mut game) => game.handle(msg),
    }
  }
}
