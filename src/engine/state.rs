use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use std::collections::HashMap;
use crate::engine::constants;
use crate::engine::deck::Deck;
use crate::engine::models::{CardKind, GameConfig, Player, PlayerPublic};
use crate::engine::scoring::ScoreBreakdown;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Not enough cards in deck for new round")]
    NotEnoughCards,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PassDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerTurnState {
    NotSelected,
    Selected,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameStatus {
    pub round: usize,
    pub turn: usize,
    pub round_count: usize,
    pub pass_direction: PassDirection,
    pub is_game_over: bool,
    pub player_turn_states: Vec<PlayerTurnState>,
}


pub struct Game {
    pub seed: u64,
    rng: ChaCha8Rng,
    deck: Deck,
    pub players: Vec<Player>,
    pub round: usize,
    pub turn: usize,
    pub player_turn_states: Vec<PlayerTurnState>,
    pub round_count: usize,
}

impl Game {
    fn get_pass_direction(round: usize) -> PassDirection {
        if round % 2 == 1 {
            PassDirection::Left
        } else {
            PassDirection::Right
        }
    }
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        let num_players = config.player_names.len();
        if num_players < 2 {
            return Err(GameError::InvalidConfig);
        }

        let cards_per_player = constants::cards_per_player(num_players)
            .ok_or(GameError::InvalidConfig)?;

        let seed = config.seed.unwrap_or_else(rand::random);
        let rng = ChaCha8Rng::seed_from_u64(seed);
        
        // Extract card distribution before moving config
        let card_distribution = config.card_distribution.clone();
        
        let players: Vec<Player> = config.player_names
            .into_iter()
            .enumerate()
            .map(|(id, username)| Player {
                id,
                username,
                hand: HashMap::new(),
                public_cards: HashMap::new(),
                boosted_fruit_teas: HashMap::new(),
            })
            .collect();

        let player_turn_states = vec![PlayerTurnState::NotSelected; num_players];
        
        let mut game = Game {
            seed,
            rng,
            deck: Deck::new(),
            players: Vec::new(),
            round: 1,
            turn: 1,
            player_turn_states,
            round_count: config.round_count,
        };

        game.build_deck(card_distribution);
        game.players = players;
        game.distribute_cards(cards_per_player)?;
        Ok(game)
    }


    // deal cards to players
    fn distribute_cards(&mut self, cards_per_player: usize) -> Result<(), GameError> {
        let num_players = self.players.len();
        let total_cards_needed = num_players * cards_per_player;
        if self.deck.size() < total_cards_needed {
            return Err(GameError::NotEnoughCards);
        }

        for player in self.players.iter_mut() {
            for _ in 0..cards_per_player {
                if let Some(card) = self.deck.draw(&mut self.rng) {
                    *player.hand.entry(card).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    pub fn get_current_pass_direction(&self) -> PassDirection {
        Self::get_pass_direction(self.round)
    }

    //passes cards to next player
    pub fn pass_hands(&mut self) {
        let direction = self.get_current_pass_direction();
        let num_players = self.players.len();
        
        match direction {
            PassDirection::Left => {
                let last_hand = self.players.last().unwrap().hand.clone();
                for i in (1..num_players).rev() {
                    self.players[i].hand = self.players[i - 1].hand.clone();
                }
                self.players[0].hand = last_hand;
            }
            PassDirection::Right => {
                let first_hand = self.players[0].hand.clone();
                for i in 0..(num_players - 1) {
                    self.players[i].hand = self.players[i + 1].hand.clone();
                }
                self.players[num_players - 1].hand = first_hand;
            }
        }
    }
    
    // Validate hand submission - client sends selected cards and remaining hand
    // The client handles all Drink Tray logic (activation, allowing 2 selections)
    // Backend just validates that hand math is correct: current_hand = selected + remaining
    pub fn validate_hand_submission(&self, player_id: usize, selected_cards: &HashMap<CardKind, usize>, remaining_hand: &HashMap<CardKind, usize>) -> Result<(), GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }

        let current_hand = &self.players[player_id].hand;

        // Validate that selected + remaining = current
        let mut reconstructed = HashMap::new();
        for (kind, count) in selected_cards {
            *reconstructed.entry(*kind).or_insert(0) += count;
        }
        for (kind, count) in remaining_hand {
            *reconstructed.entry(*kind).or_insert(0) += count;
        }

        if &reconstructed != current_hand {
            return Err(GameError::InvalidConfig);
        }

        Ok(())
    }
    
    // marks player as selected for current turn
    // use when player confirms card to submit
    pub fn mark_player_selected(&mut self, player_id: usize) -> Result<(), GameError> {
        if player_id >= self.player_turn_states.len() {
            return Err(GameError::InvalidConfig);
        }
        self.player_turn_states[player_id] = PlayerTurnState::Selected;
        Ok(())
    }

    pub fn all_players_selected(&self) -> bool {
        self.player_turn_states.iter().all(|s| *s == PlayerTurnState::Selected)
    }

    pub fn reset_turn_states(&mut self) {
        for state in &mut self.player_turn_states {
            *state = PlayerTurnState::NotSelected;
        }
    }

    pub fn process_turn(
        &mut self,
        submissions: Vec<Option<(HashMap<CardKind, usize>, HashMap<CardKind, usize>)>>
    ) -> Result<(), GameError> {
        if !self.all_players_selected() {
            return Err(GameError::InvalidConfig);
        }

        if submissions.len() != self.players.len() {
            return Err(GameError::InvalidConfig);
        }

        // move selected cards to public_cards and track which cards have on_draft
        let mut cards_with_on_draft: Vec<Option<CardKind>> = vec![None; self.players.len()];
        
        for (player_id, submission_opt) in submissions.iter().enumerate() {
            if let Some((selected_cards, remaining_hand)) = submission_opt {
                let player = &mut self.players[player_id];

                // Add selected cards to public_cards and handle Popping Bubbles pairing
                for (kind, count) in selected_cards {
                    // If this is a fruit tea, check if there are available Popping Bubbles
                    if kind.is_fruit_tea() {
                        // Check current count of Popping Bubbles in public_cards (decreases as we pair them)
                        let available_popping_bubbles = player.public_cards.get(&CardKind::PoppingBubbles).copied().unwrap_or(0);
                        
                        let to_boost = (*count).min(available_popping_bubbles);
                        let remaining = *count - to_boost;
                        
                        // Add boosted fruit teas to boosted_fruit_teas (not public_cards)
                        // Also remove the paired Popping Bubbles from public_cards
                        if to_boost > 0 {
                            *player.boosted_fruit_teas.entry(*kind).or_insert(0) += to_boost;
                            
                            // Remove paired Popping Bubbles from public_cards
                            if let Some(popping_count) = player.public_cards.get_mut(&CardKind::PoppingBubbles) {
                                *popping_count -= to_boost;
                                if *popping_count == 0 {
                                    player.public_cards.remove(&CardKind::PoppingBubbles);
                                }
                            }
                        }
                        
                        // Add remaining (unboosted) fruit teas to public_cards
                        if remaining > 0 {
                            *player.public_cards.entry(*kind).or_insert(0) += remaining;
                        }
                    } else {
                        // Non-fruit tea cards go to public_cards normally
                        *player.public_cards.entry(*kind).or_insert(0) += count;
                    }
                    
                    // Track if this card has on_draft action
                    if kind.on_draft().is_some() {
                        cards_with_on_draft[player_id] = Some(*kind);
                    }
                }

                player.hand = remaining_hand.clone();
            }
        }

        let all_hands_empty = self.players.iter().all(|p| p.hand.is_empty());

        if !all_hands_empty {
            self.pass_hands();
        }

        // process on_draft actions 
        let direction = self.get_current_pass_direction();
        let num_players = self.players.len();
        
        let mut hands: Vec<HashMap<CardKind, usize>> = self.players.iter().map(|p| p.hand.clone()).collect();
        
        for (player_id, card_kind_opt) in cards_with_on_draft.iter().enumerate() {
            if let Some(card_kind) = card_kind_opt {
                if let Some(on_draft_fn) = card_kind.on_draft() {
                    on_draft_fn(
                        player_id,
                        num_players,
                        direction,
                        &mut hands,
                        &mut self.deck,
                        &mut self.rng
                    )
                    .map_err(|_| GameError::InvalidConfig)?;
                }
            }
        }
        
        if !all_hands_empty {
            for (i, player) in self.players.iter_mut().enumerate() {
                player.hand = hands[i].clone();
            }
            self.next_turn();
        } else {
            if self.round < self.round_count {
                self.start_new_round()?;
            }
        }

        Ok(())
    }

    pub fn is_game_over(&self) -> bool {
        self.round >= self.round_count && self.players.iter().all(|p| p.hand.is_empty())
    }

    /// Calculate score for a specific player based on their public cards
    /// Returns both the total score and a detailed breakdown
    pub fn calculate_player_score(&self, player_id: usize) -> Result<(f32, ScoreBreakdown), GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }
        
        let player = &self.players[player_id];
        let breakdown = crate::engine::scoring::calculate_player_score(player, &self.players, player_id);
        Ok((breakdown.total_score, breakdown))
    }

    // Public API methods

    /// Activate drink tray for a player
    pub fn activate_drink_tray(&mut self, player_id: usize) -> Result<(), GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }

        let player = &mut self.players[player_id];
        if let Some(drink_tray_count) = player.public_cards.get_mut(&CardKind::DrinkTray) {
            *drink_tray_count -= 1;
            if *drink_tray_count == 0 {
                player.public_cards.remove(&CardKind::DrinkTray);
            }
            *player.hand.entry(CardKind::DrinkTray).or_insert(0) += 1;
            Ok(())
        } else {
            Err(GameError::InvalidConfig)
        }
    }

    /// Get current player's hand
    pub fn get_player_hand(&self, player_id: usize) -> Result<&HashMap<CardKind, usize>, GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }
        Ok(&self.players[player_id].hand)
    }

    /// Get all players' public information
    pub fn get_players_public(&self) -> Vec<PlayerPublic> {
        self.players.iter().map(|p| p.to_public()).collect()
    }

    /// Get a specific player's public information
    pub fn get_player_public(&self, player_id: usize) -> Result<PlayerPublic, GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }
        Ok(self.players[player_id].to_public())
    }

    /// Get current game status
    pub fn get_game_status(&self) -> GameStatus {
        GameStatus {
            round: self.round,
            turn: self.turn,
            round_count: self.round_count,
            pass_direction: self.get_current_pass_direction(),
            is_game_over: self.is_game_over(),
            player_turn_states: self.player_turn_states.clone(),
        }
    }

    /// Get current round number
    pub fn get_round(&self) -> usize {
        self.round
    }

    /// Get current turn number
    pub fn get_turn(&self) -> usize {
        self.turn
    }

    /// Get number of players
    pub fn num_players(&self) -> usize {
        self.players.len()
    }


    /// Get player turn state
    pub fn get_player_turn_state(&self, player_id: usize) -> Result<PlayerTurnState, GameError> {
        if player_id >= self.player_turn_states.len() {
            return Err(GameError::InvalidConfig);
        }
        Ok(self.player_turn_states[player_id])
    }

    // distribute new cards to players
    pub fn start_new_round(&mut self) -> Result<(), GameError> {
        if self.round >= self.round_count {
            return Err(GameError::InvalidConfig);
        }
        
        let num_players = self.players.len();
        let cards_per_player: usize = constants::cards_per_player(num_players)
            .ok_or(GameError::InvalidConfig)?;

        self.distribute_cards(cards_per_player)?;
        self.round += 1;
        self.turn = 1;
        self.reset_turn_states();
        Ok(())
    }

    pub fn next_turn(&mut self) {
        self.turn += 1;
        self.reset_turn_states();
    }

    // shuffle deck
    fn build_deck(&mut self, distribution_opt: Option<HashMap<CardKind, usize>>) {
        let distribution = distribution_opt.unwrap_or_else(|| {
            // Fallback to default if not provided
            use CardKind::*;
            let mut dist = HashMap::new();
            dist.insert(TapiocaPearl, 14);
            dist.insert(BrownSugarMilkTea, 14);
            dist.insert(ThaiTea, 12);
            dist.insert(MochiIceCream, 8);
            dist.insert(Matcha, 10);
            dist.insert(MysteryTea, 6);
            dist.insert(PoppingBubbles, 10);
            dist.insert(MangoTea, 10);
            dist.insert(LycheeTea, 10);
            dist.insert(PassionFruitTea, 10);
            dist.insert(DrinkTray, 10);
            dist
        });
        
        // Set initial distribution for auto-reshuffle
        self.deck.set_initial_distribution(distribution.clone());
        
        // Add cards to deck
        for (kind, count) in distribution {
            self.deck.add(kind, count);
        }
    }
}

/// Wrapper for Game that provides a player-specific view
pub struct GamePlayerView<'a> {
    pub game: &'a mut Game,
    pub player_id: usize,
}

impl<'a> GamePlayerView<'a> {
    pub fn new(game: &'a mut Game, player_id: usize) -> Self {
        Self { game, player_id }
    }
}

impl<'a> crate::tui::GameInterface for GamePlayerView<'a> {
    fn get_hand(&self) -> HashMap<CardKind, usize> {
        self.game.get_player_hand(self.player_id)
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    fn get_game_status(&self) -> GameStatus {
        self.game.get_game_status()
    }

    fn get_players_public(&self) -> Vec<PlayerPublic> {
        self.game.get_players_public()
    }

    fn submit_turn(&mut self, selected: HashMap<CardKind, usize>, remaining: HashMap<CardKind, usize>) -> Result<(), String> {
        self.game.validate_hand_submission(self.player_id, &selected, &remaining)
            .map_err(|e| format!("{:?}", e))?;
        self.game.mark_player_selected(self.player_id)
            .map_err(|e| format!("{:?}", e))?;

        // Update the player's hand with the remaining cards
        if let Some(player) = self.game.players.get_mut(self.player_id) {
            player.hand = remaining;
        }

        Ok(())
    }

    fn get_player_id(&self) -> usize {
        self.player_id
    }

    fn activate_drink_tray(&mut self) -> Result<(), String> {
        if let Some(player) = self.game.players.get_mut(self.player_id) {
            if let Some(drink_tray_count) = player.public_cards.get_mut(&CardKind::DrinkTray) {
                *drink_tray_count -= 1;
                if *drink_tray_count == 0 {
                    player.public_cards.remove(&CardKind::DrinkTray);
                }
                *player.hand.entry(CardKind::DrinkTray).or_insert(0) += 1;
                Ok(())
            } else {
                Err("No DrinkTray in public cards".to_string())
            }
        } else {
            Err("Invalid player id".to_string())
        }
    }
}
