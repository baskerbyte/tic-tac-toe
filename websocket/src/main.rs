pub mod server;
pub mod json;
mod events;
mod commands;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    server::App::new().run()
        .await
}
