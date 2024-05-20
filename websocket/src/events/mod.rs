use crate::json::{Command, EventData};
use crate::server::room::Room;
use crate::server::send_message;
use crate::server::session::SocketSession;

mod command;

pub fn handle_command(
    command: Command,
    rooms: &mut Vec<Room>,
    queue: &mut Vec<SocketSession>,
) {
    match command {
        Command::JoinUser { addr, name } =>
            command::join(name, addr, rooms, queue),
        Command::RemoveUser { addr } =>
            command::remove_user(addr, rooms, queue),
        Command::Reply { addr, event } => {
            match event.d {
                Some(EventData::Position { x, y }) => {
                    command::reply_position(
                        addr,
                        (x, y),
                        rooms,
                        queue,
                    );
                }
                _ => {}
            }
        }
    }
}

pub async fn handle_client(
    session: &mut SocketSession,
    event: web_socket::Event,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>,
    ws_writer: &mut web_socket::WebSocket<tokio::net::tcp::OwnedWriteHalf>
) -> Result<(), ()> {
    match event {
        web_socket::Event::Data { data, .. } => {
            let event = match serde_json::from_slice::<crate::json::SocketRequest>(&data) {
                Ok(event) => event,
                Err(_) => return Ok(())
            };

            match (event.opcode, event.d.clone().unwrap()) {
                (10, EventData::Position { .. }) => {
                    send_message(
                        &cmd_tx,
                        Command::Reply { addr: session.addr, event },
                    );
                }
                (12, EventData::Identify { name }) => {
                    send_message(
                        &cmd_tx,
                        Command::JoinUser { addr: session.addr, name },
                    )
                }
                _ => {}
            }
        }
        web_socket::Event::Ping(_) => {
            ws_writer.send_pong("p").await;
        }
        web_socket::Event::Pong(_) => session.refresh_hb(),
        web_socket::Event::Error(_) | web_socket::Event::Close { .. } => {
            send_message(
                &cmd_tx,
                Command::RemoveUser { addr: session.addr },
            );

            return Err(())
        }
    }
    
    Ok(())
}