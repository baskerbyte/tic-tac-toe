use common::settings::{AppSettings, Protocol};
use web_socket::WebSocket;

use crate::{
    json::{Command, EventData, SocketRequest},
    server::session::{Room, SocketSession},
};

pub mod handshake;
pub mod request;
pub mod session;

pub struct App {
    pub settings: AppSettings,
    pub rooms: Vec<Room>,
}

impl App {
    pub fn new() -> Self {
        Self {
            settings: AppSettings::new("application.toml"),
            rooms: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.settings.websocket.socket()).await?;
        log::info!(
            "WebSocket server is listening on ws://{}",
            self.settings.websocket.socket()
        );

        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<Command>();

        loop {
            let cmd_tx = cmd_tx.clone();

            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    let (reader, mut writer) = stream.into_split();
                    let mut reader = tokio::io::BufReader::new(reader);

                    let req = request::HttpRequest::parse(&mut reader).await?;

                    let key = match request::get_sec_key(&req) {
                        Some(key) => key,
                        None => {
                            log::error!("[{addr}] failed to get websocket key");
                            continue;
                        }
                    };

                    let res = handshake::response(key, [("x-agent", "web-socket")]);
                    tokio::io::AsyncWriteExt::write_all(&mut writer, res.as_bytes()).await?;

                    log::trace!("[{addr}] successfully connected");

                    // Channel to send events between sockets
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SocketRequest>();
                    let player_session = session::PlayerSession::new(addr.clone(), tx.clone());

                    if let Some(room) = self.rooms.iter_mut().find(|room| room.is_available()) {
                        room.player2 = Some(player_session)
                    } else {
                        self.rooms.push(Room::new(player_session, None))
                    }

                    tokio::spawn(async move {
                        let mut session: SocketSession = SocketSession::new(addr, tx);

                        let mut ws_writer = WebSocket::server(writer);
                        let mut ws_reader = WebSocket::server(reader);

                        let mut interval = tokio::time::interval(std::time::Duration::from_secs(20));

                        loop {
                            tokio::select! {
                                Ok(event) = ws_reader.recv() => {
                                    match event {
                                        web_socket::Event::Data { data, .. } => {
                                            let event = match serde_json::from_slice::<SocketRequest>(&data) {
                                                Ok(event) => event,
                                                Err(_) => { continue; }
                                            };

                                            match event.d.clone().unwrap() {
                                                EventData::Position { .. } => {
                                                    cmd_tx.send(Command::Reply { addr, event: event });
                                                }
                                                _ => {}
                                            }
                                        },
                                        web_socket::Event::Ping(_) => {
                                            ws_writer.send_pong("p").await;
                                        },
                                        web_socket::Event::Pong(_) => session.refresh_hb(),
                                        web_socket::Event::Error(_) | web_socket::Event::Close { .. } => {
                                            cmd_tx.send(Command::RemoveUser { addr });

                                            break;
                                        },
                                    }
                                },
                                Some(event) = rx.recv() => {
                                    match event.opcode {
                                        8 => {
                                            cmd_tx.send(Command::RemoveUser { addr });

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
                                        cmd_tx.send(Command::RemoveUser { addr });

                                        break;
                                    };
                                }
                            }
                        }
                    });
                }
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        Command::RemoveUser { addr } => {
                            if let Some(idx) = self.rooms.iter().position(|room| {
                                room.player1.addr == addr || room.player2.as_ref().map_or(false, |session| session.addr == addr)
                            }) {
                                self.rooms.remove(idx);
                            }
                        },
                        //            Orign addr
                        Command::Reply { addr, event } => {
                            if let Some(room) = self.rooms.iter_mut().find(|room| {
                                room.player1.addr == addr || room.player2.as_ref().map_or(false, |session| session.addr == addr)
                            }) {
                                match event.d {
                                    Some(EventData::Position { x, y }) => {
                                        let is_player1 = room.player1.addr == addr;
                                        room.mark_position(is_player1, (x, y));

                                        if is_player1 {
                                            let request = SocketRequest { opcode: 10, d: Some(EventData::Position { x, y }) };

                                            room.player2.as_ref().unwrap().frame.send(request.clone());
                                        } else {
                                            let request = SocketRequest { opcode: 10, d: Some(EventData::Position { x, y }) };

                                            room.player1.frame.send(request.clone());
                                        };

                                        let request = if room.is_win() {
                                            SocketRequest { opcode: 11, d: Some(EventData::EndRoom { status: if is_player1 { 1 } else { 2 } }) }
                                        } else if room.is_full() {
                                            SocketRequest { opcode: 11, d: Some(EventData::EndRoom { status: 3 }) }
                                        } else {
                                            continue;
                                        };

                                        room.reply_event(request);
                                        room.reset();
                                    }
                                    Some(_) | None => {}
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}
