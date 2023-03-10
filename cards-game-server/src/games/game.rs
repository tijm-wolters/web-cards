use std::{collections::HashMap, sync::Arc};

use actix::{
    prelude::{Actor, Context},
    Handler, Recipient,
};
use uuid::Uuid;

use crate::{error, types, utils};

type Client = Recipient<types::SimpleMessage>;

pub struct Game {
    pub connections: HashMap<Uuid, Client>,
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
        // Allow clippy rule as more games will be addedTM
        #[allow(clippy::infallible_destructuring_match)]
        let mut game = match self.r#type {
            types::GameType::TicTacToe(game) => game,
        };

        if self.connections.len() >= game.max_players.try_into().unwrap() {
            game.init(self)?;
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
        match self.r#type {
            types::GameType::TicTacToe(mut game) => {
                if let Err(error) = game.handle(self, &msg) {
                    eprintln!("Something went wrong: {}", error);
                }
            }
        }
    }
}
