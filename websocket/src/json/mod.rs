#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SocketRequest {
    pub opcode: u32,
    pub d: Option<EventData>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum EventData {
    // opcode: 10
    MarkPosition {
        position: usize
    },
    // opcode: 11
    // 1 -> won of player 1; 2 -> won of player 2; 3 -> draw
    EndRoom {
        status: u8,
    },
    // opcode: 12
    JoinRoom {
        player_name: String,
        room_id: u8,
        room_code: Option<String>,
    },
    // opcode: 13
    Joined {
        id: u8,
        name: String,
    },
    // Left -> opcode: 14
    // opcode: 15
    CreateRoom {
        player_name: String,
        public: bool,
    },
    // opcode: 16
    DeleteRoom {
        id: u8,
    },
    // opcode: 17
    ListRooms {
        parties: Option<Vec<EventData>>,
    },
    // opcode: 18
    RoomCreated {
        id: u8,
        player_name: String,
        players_amount: u8,
        public: bool
    },
    // opcode: 19
    RoomDeleted {
        id: u8,
    },
    // opcode: 20
    RoomPlayers {
        id: u8,
        amount: u8,
    },
    // opcode: 21
    OwnerCode {
        code: String,
    },
    // PlayAgain -> opcode: 22
    Message(String),
}

pub enum Command {
    JoinUser {
        addr: std::net::SocketAddr,
        data: EventData,
    },
    RemoveUser {
        addr: std::net::SocketAddr,
    },
    MarkPosition {
        addr: std::net::SocketAddr,
        data: EventData,
    },
    CreateRoom {
        addr: std::net::SocketAddr,
        data: EventData,
    },
    DeleteRoom {
        addr: std::net::SocketAddr,
        id: u8,
    },
    ListRooms {
        addr: std::net::SocketAddr,
    },
    StartGame {
        addr: std::net::SocketAddr,
    },
    PlayAgain {
        addr: std::net::SocketAddr,
    }
}

impl SocketRequest {
    pub fn new(opcode: u32, d: Option<EventData>) -> Self {
        Self { opcode, d }
    }
}
