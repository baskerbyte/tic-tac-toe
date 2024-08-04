use crate::json::{EventData, SocketRequest};
use crate::server::send_message;
use crate::server::session::SocketSession;

#[derive(Debug)]
pub struct Room {
    pub tray: [char; 9],
    pub player1: Option<SocketSession>,
    pub player2: Option<SocketSession>,
    pub player1_turn: bool,
    pub duration_turn: Option<std::time::Instant>,
    pub code: Option<String>,
    pub name: String
}

impl Room {
    pub fn new(
        player1: Option<SocketSession>,
        player2: Option<SocketSession>,
        code: Option<String>,
        name: String
    ) -> Self {
        Self {
            tray: [' '; 9],
            player1: player1,
            player2,
            player1_turn: true,
            duration_turn: None,
            code,
            name
        }
    }

    pub fn reset(self) -> Self {
        Self::new(self.player1, self.player2, self.code, self.name)
    }

    pub fn is_available(&self) -> bool {
        self.player1.is_none() || self.player2.is_none()
    }

    pub fn find_player(&self, addr: std::net::SocketAddr) -> bool {
        is_player(&self.player1, addr) || is_player(&self.player2, addr)
    }

    pub fn mark_position(
        &mut self,
        is_player1: bool,
        position: usize,
    ) -> Result<(), SocketRequest> {
        if position < 0 || position > 9 {
            return Err(SocketRequest::new(
                1007,
                Some(EventData::Message("invalid position".to_string())),
            ));
        }

        if self.player1_turn && !is_player1 || !self.player1_turn && is_player1 {
            return Err(SocketRequest::new(
                1007,
                Some(EventData::Message("not your turn".to_string())),
            ));
        }

        if self.tray[position] != ' ' {
            return Err(SocketRequest::new(
                1007,
                Some(EventData::Message("position already taken".to_string())),
            ));
        }

        self.tray[position] = if is_player1 { 'X' } else { 'O' };

        Ok(())
    }

    pub fn is_full(&self) -> bool {
        !self.tray.iter().any(|square| square == &' ')
    }

    pub fn is_win(&self) -> bool {
        for i in 0..=2 {
            // Check horizontal and vertical lines
            if is_line_equal(self.tray[i], self.tray[i + 3], self.tray[i + 6])
                || is_line_equal(self.tray[i], self.tray[i + 1], self.tray[i + 2])
            {
                return true;
            }
        }

        // Check diagonals
        if is_line_equal(self.tray[0], self.tray[4], self.tray[8])
            || is_line_equal(self.tray[2], self.tray[4], self.tray[6])
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
        self.duration_turn = Some(std::time::Instant::now());
        self.player1_turn = !self.player1_turn;
    }

    pub fn timer(&self) {
        if let Some(duration_turn) = self.duration_turn {
            if std::time::Instant::now().duration_since(duration_turn)
                > std::time::Duration::new(30, 0)
            {
                let player = if self.player1_turn {
                    &self.player1
                } else {
                    &self.player2
                };

                log::trace!(
                    "[{}] disconnected due to inactivity",
                    player.as_ref().unwrap().addr
                );

                send_message(
                    &player.as_ref().unwrap().frame,
                    SocketRequest { opcode: 8, d: None },
                );
            }
        }
    }
}

fn is_line_equal(a: char, b: char, c: char) -> bool {
    a == b && b == c && a != ' '
}

pub fn is_player(player: &Option<SocketSession>, addr: std::net::SocketAddr) -> bool {
    player
        .as_ref()
        .map_or(false, |session| session.addr == addr)
}
