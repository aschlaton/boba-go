use rand::prelude::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::engine::cards::{Card, CardKind};

#[derive(Debug, Clone)]
pub struct Config {
    pub num_players: usize,
    pub seed: Option<u64>,
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid configuration")]
    InvalidConfig,
}

#[derive(Debug, Clone)]
pub struct PlayerPublic {
    pub id: usize,
    pub name: String,
}

pub struct Game {
    pub seed: u64,
    rng: ChaCha8Rng,
    pub deck: Vec<Card>,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, GameError> {
        if config.num_players < 2 {
            return Err(GameError::InvalidConfig);
        }

        let seed = config.seed.unwrap_or_else(rand::random);
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let mut game = Game {
            seed,
            rng,
            deck: Vec::new(),
        };
        
        game.deck = game.build_deck();
        Ok(game)
    }

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
