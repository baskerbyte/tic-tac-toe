use std::env::var;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WebSocketSettings {
    pub host: String,
    pub port: u16
}

impl Default for WebSocketSettings {
    fn default() -> Self {
        Self {
            host: var("WEBSOCKET_HOST")
                .unwrap_or("localhost".to_string()),
            port: var("WEBSOCKET_PORT")
                .unwrap_or("9002".to_string())
                .parse().unwrap(),
        }
    }
}

impl super::Protocol for WebSocketSettings {
    fn get_host(&self) -> &String {
        &self.host
    }

    fn get_port(&self) -> &u16 {
        &self.port
    }
}