use std::collections::HashMap;
use libp2p::PeerId;

use crate::engine::{models::{CardKind, PlayerPublic}, state::{GameStatus, PlayerTurnState}};

pub struct GameClientState {
    pub player_id: usize,
    pub hand: HashMap<CardKind, usize>,
    pub players_public: Vec<PlayerPublic>,
    pub game_status: GameStatus,
    pub selected_cards: HashMap<CardKind, usize>,
    pub turn_submitted: bool,
    pub host_peer_id: Option<PeerId>,
}

impl GameClientState {
    pub fn new(
        player_id: usize,
        hand: HashMap<CardKind, usize>,
        players_public: Vec<PlayerPublic>,
        game_status: GameStatus,
    ) -> Self {
        Self {
            player_id,
            hand,
            players_public,
            game_status,
            selected_cards: HashMap::new(),
            turn_submitted: false,
            host_peer_id: None,
        }
    }

    pub fn select_card(&mut self, card: CardKind) -> Result<(), String> {
        let hand_count = self.hand.get(&card).copied().unwrap_or(0);
        let selected_count = self.selected_cards.get(&card).copied().unwrap_or(0);

        if selected_count >= hand_count {
            return Err("Cannot select more cards than in hand".to_string());
        }

        *self.selected_cards.entry(card).or_insert(0) += 1;
        Ok(())
    }

    pub fn deselect_card(&mut self, card: CardKind) -> Result<(), String> {
        let selected_count = self.selected_cards.get(&card).copied().unwrap_or(0);

        if selected_count == 0 {
            return Err("No cards of this type selected".to_string());
        }

        if selected_count == 1 {
            self.selected_cards.remove(&card);
        } else {
            *self.selected_cards.get_mut(&card).unwrap() -= 1;
        }

        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selected_cards.clear();
    }

    // get remaining hand after removing selected cards
    pub fn get_remaining_hand(&self) -> HashMap<CardKind, usize> {
        let mut remaining = self.hand.clone();

        for (card, selected_count) in &self.selected_cards {
            if let Some(hand_count) = remaining.get_mut(card) {
                *hand_count = hand_count.saturating_sub(*selected_count);
                if *hand_count == 0 {
                    remaining.remove(card);
                }
            }
        }

        remaining
    }

    pub fn mark_turn_submitted(&mut self) {
        self.turn_submitted = true;
    }

    pub fn reset_for_new_turn(&mut self) {
        self.selected_cards.clear();
        self.turn_submitted = false;
    }

    pub fn update_hand(&mut self, new_hand: HashMap<CardKind, usize>) {
        self.hand = new_hand;
        self.reset_for_new_turn();
    }

    pub fn update_game_status(&mut self, status: GameStatus) {
        self.game_status = status;
    }

    pub fn update_players_public(&mut self, players: Vec<PlayerPublic>) {
        self.players_public = players;
    }

    pub fn can_submit(&self) -> bool {
        !self.selected_cards.is_empty() && !self.turn_submitted
    }

    pub fn get_own_turn_state(&self) -> PlayerTurnState {
        self.game_status.player_turn_states
            .get(self.player_id)
            .copied()
            .unwrap_or(PlayerTurnState::NotSelected)
    }
}

