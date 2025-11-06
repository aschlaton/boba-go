use std::collections::HashMap;
use crate::engine::models::cards::CardKind;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub username: String,
    pub hand: HashMap<CardKind, usize>,
    pub public_cards: HashMap<CardKind, usize>,
}

impl Player {
    pub fn to_public(&self) -> PlayerPublic {
        PlayerPublic {
            id: self.id,
            name: self.username.clone(),
            public_cards: self.public_cards.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerPublic {
    pub id: usize,
    pub name: String,
    pub public_cards: HashMap<CardKind, usize>,
}

