use crate::json::{Command, EventData};

mod game;
pub mod rooms;
pub mod users;

pub fn handle(
    command: Command,
    rooms: &mut Vec<crate::server::room::Room>,
    queue: &mut Vec<crate::server::session::SocketSession>,
) {
    match command {
        Command::JoinUser {
            addr,
            data:
                EventData::JoinRoom {
                    player_name,
                    room_id,
                    room_code,
                },
        } => users::join(addr, player_name, room_id, room_code, rooms, queue),
        Command::RemoveUser { addr } => users::remove(addr, rooms, queue),
        Command::MarkPosition {
            addr,
            data: EventData::MarkPosition { position },
        } => game::position(addr, position, rooms),
        Command::CreateRoom {
            addr,
            data:
                EventData::CreateRoom {
                    player_name,
                    public,
                },
        } => rooms::create(addr, player_name, public, rooms, queue),
        Command::DeleteRoom { addr, id } => rooms::delete(addr, id, rooms, queue),
        Command::ListRooms { addr } => rooms::list(addr, rooms, queue),
        Command::PlayAgain { addr } => game::play_again(addr, rooms),
        Command::LeaveRoom { addr } => rooms::leave(addr, rooms, queue),
        _ => {}
    }
}

pub fn notify_connections(
    event: crate::json::SocketRequest,
    queue: &mut Vec<crate::server::session::SocketSession>,
) {
    for connection in queue.iter() {
        crate::server::send_message(&connection.frame, event.clone())
    }
}
