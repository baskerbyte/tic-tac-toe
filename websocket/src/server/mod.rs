use web_socket::WebSocket;

use common::settings::{AppSettings, Protocol};

use crate::{
    json::{Command, EventData, SocketRequest},
    server::room::Room,
    server::session::SocketSession,
};

pub mod handshake;
pub mod request;
pub mod session;
mod room;

pub struct App {
    pub settings: AppSettings,
    pub rooms: Vec<Room>,
    pub queue: Vec<SocketSession>,
}

impl App {
    pub fn new() -> Self {
        Self {
            settings: AppSettings::new("application.toml"),
            rooms: Vec::new(),
            queue: Vec::new(),
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
                    let mut session: SocketSession = SocketSession::new(addr, tx);
                    self.queue.push(session.clone());
                    
                    tokio::spawn(async move {
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
                                                },
                                                EventData::Identify { name } => {
                                                    cmd_tx.send(Command::JoinUser { addr, name });
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
                    println!("{:?}", self.rooms);
                    match cmd {
                        Command::JoinUser { addr, name} => {
                            if self.rooms.iter().any(|room| room.find_player(addr)) {
                                continue;
                            }

                            if let Some(idx) = self.queue.iter().position(|session| session.addr == addr) {
                                let mut player = self.queue.remove(idx);
                                player.name = Some(name.clone());
                                
                                if let Some(room) = self.rooms.iter_mut().find(|room| room.is_available()) {
                                    if room.player1.is_none() {
                                        room.player1 = Some(player);

                                        room.player2.as_ref().unwrap().frame.send(
                                            SocketRequest { opcode: 13, d: Some(EventData::Joined { name }) }
                                        );
                                    } else {
                                        room.player2 = Some(player);

                                        room.player1.as_ref().unwrap().frame.send(
                                            SocketRequest { opcode: 13, d: Some(EventData::Joined { name }) }
                                        );
                                    }
                                } else {
                                    self.rooms.push(Room::new(Some(player), None))
                                }
                            }
                        }
                        Command::RemoveUser { addr } => {
                            if let Some(idx) = self.rooms.iter().position(|room| room.find_player(addr)) {
                                let room = self.rooms.remove(idx);
                                let player = if Room::is_player(&room.player1, addr) {
                                    room.player2
                                } else {
                                    room.player1
                                };

                                if let Some(player) = player {
                                    let name = player.name.as_ref().unwrap().to_string();

                                    self.queue.push(player);
                                    cmd_tx.send(Command::JoinUser { addr, name });
                                }
                            }
                        },
                        //            Orign addr
                        Command::Reply { addr, event } => {
                            if let Some(room) = self.rooms.iter_mut().find(|room| room.find_player(addr)) {
                                match event.d {
                                    Some(EventData::Position { x, y }) => {
                                        let is_player1 = Room::is_player(&room.player1, addr);
                                        if let Err(_) = room.mark_position(is_player1, (x, y)) {
                                            continue;
                                        }

                                        if is_player1 {
                                            let request = SocketRequest { opcode: 10, d: Some(EventData::Position { x, y }) };

                                            room.player2.as_ref().unwrap().frame.send(request);
                                        } else {
                                            let request = SocketRequest { opcode: 10, d: Some(EventData::Position { x, y }) };

                                            room.player1.as_ref().unwrap().frame.send(request);
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
