use rand::prelude::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::engine::constants;
use crate::engine::models::{Card, CardKind, GameConfig, Player};

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Not enough cards in deck for new round")]
    NotEnoughCards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerTurnState {
    NotSelected,
    Selected,
}

pub struct Game {
    pub seed: u64,
    rng: ChaCha8Rng,
    pub deck: Vec<Card>,
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
        
        let mut players: Vec<Player> = config.player_names
            .into_iter()
            .enumerate()
            .map(|(id, username)| Player {
                id,
                username,
                hand: Vec::new(),
                public_cards: Vec::new(),
            })
            .collect();

        let player_turn_states = vec![PlayerTurnState::NotSelected; num_players];
        
        let mut game = Game {
            seed,
            rng,
            deck: Vec::new(),
            players: Vec::new(),
            round: 1,
            turn: 1,
            player_turn_states,
            round_count: config.round_count,
        };
        
        game.deck = game.build_deck();
        game.players = players;
        game.distribute_cards(cards_per_player)?;
        Ok(game)
    }


    // deal cards to players
    fn distribute_cards(&mut self, cards_per_player: usize) -> Result<(), GameError> {
        let num_players = self.players.len();
        let total_cards_needed = num_players * cards_per_player;
        if self.deck.len() < total_cards_needed {
            return Err(GameError::NotEnoughCards);
        }

        for (i, player) in self.players.iter_mut().enumerate() {
            let start = i * cards_per_player;
            let end: usize = start + cards_per_player;
            player.hand.extend_from_slice(&self.deck[start..end]);
        }

        self.deck = self.deck.split_off(total_cards_needed);
        Ok(())
    }

    pub fn get_current_pass_direction(&self) -> PassDirection {
        Self::get_pass_direction(self.round)
    }

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
    
    pub fn validate_hand_submission(&self, player_id: usize, final_hand: &[Card]) -> Result<(), GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }
        
        let current_hand_size = self.players[player_id].hand.len();
        
        if final_hand.len() != current_hand_size - 1 {
            return Err(GameError::InvalidConfig);
        }
        
        Ok(())
    }

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

    pub fn process_turn(&mut self, pending_hands: Vec<Option<Vec<Card>>>) -> Result<(), GameError> {
        if !self.all_players_selected() {
            return Err(GameError::InvalidConfig);
        }

        if pending_hands.len() != self.players.len() {
            return Err(GameError::InvalidConfig);
        }

        for (player_id, pending_hand_opt) in pending_hands.iter().enumerate() {
            if let Some(final_hand) = pending_hand_opt {
                let current_hand = &self.players[player_id].hand;
                let selected_cards: Vec<Card> = current_hand
                    .iter()
                    .filter(|card| !final_hand.contains(card))
                    .cloned()
                    .collect();
                
                for card in &selected_cards {
                    self.players[player_id].public_cards.push(card.clone());
                }
                
                self.players[player_id].hand = final_hand.clone();
            }
        }

        self.pass_hands();
        
        let all_hands_empty = self.players.iter().all(|p| p.hand.is_empty());
        if all_hands_empty {
            if self.round < self.round_count {
                self.start_new_round()?;
            }
        } else {
            self.next_turn();
        }
        
        Ok(())
    }

    pub fn is_game_over(&self) -> bool {
        self.round >= self.round_count && self.players.iter().all(|p| p.hand.is_empty())
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
    // TODO: probably add some way to configure card counts
    fn build_deck(&mut self) -> Vec<Card> {
        use CardKind::*;
        let mut kinds = Vec::<CardKind>::new();
        kinds.extend(std::iter::repeat(TapiocaPearl).take(14));
        kinds.extend(std::iter::repeat(BrownSugarMilkTea).take(14));
        kinds.extend(std::iter::repeat(ThaiTea).take(12));
        kinds.extend(std::iter::repeat(MochiIceCream).take(8));
        kinds.extend(std::iter::repeat(Matcha).take(10));
        kinds.extend(std::iter::repeat(AloeJelly).take(10));

        let mut deck: Vec<Card> = kinds.into_iter().enumerate()
            .map(|(i, k)| Card { id: i as u32, kind: k })
            .collect();
        deck.shuffle(&mut self.rng);
        deck
    }
}
