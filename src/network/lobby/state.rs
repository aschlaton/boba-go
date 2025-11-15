use std::collections::HashMap;
use libp2p::PeerId;

use super::protocol::LobbyPlayer;


pub struct LobbyHostState {
    pub room_name: String,
    pub host_player_name: String,
    players: HashMap<PeerId, LobbyPlayer>,
    next_player_id: usize,
}

impl LobbyHostState {
    /// Create a new lobby
    pub fn new(room_name: String, host_player_name: String) -> Self {
        Self {
            room_name,
            host_player_name,
            players: HashMap::new(),
            next_player_id: 0,
        }
    }

    /// Check if a player name is already taken
    pub fn is_name_taken(&self, name: &str) -> bool {
        self.host_player_name == name || self.players.values().any(|p| p.name == name)
    }

    /// Add a new player to the lobby and return their assigned ID
    pub fn add_player(&mut self, peer: PeerId, player_name: String) -> usize {
        let player_id = self.next_player_id + 1;
        self.next_player_id += 1;

        let lobby_player = LobbyPlayer {
            id: player_id,
            name: player_name,
        };

        self.players.insert(peer, lobby_player);
        player_id
    }

    /// Remove a player from the lobby
    pub fn remove_player(&mut self, peer: &PeerId) -> Option<LobbyPlayer> {
        self.players.remove(peer)
    }

    /// Get all players in the lobby (including host as player 0)
    pub fn get_all_players(&self) -> Vec<LobbyPlayer> {
        let mut players: Vec<LobbyPlayer> = self.players.values().cloned().collect();
        // Add host as player 0
        players.insert(
            0,
            LobbyPlayer {
                id: 0,
                name: self.host_player_name.clone(),
            },
        );
        players
    }

    /// Get the number of players (excluding host)
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}
