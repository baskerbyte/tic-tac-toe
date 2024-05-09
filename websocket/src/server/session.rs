use tokio::sync::mpsc::error::SendError;

use crate::json::{EventData, SocketRequest};

#[derive(Debug)]
pub struct Room {
    pub tray: [[char; 3]; 3],
    pub player1: PlayerSession,
    pub player2: Option<PlayerSession>,
}

#[derive(Debug)]
pub struct PlayerSession {
    pub addr: std::net::SocketAddr,
    pub frame: tokio::sync::mpsc::UnboundedSender<SocketRequest>,
}

#[derive(Clone)]
pub struct SocketSession {
    pub addr: std::net::SocketAddr,
    pub frame: tokio::sync::mpsc::UnboundedSender<SocketRequest>,
    // Last heartbeat received
    hb: std::time::Instant,
}

impl Room {
    pub fn new(player1: PlayerSession, player2: Option<PlayerSession>) -> Self {
        Self {
            tray: [[' '; 3]; 3],
            player1,
            player2,
        }
    }

    pub fn is_available(&self) -> bool {
        self.player2.is_none()
    }

    pub fn mark_position(&mut self, is_player1: bool, (x, y): (usize, usize)) {
        if self.tray[x][y] != ' ' {
            return;
        }

        self.tray[x][y] = if is_player1 { 'X' } else { 'O' };
    }

    pub fn is_full(&self) -> bool {
        for row in &self.tray {
            for cell in row {
                if cell == &' ' {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_win(&self) -> bool {
        for i in 0..=2 {
            // Check horizontal and vertical lines
            if is_line_equal(self.tray[i][0], self.tray[i][1], self.tray[i][2])
                || is_line_equal(self.tray[0][i], self.tray[1][i], self.tray[2][i])
            {
                return true;
            }
        }

        // Check diagonals
        if is_line_equal(self.tray[0][0], self.tray[1][1], self.tray[2][2])
            || is_line_equal(self.tray[2][0], self.tray[1][1], self.tray[0][2])
        {
            return true;
        }

        false
    }

    pub fn reset(&mut self) {
        for row in self.tray.iter_mut() {
            for cell in row.iter_mut() {
                *cell = ' ';
            }
        }
    }

    pub fn reply_event(&self, event: SocketRequest) {
        self.player1.frame.send(event.clone());

        if let Some(player2) = &self.player2 {
            player2.frame.send(event);
        }
    }
}

impl PlayerSession {
    pub fn new(
        addr: std::net::SocketAddr,
        frame: tokio::sync::mpsc::UnboundedSender<SocketRequest>,
    ) -> Self {
        Self { addr, frame }
    }
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
        }
    }

    pub fn heartbeat(&self) -> Result<(), SendError<SocketRequest>> {
        if std::time::Instant::now().duration_since(self.hb) > std::time::Duration::new(10, 0) {
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

    pub fn send_hello(
        &self,
        interval: &tokio::time::Interval,
    ) -> Result<(), SendError<SocketRequest>> {
        self.frame.send(SocketRequest {
            opcode: 10,
            d: Some(EventData::HelloEvent {
                heartbeat_interval: interval.period().as_millis(),
            }),
        })
    }
}

fn is_line_equal(a: char, b: char, c: char) -> bool {
    a == b && b == c && a != ' '
}
