use super::player::PlayerId;

/// Messages from client to server
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Login {
        name: String,
        password: String,
    },
    Logout {
        player_id: PlayerId,
    },
    Move {
        player_id: PlayerId,
        dx: i32,
        dy: i32,
    },
    Attack {
        player_id: PlayerId,
    },
    Say {
        player_id: PlayerId,
        message: String,
    },
}

/// Messages from server to client
#[derive(Debug, Clone)]
pub enum ServerMessage {
    LoginOk {
        player_id: PlayerId,
    },
    LoginFailed {
        reason: String,
    },
    PlayerMoved {
        player_id: PlayerId,
        x: i32,
        y: i32,
    },
    PlayerStats {
        player_id: PlayerId,
        hp: i32,
        max_hp: i32,
        level: i32,
        xp: i32,
    },
    CreatureMoved {
        creature_id: u32,
        x: i32,
        y: i32,
    },
    CreatureHealth {
        creature_id: u32,
        health_percent: u8,
    },
    TextMessage {
        message: String,
    },
    PlayerDied {
        player_id: PlayerId,
    },
}