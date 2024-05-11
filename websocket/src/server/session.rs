use tokio::sync::mpsc::error::SendError;

use crate::json::SocketRequest;

#[derive(Clone)]
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

    pub fn heartbeat(&self) -> Result<(), SendError<SocketRequest>> {
        if std::time::Instant::now().duration_since(self.hb) > std::time::Duration::new(45, 0) {
            log::trace!("[{}] client heartbeat failed, disconnecting!", self.addr);

            // Send close event
            return Err(SendError(SocketRequest { opcode: 8, d: None }));
        }

        // Send ping event
        self.frame.send(SocketRequest { opcode: 9, d: None })
    }

    pub fn refresh_hb(&mut self) {
        self.hb = std::time::Instant::now();
    }
}
