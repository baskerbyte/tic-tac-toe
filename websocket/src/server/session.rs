use crate::json::SocketRequest;
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
