use actix::prelude::{Message, Recipient};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
  pub client_addr: Recipient<WsMessage>,
  pub client_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub client_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameMessage {
  pub client_id: Uuid,
  pub json: String,
}
