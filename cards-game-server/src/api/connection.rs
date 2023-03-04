use std::collections::HashMap;

use actix::Addr;
use actix_web::{middleware, App, HttpServer, web::Data};
use uuid::Uuid;

use crate::{api::websockets, game::Game};

#[actix_web::main]
pub async fn start_webserver() -> std::io::Result<()> {
  let game_pool: HashMap<Uuid, Addr<Game>> = HashMap::new();

  HttpServer::new(move || {
    App::new()
    .service(websockets::index)
    .app_data(Data::new(game_pool.clone()))
    .wrap(middleware::Logger::default())
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}