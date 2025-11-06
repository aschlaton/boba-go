use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use std::collections::HashMap;
use crate::engine::constants;
use crate::engine::deck::Deck;
use crate::engine::models::{CardKind, GameConfig, Player};

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
    
    pub fn validate_hand_submission(&self, player_id: usize, final_hand: &HashMap<CardKind, usize>) -> Result<(), GameError> {
        if player_id >= self.players.len() {
            return Err(GameError::InvalidConfig);
        }
        
        let current_hand_size: usize = self.players[player_id].hand.values().sum();
        let final_hand_size: usize = final_hand.values().sum();
        
        if final_hand_size != current_hand_size - 1 {
            return Err(GameError::InvalidConfig);
        }
        
        for (kind, count) in final_hand {
            let current_count = self.players[player_id].hand.get(kind).copied().unwrap_or(0);
            if *count > current_count {
                return Err(GameError::InvalidConfig);
            }
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

                // Add selected cards to public_cards
                for (kind, count) in selected_cards {
                    *player.public_cards.entry(*kind).or_insert(0) += count;
                    
                    // Track if this card has on_draft action
                    if kind.on_draft().is_some() {
                        cards_with_on_draft[player_id] = Some(*kind);
                    }
                }

                // Update player's hand to remaining hand (NOT transformed yet)
                player.hand = remaining_hand.clone();
            }
        }

        self.pass_hands();

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
        
        // Sync hands back to players
        for (i, player) in self.players.iter_mut().enumerate() {
            player.hand = hands[i].clone();
        }

        // check if round is over or continue to next turn
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
            dist.insert(AloeJelly, 10);
            dist.insert(MysteryTea, 6);
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
