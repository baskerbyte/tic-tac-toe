use crate::{
    json::{EventData, SocketRequest},
    server::{room::Room, send_message, session::SocketSession},
};

pub fn join(
    addr: std::net::SocketAddr,
    player_name: String,
    room_id: u8,
    room_code: Option<String>,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    let mut session = get_session(addr, queue).unwrap();

    if session.name.is_none() || session.name.as_ref() != Some(&player_name) {
        session.name = Some(player_name);
    };

    let room = &mut rooms[room_id as usize];

    if room_code != room.code {
        return;
    }

    join_room(addr, session, room);
}

pub fn remove(addr: std::net::SocketAddr, rooms: &mut Vec<Room>, queue: &mut Vec<SocketSession>) {
    if let Some(idx) = rooms.iter().position(|room| room.find_player(addr)) {
        let room = &mut rooms[idx];
        let player = if crate::server::room::is_player(&room.player1, addr) {
            queue.push(room.player1.clone().unwrap());
            room.player1 = None;

            &room.player2
        } else {
            queue.push(room.player2.clone().unwrap());
            room.player2 = None;

            &room.player1
        };

        if let Some(player) = player {
            // Left event
            send_message(
                &player.frame,
                SocketRequest {
                    opcode: 14,
                    d: None,
                },
            );
        }
    } else if let Some(idx) = queue.iter().position(|session| session.addr == addr) {
        queue.remove(idx);
    }
}

pub fn get_session(
    addr: std::net::SocketAddr,
    queue: &mut Vec<SocketSession>,
) -> Option<SocketSession> {
    if let Some(idx) = queue.iter().position(|session| session.addr == addr) {
        Some(queue.remove(idx))
    } else {
        None
    }
}

pub fn join_room(addr: std::net::SocketAddr, session: SocketSession, room: &mut Room) {
    let name = session.name.clone().unwrap();
    log::trace!("[{addr}] {} joined in match", &name);

    match (room.player1.is_none(), room.player2.is_none()) {
        (true, false) | (true, true) => {
            room.player1 = Some(session);

            notify_joined(true, room.player1.as_ref().unwrap(), room.player2.as_ref());
        }
        (false, true) => {
            room.player2 = Some(session);

            notify_joined(false, room.player2.as_ref().unwrap(), room.player1.as_ref());
        }
        _ => return,
    };
}

pub fn notify_joined(
    is_player1: bool,
    joined_player: &SocketSession,
    other_player: Option<&SocketSession>,
) {
    let other_player = match other_player {
        Some(value) => value,
        None => return,
    };

    crate::server::send_message(
        &other_player.frame,
        SocketRequest::new(
            13,
            Some(EventData::Joined {
                id: is_player1 as u8,
                name: joined_player
                    .name
                    .clone()
                    .unwrap_or_else(|| "Anonymous".into()),
            }),
        ),
    );

    crate::server::send_message(
        &joined_player.frame,
        SocketRequest::new(
            13,
            Some(EventData::Joined {
                id: !is_player1 as u8,
                name: other_player
                    .name
                    .clone()
                    .unwrap_or_else(|| "Anonymous".into()),
            }),
        ),
    );
}
