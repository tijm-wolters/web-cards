use actix::{Actor, Addr};
use actix_web::{middleware, web, App, HttpServer};

use crate::{api::websocket, games::game::Game};

#[actix_web::main]
pub async fn start_webserver() -> std::io::Result<()> {
    let game_server: Addr<Game> = Game::default().start();

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
