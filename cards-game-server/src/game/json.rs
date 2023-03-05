use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Type {
  ConnectionSuccess, // Server to the connected client
  ClientConnected, // Broadcasted message about connected client
  ClientDisconnected, // Broadcasted message about disconnected client
}

#[derive(Serialize, Deserialize)]
pub enum Data {
  ConnectionData { client_uuid: Option<String> },
}

#[derive(Serialize, Deserialize)]
pub struct JsonMessage {
  pub r#type: Type,
  pub data: Option<Data>,
}