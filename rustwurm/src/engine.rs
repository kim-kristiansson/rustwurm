pub type PlayerId = u32;

#[derive(Debug)]
pub enum EngineClientMsg {
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
    }
}

#[derive(Debug)]
pub enum EngineServerMsg {
    LoginOk {
        player_id: PlayerId,
    },
    LoginFailed {
        reason: String,
    },
    PlayerMoved {
        player_id: PlayerId,
        dx: i32,
        dy: i32,
    },
    PlayerStats {
        player_id: PlayerId,
        hp: i32,
        lvl: i32,
        xp: i32,
    }
}