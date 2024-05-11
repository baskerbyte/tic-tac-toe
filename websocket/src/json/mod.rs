#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SocketRequest {
    pub opcode: u32,
    pub d: Option<EventData>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(untagged)]
pub enum EventData {
    // opcode: 10
    Position { x: usize, y: usize },
    // opcode: 11
    // 1 -> won of player 1; 2 -> won of player 2; 3 -> draw
    EndRoom { status: u8 },
    // opcode: 12
    Identify { name: String },
    // opcode: 13
    Joined { name: String },
    // Left Event = opcode: 14
}

pub enum Command {
    JoinUser {
        addr: std::net::SocketAddr,
        name: String,
    },
    RemoveUser {
        addr: std::net::SocketAddr,
    },
    Reply {
        addr: std::net::SocketAddr,
        event: SocketRequest,
    },
}
