use crate::json::{Command, EventData};
use crate::server::send_message;
use crate::server::session::SocketSession;

pub async fn handle(
    session: &mut SocketSession,
    data: Box<[u8]>,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>,
) -> Result<(), ()> {
    let event = match serde_json::from_slice::<crate::json::SocketRequest>(&data) {
        Ok(event) => event,
        Err(_) => return Ok(()),
    };

    match (event.opcode, event.d.clone()) {
        (10, Some(EventData::MarkPosition { .. })) => {
            send_message(
                &cmd_tx,
                Command::MarkPosition {
                    addr: session.addr,
                    data: event.d.unwrap(),
                },
            );
        }
        (12, Some(EventData::JoinRoom { .. })) => send_message(
            &cmd_tx,
            Command::JoinUser {
                addr: session.addr,
                data: event.d.unwrap(),
            },
        ),
        (14, None) => send_message(&cmd_tx, Command::LeaveRoom { addr: session.addr }),
        (15, Some(EventData::CreateRoom { .. })) => send_message(
            &cmd_tx,
            Command::CreateRoom {
                addr: session.addr,
                data: event.d.unwrap(),
            },
        ),
        (17, None) => send_message(&cmd_tx, Command::ListRooms { addr: session.addr }),
        (22, None) => send_message(&cmd_tx, Command::PlayAgain { addr: session.addr }),
        _ => {}
    }

    Ok(())
}
