use crate::json::{EventData, SocketRequest};
use crate::server::send_message;
use crate::server::session::SocketSession;

#[derive(Debug)]
pub struct Room {
    pub tray: [[char; 3]; 3],
    pub player1: Option<SocketSession>,
    pub player2: Option<SocketSession>,
    pub player1_turn: bool,
    pub duration_turn: std::time::Instant,
}

impl Room {
    pub fn new(player1: Option<SocketSession>, player2: Option<SocketSession>) -> Self {
        Self {
            tray: [[' '; 3]; 3],
            player1,
            player2,
            player1_turn: true,
            duration_turn: std::time::Instant::now() + std::time::Duration::from_secs(8),
        }
    }

    pub fn is_available(&self) -> bool {
        self.player1.is_none() || self.player2.is_none()
    }

    pub fn find_player(&self, addr: std::net::SocketAddr) -> bool {
        is_player(&self.player1, addr) ||
            is_player(&self.player2, addr)
    }

    pub fn mark_position(
        &mut self,
        is_player1: bool,
        (x, y): (usize, usize),
    ) -> Result<(), SocketRequest> {
        if x < 0 || x > 2 || y < 0 || y > 2 {
            return Err(SocketRequest::new(1007, Some(EventData::Message("invalid position".to_string()))));
        }

        if self.player1_turn && !is_player1 || !self.player1_turn && is_player1 {
            return Err(SocketRequest::new(1007, Some(EventData::Message("not your turn".to_string()))));
        }

        if self.tray[x][y] != ' ' {
            return Err(SocketRequest::new(1007, Some(EventData::Message("position already taken".to_string()))));
        }

        self.tray[x][y] = if is_player1 { 'X' } else { 'O' };

        Ok(())
    }

    pub fn is_full(&self) -> bool {
        !self.tray.iter().any(|row| row.iter().any(|col| col == &' '))
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

    pub fn reply_event(&self, event: SocketRequest) {
        if let (Some(player1), Some(player2)) = (&self.player1, &self.player2) {
            send_message(&player1.frame, event.clone());
            send_message(&player2.frame, event);
        }
    }

    pub fn refresh_turn(&mut self) {
        self.duration_turn = std::time::Instant::now();
        self.player1_turn = !self.player1_turn;
    }
    
    pub fn timer(&self) {
        if std::time::Instant::now().duration_since(self.duration_turn) > std::time::Duration::new(30, 0) {
            let player = if self.player1_turn {
                &self.player1
            } else {
                &self.player2
            };

            log::trace!("[{}] disconnected due to inactivity", player.as_ref().unwrap().addr);

            send_message(
                &player.as_ref().unwrap().frame,
                SocketRequest { opcode: 8, d: None }
            );
        }
    }
}

fn is_line_equal(a: char, b: char, c: char) -> bool {
    a == b && b == c && a != ' '
}

pub fn is_player(player: &Option<SocketSession>, addr: std::net::SocketAddr) -> bool {
    player.as_ref().map_or(false, |session| session.addr == addr)
}