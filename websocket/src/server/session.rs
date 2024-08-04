use crate::json::{Command, SocketRequest};
use crate::server::send_message;

#[derive(Clone, Debug)]
pub struct SocketSession {
    pub addr: std::net::SocketAddr,
    pub frame: tokio::sync::mpsc::UnboundedSender<SocketRequest>,
    // Last heartbeat received
    hb: std::time::Instant,
    pub name: Option<String>,
}

impl SocketSession {
    pub fn new(
        addr: std::net::SocketAddr,
        frame: tokio::sync::mpsc::UnboundedSender<SocketRequest>,
    ) -> Self {
        Self {
            addr,
            frame,
            hb: std::time::Instant::now(),
            name: None,
        }
    }

    pub fn heartbeat(&self) -> Result<(), SocketRequest> {
        if std::time::Instant::now().duration_since(self.hb) > std::time::Duration::new(45, 0) {
            log::trace!("[{}] client heartbeat failed, disconnecting!", self.addr);

            // Send close event
            return Err(SocketRequest { opcode: 8, d: None });
        }

        // Send ping event
        send_message(&self.frame, SocketRequest { opcode: 9, d: None });

        Ok(())
    }

    pub fn refresh_hb(&mut self) {
        self.hb = std::time::Instant::now();
    }
}

pub async fn handle_client(
    session: &mut SocketSession,
    event: web_socket::Event,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>,
    ws_writer: &mut web_socket::WebSocket<tokio::net::tcp::OwnedWriteHalf>,
) -> Result<(), ()> {
    match event {
        web_socket::Event::Data { data, .. } => {
            crate::events::handle(session, data, cmd_tx).await;
        }
        web_socket::Event::Ping(_) => {
            ws_writer.send_pong("p").await;
        }
        web_socket::Event::Pong(_) => session.refresh_hb(),
        web_socket::Event::Error(_) | web_socket::Event::Close { .. } => {
            send_message(&cmd_tx, Command::RemoveUser { addr: session.addr });

            return Err(());
        }
    }

    Ok(())
}
