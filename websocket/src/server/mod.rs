use web_socket::WebSocket;

use common::settings::{AppSettings, Protocol};

use crate::{
    json::{Command, SocketRequest},
    server::room::Room,
    server::session::SocketSession,
};

pub mod handshake;
pub mod request;
pub mod room;
pub mod session;

pub struct App {
    pub settings: AppSettings,
    pub rooms: Vec<Room>,
    pub queue: Vec<SocketSession>,
    pub cmd_tx: tokio::sync::mpsc::UnboundedSender<Command>,
    pub cmd_rx: tokio::sync::mpsc::UnboundedReceiver<Command>,
}

impl App {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

        Self {
            settings: AppSettings::new("application.toml"),
            rooms: Vec::new(),
            queue: Vec::new(),
            cmd_tx,
            cmd_rx,
        }
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.settings.websocket.socket()).await?;
        log::info!(
            "WebSocket server is listening on ws://{}",
            self.settings.websocket.socket()
        );

        loop {
            // Channel to modify Vecs without parallelism
            let cmd_tx = self.cmd_tx.clone();
            let mut room_turn = tokio::time::interval(std::time::Duration::from_secs(15));

            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    let (reader, mut writer) = stream.into_split();
                    let mut reader = tokio::io::BufReader::new(reader);

                    if let Err(e) = handshake::send(&mut reader, &mut writer).await {
                        log::error!("[{addr}] failed to handshake {e}");
                        continue;
                    }

                    log::trace!("[{addr}] successfully connected");

                    // Channel to send events between sockets
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SocketRequest>();
                    let mut session: SocketSession = SocketSession::new(addr, tx);
                    self.queue.push(session.clone());

                    tokio::spawn(async move {
                        let mut ws_writer = WebSocket::server(writer);
                        let mut ws_reader = WebSocket::server(reader);

                        let mut interval = tokio::time::interval(std::time::Duration::from_secs(20));

                        loop {
                            tokio::select! {
                                Ok(event) = ws_reader.recv() => {
                                    if let Err(_) = session::handle_client(
                                        &mut session,
                                        event,
                                        &cmd_tx,
                                        &mut ws_writer
                                    ).await {
                                        break;
                                    }
                                },
                                Some(event) = rx.recv() => {
                                    match event.opcode {
                                        8 => {
                                            send_message(
                                                &cmd_tx,
                                                Command::RemoveUser { addr }
                                            );

                                            break;
                                        }
                                        9 => { ws_writer.send_ping("p").await; }
                                        _ => {
                                            ws_writer.send(serde_json::to_string(&event).unwrap().as_str()).await;
                                        }
                                    }
                                },
                                _ = interval.tick() => {
                                    if let Err(_) = session.heartbeat() {
                                        send_message(
                                            &cmd_tx,
                                            Command::RemoveUser { addr }
                                        );

                                        break;
                                    };
                                }
                            }
                        }
                    });
                }
                Some(cmd) = self.cmd_rx.recv() =>
                    crate::commands::handle(cmd, &mut self.rooms, &mut self.queue),
                _ = room_turn.tick() => {
                    for (id, room) in self.rooms.iter().enumerate() {
                        if room.player1.is_some() && room.player2.is_some() {
                            room.timer()
                        }

                        if room.player1.is_none() && room.player2.is_none() {
                            crate::commands::notify_connections(
                                SocketRequest::new(19, Some(crate::json::EventData::RoomDeleted { id })),
                                &mut self.queue,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn send_message<T : std::fmt::Debug>(frame: &tokio::sync::mpsc::UnboundedSender<T>, data: T) {
    if let Err(_) = frame.send(data) {
        log::error!("failed to send frame message");
    }
}