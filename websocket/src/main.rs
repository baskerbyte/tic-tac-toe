pub mod server;
pub mod json;
mod events;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    server::App::new().run()
        .await
}
