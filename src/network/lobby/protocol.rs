use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    JoinRequest { player_name: String },
}

/// messages sent from host to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostMessage {

    // accept or reject join request
    JoinResponse {
        accepted: bool,
        player_id: Option<usize>,
        rejection_reason: Option<String>,
        lobby_players: Vec<LobbyPlayer>,
    },

    // broadcast when players join/leave
    LobbyUpdate {
        players: Vec<LobbyPlayer>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: usize,
    pub name: String,
}

