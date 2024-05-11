use crate::json::SocketRequest;
use crate::server::session::SocketSession;

pub struct Room {
    pub tray: [[char; 3]; 3],
    pub player1: Option<SocketSession>,
    pub player2: Option<SocketSession>,
}

impl Room {
    pub fn new(player1: Option<SocketSession>, player2: Option<SocketSession>) -> Self {
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

fn is_line_equal(a: char, b: char, c: char) -> bool {
    a == b && b == c && a != ' '
}