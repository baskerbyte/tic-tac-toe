use crate::json::{EventData, SocketRequest};
use crate::server::room::{is_player, Room};
use crate::server::send_message;
use crate::server::session::SocketSession;

pub fn join(
    name: String,
    addr: std::net::SocketAddr,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    if let Some(room) = rooms.iter().find(|room| room.find_player(addr)) {
        if room.player1.as_ref().unwrap().addr == addr {
            send_message(
                &room.player1.as_ref().unwrap().frame,
                SocketRequest::new(1011, Some(EventData::Message("player already in match".to_string()))),
            );
        }
        
        return;
    }

    log::trace!("[{addr}] {} joined in match", name.clone());

    if let Some(idx) = queue.iter().position(|session| session.addr == addr) {
        let mut player = queue.remove(idx);
        player.name = Some(name.clone());

        if let Some(room) = rooms.iter_mut().find(|room| room.is_available()) {
            match (room.player1.is_none(), room.player2.is_none()) {
                (true, false) => room.player1 = Some(player),
                (false, true) => room.player2 = Some(player),
                _ => return
            };

            if let (Some(other), Some(current)) = (room.player1.as_ref(), room.player2.as_ref()) {
                send_message(
                    &other.frame,
                    SocketRequest::new(13, Some(EventData::Joined { name: current.name.as_ref().unwrap().to_string() })),
                );

                send_message(
                    &current.frame,
                    SocketRequest::new(13, Some(EventData::Joined { name: other.name.as_ref().unwrap().to_string() })),
                );
            }
        } else {
            rooms.push(Room::new(Some(player), None))
        }
    }
}

pub fn remove_user(
    addr: std::net::SocketAddr,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    if let Some(idx) = rooms.iter().position(|room| room.find_player(addr)) {
        let room = rooms.remove(idx);
        let player = if is_player(&room.player1, addr) {
            room.player2
        } else {
            room.player1
        };

        if let Some(player) = player {
            // Left event
            send_message(
                &player.frame,
                SocketRequest { opcode: 14, d: None },
            );
            queue.push(player);
        }
    }
}

pub fn reply_position(
    addr: std::net::SocketAddr,
    (x, y): (usize, usize),
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    if let Some(idx) = rooms.iter_mut().position(|room| room.find_player(addr)) {
        let room = &mut rooms[idx];

        let is_player1 = is_player(&room.player1, addr);
        if let Err(e) = room.mark_position(is_player1, (x, y)) {
            if is_player1 {
                send_message(
                    &room.player1.as_ref().unwrap().frame,
                    e,
                );
            } else {
                send_message(
                    &room.player2.as_ref().unwrap().frame,
                    e,
                );
            }
            
            return;
        }

        let player = if is_player1 {
            room.player2.as_ref().unwrap()
        } else {
            room.player1.as_ref().unwrap()
        };

        send_message(
            &player.frame,
            SocketRequest { opcode: 10, d: Some(EventData::Position { x, y }) },
        );
        room.refresh_turn();

        let request = if room.is_win() {
            SocketRequest { opcode: 11, d: Some(EventData::EndRoom { status: if is_player1 { 1 } else { 2 } }) }
        } else if room.is_full() {
            SocketRequest { opcode: 11, d: Some(EventData::EndRoom { status: 3 }) }
        } else {
            return;
        };

        room.reply_event(request);
        let room = rooms.remove(idx);

        queue.push(room.player1.unwrap());
        queue.push(room.player2.unwrap());
    }
}