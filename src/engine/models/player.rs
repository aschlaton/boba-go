use std::collections::HashMap;
use crate::engine::models::cards::CardKind;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub username: String,
    pub hand: HashMap<CardKind, usize>,
    pub public_cards: HashMap<CardKind, usize>,
    /// Tracks how many of each fruit tea card are boosted by Popping Bubbles
    pub boosted_fruit_teas: HashMap<CardKind, usize>,
}

impl Player {
    pub fn to_public(&self) -> PlayerPublic {
        PlayerPublic {
            id: self.id,
            name: self.username.clone(),
            public_cards: self.public_cards.clone(),
            boosted_fruit_teas: self.boosted_fruit_teas.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerPublic {
    pub id: usize,
    pub name: String,
    pub public_cards: HashMap<CardKind, usize>,
    pub boosted_fruit_teas: HashMap<CardKind, usize>,
}

