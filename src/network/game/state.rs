use std::collections::HashMap;
use libp2p::PeerId;

use crate::engine::Game;
use crate::engine::models::CardKind;

pub struct GameHostState {
    pub game: Game,
    pub peer_to_player_id: HashMap<PeerId, usize>,
    pub player_id_to_peer: HashMap<usize, PeerId>,
    pub turn_submissions: HashMap<usize, (HashMap<CardKind, usize>, HashMap<CardKind, usize>)>,
}

impl GameHostState {
    pub fn new(
        game: Game,
        peer_to_player_id: HashMap<PeerId, usize>,
        player_id_to_peer: HashMap<usize, PeerId>,
    ) -> Self {
        Self {
            game,
            peer_to_player_id,
            player_id_to_peer,
            turn_submissions: HashMap::new(),
        }
    }

    pub fn get_player_id(&self, peer: &PeerId) -> Option<usize> {
        self.peer_to_player_id.get(peer).copied()
    }

    pub fn get_peer_id(&self, player_id: usize) -> Option<&PeerId> {
        self.player_id_to_peer.get(&player_id)
    }

    // remove player on disconnect
    pub fn remove_player(&mut self, peer: &PeerId) -> Option<usize> {
        if let Some(player_id) = self.peer_to_player_id.remove(peer) {
            self.player_id_to_peer.remove(&player_id);
            Some(player_id)
        } else {
            None
        }
    }

    pub fn connected_player_count(&self) -> usize {
        self.peer_to_player_id.len()
    }
}
