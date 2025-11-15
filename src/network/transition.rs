use std::collections::HashMap;

use crate::engine::Game;
use super::{Host, Client};
use super::lobby::{LobbyHostState, LobbyClientState};
use super::game::{GameHostState, GameClientState};

// transition lobby host to game host
pub fn lobby_to_game_host(
    lobby_host: Host<LobbyHostState>,
    game: Game,
) -> Host<GameHostState> {
    // extract peer mappings from lobby
    let (peer_to_player_id, player_id_to_peer) = lobby_host.state.get_peer_mappings();

    Host::<GameHostState>::new(
        lobby_host.swarm,
        lobby_host.topic,
        game,
        peer_to_player_id,
        player_id_to_peer,
    )
}

// transition lobby client to game client
pub fn lobby_to_game_client(
    lobby_client: Client<LobbyClientState>,
    player_id: usize,
    initial_hand: HashMap<crate::engine::models::CardKind, usize>,
    players_public: Vec<crate::engine::models::PlayerPublic>,
    game_status: crate::engine::state::GameStatus,
) -> Client<GameClientState> {
    Client::<GameClientState>::new(
        lobby_client.swarm,
        lobby_client.topic,
        player_id,
        initial_hand,
        players_public,
        game_status,
    )
}
