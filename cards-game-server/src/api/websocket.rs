use actix::{fut, Actor, Addr, StreamHandler, AsyncContext, Running, Handler, WrapFuture, ActorFutureExt, ActorContext, ContextFutureSpawner};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{games::game::Game, types};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
  game_addr: Addr<Game>,
  heart_beat: Instant,
  player: types::Player,
}

impl WebSocket {
  pub fn new(game: Addr<Game>, name: String) -> WebSocket {
    WebSocket {
      game_addr: game,
      heart_beat: Instant::now(),
      player: types::Player {
        client_uuid: Uuid::new_v4(),
        name,
      }
    }
  }

  pub fn heart_beat(&self, ctx: &mut ws::WebsocketContext<Self>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |conn, ctx| {
      if Instant::now().duration_since(conn.heart_beat) > CLIENT_TIMEOUT {
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
    self.game_addr.send(types::ConnectMessage {
      client_addr: client_addr.recipient(),
      player: types::Player {
        client_uuid: self.player.client_uuid,
        name: self.player.name.to_owned(),
      },
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
    self.game_addr.do_send(types::DisconnectMessage {
      player_like: types::PlayerLike {
        client_uuid: self.player.client_uuid,
      }
    });
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
      Ok(ws::Message::Text(text)) => self.game_addr.do_send(types::IncomingMessage {
        player_like: { types::PlayerLike { client_uuid: self.player.client_uuid }},
        json: text.to_string(),
      }),
      _ => (),
    }
  }
}

pub async fn index(
    req: HttpRequest,
    stream: web::Payload,
    name: web::Path<String>,
    game: web::Data<Addr<Game>>,
  ) -> Result<HttpResponse, Error> {
  ws::start(WebSocket::new(game.get_ref().clone(), name.to_owned()), &req, stream)
}

impl Handler<types::SimpleMessage> for WebSocket {
  type Result = ();

  fn handle(&mut self, msg: types::SimpleMessage, ctx: &mut Self::Context) {
      ctx.text(msg.0);
  }
}
