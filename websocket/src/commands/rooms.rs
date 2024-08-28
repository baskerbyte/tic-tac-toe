use crate::{
    json::{EventData, SocketRequest},
    server::{room::Room, send_message, session::SocketSession},
};

pub fn create(
    addr: std::net::SocketAddr,
    player_name: String,
    public: bool,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    log::info!("Creating room for {player_name}");
    
    let mut session = super::users::get_session(addr, queue).unwrap();

    if session.name.is_none() || session.name.as_ref() != Some(&player_name) {
        session.name = Some(player_name.clone());
    };

    let room_id = rooms.len();
    let code = generate_room_code(public);

    let room = Room::new(None, None, code.clone(), player_name.clone());
    rooms.push(room);
    let room = &mut rooms[room_id];

    super::users::join_room(addr, session, room);

    if let Some(code) = code {
        send_message(
            &room.player1.as_ref().unwrap().frame,
            SocketRequest::new(21, Some(EventData::OwnerCode { code })),
        )
    }

    super::notify_connections(
        SocketRequest::new(
            18,
            Some(EventData::RoomCreated {
                id: room_id as u8,
                player_name,
                players_amount: 1,
                public,
            }),
        ),
        queue,
    )
}

pub fn leave(
    addr: std::net::SocketAddr,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    if let Some(idx) = rooms.iter_mut().position(|room| room.find_player(addr)) {
        let room = &mut rooms[idx];

        let player = if crate::server::room::is_player(&room.player1, addr) {
            queue.push(room.player1.as_ref().unwrap().clone());
            room.player1 = None;
            
            &room.player2
        } else {
            queue.push(room.player2.as_ref().unwrap().clone());
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
        };

        super::notify_connections(
            SocketRequest::new(20, Some(EventData::Left { id: idx as u8 })),
            queue,
        )
    }
}

pub fn delete(
    addr: std::net::SocketAddr,
    id: usize,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    let idx = match rooms
        .iter()
        .position(|room| room.player1.as_ref().unwrap().addr == addr)
    {
        Some(value) => value,
        None => return,
    };

    if idx != id as usize {
        return;
    }

    let room = rooms.remove(idx);

    if let Some(player1) = room.player1 {
        queue.push(player1);
    }

    if let Some(player2) = room.player2 {
        queue.push(player2);
    }

    super::notify_connections(
        SocketRequest::new(19, Some(EventData::RoomDeleted { id })),
        queue,
    )
}

pub fn list(addr: std::net::SocketAddr, rooms: &mut Vec<Room>, queue: &mut Vec<SocketSession>) {
    let session = match queue.iter().find(|session| session.addr == addr) {
        Some(value) => value,
        None => return,
    };

    let mut parties = Vec::new();

    for (idx, room) in rooms.iter().enumerate() {
        let players_amount = room.player1.is_some() as u8 + room.player2.is_some() as u8;

        let party = EventData::RoomCreated {
            id: idx as u8,
            player_name: room.name.clone(),
            players_amount,
            public: room.code.is_none(),
        };

        parties.push(party);
    }

    send_message(
        &session.frame,
        SocketRequest::new(17, Some(EventData::ListRooms { parties: Some(parties)})),
    )
}

fn generate_room_code(public: bool) -> Option<String> {
    if public {
        None
    } else {
        Some(
            rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
                .take(5)
                .map(char::from)
                .collect::<String>()
                .to_uppercase(),
        )
    }
}
