use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{start, Message, ProtocolError, WebsocketContext};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{
    games::game::TicTacToe,
    types::{
        ConnectMessage, DisconnectMessage, IncomingMessage, Player, PlayerLike, SimpleMessage,
    },
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    game_addr: Addr<TicTacToe>,
    heart_beat: Instant,
    player: Player,
}

impl WebSocket {
    pub fn new(game: Addr<TicTacToe>, name: String) -> WebSocket {
        WebSocket {
            game_addr: game,
            heart_beat: Instant::now(),
            player: Player {
                client_uuid: Uuid::new_v4(),
                name,
            },
        }
    }

    pub fn heart_beat(&self, ctx: &mut WebsocketContext<Self>) {
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
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heart_beat(ctx);

        let client_addr = ctx.address();
        self.game_addr
            .send(ConnectMessage {
                client_addr: client_addr.recipient(),
                player: Player {
                    client_uuid: self.player.client_uuid,
                    name: self.player.name.to_owned(),
                },
            })
            .into_actor(self)
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
        self.game_addr.do_send(DisconnectMessage {
            player_like: PlayerLike {
                client_uuid: self.player.client_uuid,
            },
        });
        Running::Stop
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {
                self.heart_beat = Instant::now();
            }
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            Ok(Message::Text(text)) => self.game_addr.do_send(IncomingMessage {
                player_like: {
                    PlayerLike {
                        client_uuid: self.player.client_uuid,
                    }
                },
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
    game: web::Data<Addr<TicTacToe>>,
) -> Result<HttpResponse, Error> {
    start(
        WebSocket::new(game.get_ref().clone(), name.to_owned()),
        &req,
        stream,
    )
}

impl Handler<SimpleMessage> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: SimpleMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
