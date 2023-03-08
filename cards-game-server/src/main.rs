use cards_game_server::api;

fn main() -> std::io::Result<()> {
    api::connection::start_webserver()
}
