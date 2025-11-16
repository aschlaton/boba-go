use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::engine::{
    models::{CardKind, PlayerPublic},
    state::GameStatus,
};

// messages from client to host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameClientMessage {
    // submit selected cards and remaining hand for this turn
    SubmitTurn {
        selected_cards: HashMap<CardKind, usize>,
        remaining_hand: HashMap<CardKind, usize>,
    },
}

// messages from host to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameHostMessage {
    // game state update (sent after each turn processes)
    GameUpdate {
        all_hands: Vec<HashMap<CardKind, usize>>, // indexed by player_id
        players_public: Vec<PlayerPublic>,
        game_status: GameStatus,
    },

    GameEnded {
        final_scores: Vec<(usize, f32, String, crate::engine::ScoreBreakdown)>, // (player_id, score, name, breakdown)
        reason: GameEndReason,
    },

    // error response
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEndReason {
    Completed,
    PlayerDisconnected { player_id: usize },
}

