use actix::{fut, Actor, Addr, StreamHandler, AsyncContext, Running, Handler, WrapFuture, ActorFutureExt, ActorContext, ContextFutureSpawner};
use actix_web::{get, web::{self, Data}, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{time::{Duration, Instant}, collections::HashMap};
use uuid::Uuid;

use crate::{game::Game, message::{Connect, Disconnect, WsMessage, GameMessage}};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
  game_addr: Addr<Game>,
  heart_beat: Instant,
  conn_id: Uuid,
}

impl WebSocket {
  pub fn new(game: Addr<Game>) -> WebSocket {
    WebSocket { game_addr: game, heart_beat: Instant::now(), conn_id: Uuid::new_v4() }
  }

  pub fn heart_beat(&self, ctx: &mut ws::WebsocketContext<Self>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |conn, ctx| {
      if Instant::now().duration_since(conn.heart_beat) > CLIENT_TIMEOUT {
        conn.game_addr.do_send(Disconnect { client_id: conn.conn_id });
        ctx.stop();
        return;
      }

      ctx.ping(b"Ping!");
    });
  }
}

impl Actor for WebSocket {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.heart_beat(ctx);

    let client_addr = ctx.address();
    self.game_addr.send(Connect {
      client_addr: client_addr.recipient(),
      client_id: self.conn_id,
    }).into_actor(self)
    .then(|res, _, ctx| {
      match res {
          Ok(_res) => (),
          _ => ctx.stop(),
      }
      fut::ready(())
    })
    .wait(ctx);
  }

  fn stopping(&mut self, _: &mut Self::Context) -> Running {
    self.game_addr.do_send(Disconnect { client_id: self.conn_id });
    Running::Stop
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Ping(msg)) => {
        self.heart_beat = Instant::now();
        ctx.pong(&msg);
      },
      Ok(ws::Message::Pong(_)) => {
        self.heart_beat = Instant::now();
      }
      Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
      Ok(ws::Message::Text(text)) => self.game_addr.do_send(GameMessage {
        client_id: self.conn_id,
        json: text.to_string(),
      }),
      _ => (),
    }
  }
}

#[get("/ws/{id}")]
pub async fn index(
    req: HttpRequest,
    stream: web::Payload,
    game_id: web::Path<String>,
    game_pool: Data<HashMap<Uuid, Addr<Game>>>,
  ) -> Result<HttpResponse, Error> {
  let game_uuid = Uuid::parse_str(&game_id);
  if game_uuid.is_err() {
    panic!("Malformed uuid"); // Some kind of HTTP error should be implemented
  }

  // It doesn't let me do this comparrison :(
  let game_addr = game_pool.iter().find(|pool_item: &(&Uuid, &Addr<Game>)| pool_item.0 == game_uuid);

  todo!()
  // ws::start(WebSocket::new(), &req, stream)
}

impl Handler<WsMessage> for WebSocket {
  type Result = ();

  fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
      ctx.text(msg.0);
  }
}
