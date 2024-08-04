use crate::{
    json::{EventData, SocketRequest},
    server::{room::Room, send_message},
};

pub fn position(
    addr: std::net::SocketAddr,
    position: usize,
    rooms: &mut Vec<Room>,
) {
    let idx = match rooms.iter_mut().position(|room| room.find_player(addr)) {
        Some(value) => value,
        None => return,
    };

    let room = &mut rooms[idx];

    let is_player1 = crate::server::room::is_player(&room.player1, addr);
    if let Err(e) = room.mark_position(is_player1, position) {
        if is_player1 {
            send_message(&room.player1.as_ref().unwrap().frame, e);
        } else {
            send_message(&room.player2.as_ref().unwrap().frame, e);
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
        SocketRequest {
            opcode: 10,
            d: Some(EventData::MarkPosition { position }),
        },
    );
    room.refresh_turn();
    log::trace!("[{addr}] received mark in {position} position");

    let request = if room.is_win() {
        SocketRequest {
            opcode: 11,
            d: Some(EventData::EndRoom {
                status: if is_player1 { 1 } else { 2 },
            }),
        }
    } else if room.is_full() {
        SocketRequest {
            opcode: 11,
            d: Some(EventData::EndRoom { status: 3 }),
        }
    } else {
        return;
    };

    log::info!("Room ended with status!");
    room.reply_event(request);

    let room: Room = rooms.remove(idx);
    rooms.insert(idx, room.reset());
}

pub fn play_again(
    addr: std::net::SocketAddr,
    rooms: &mut Vec<Room>,
) {
    let idx = match rooms.iter_mut().position(|room| room.find_player(addr)) {
        Some(value) => value,
        None => return,
    };

    let room = &mut rooms[idx];
    room.duration_turn = Some(std::time::Instant::now());
    room.player1_turn = !room.player1_turn;

    super::users::notify_joined(false, room.player1.as_ref().unwrap(), room.player2.as_ref());
    super::users::notify_joined(true, room.player2.as_ref().unwrap(), room.player1.as_ref());
}