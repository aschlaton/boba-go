use crate::engine::models::cards::Card;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub username: String,
    pub hand: Vec<Card>,
    pub public_cards: Vec<Card>,
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
    pub public_cards: Vec<Card>,
}

