use actix::{Actor, Addr};
use actix_web::{middleware, App, HttpServer, web};

use crate::{api::websocket, games::{game, tictactoe::TicTacToe}, types};

#[actix_web::main]
pub async fn start_webserver() -> std::io::Result<()> {
  let game_server: Addr<game::Game> = game::Game::new(
    types::GameType::TicTacToe(TicTacToe::default())
  ).start();
  
  HttpServer::new(move || {
    App::new()
    .route("/ws/{name}", web::get().to(websocket::index))
    .app_data(web::Data::new(game_server.clone()))
    .wrap(middleware::Logger::default())
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}