use actix::{Actor, Addr};
use actix_web::{middleware, App, HttpServer, web};

use crate::{api::websocket, game::game};

#[actix_web::main]
pub async fn start_webserver() -> std::io::Result<()> {
  let game_server: Addr<game::Game> = game::Game::new(game::GameType::TicTacToe).start();
  
  HttpServer::new(move || {
    App::new()
    .route("/ws", web::get().to(websocket::index))
    .app_data(web::Data::new(game_server.clone()))
    .wrap(middleware::Logger::default())
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}