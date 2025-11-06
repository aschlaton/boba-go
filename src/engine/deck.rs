use std::collections::HashMap;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::engine::models::CardKind;

pub struct Deck {
    cards: HashMap<CardKind, usize>,
    total: usize,
    initial_distribution: HashMap<CardKind, usize>,
}

impl Deck {
    pub fn new() -> Self {
        Deck {
            cards: HashMap::new(),
            total: 0,
            initial_distribution: HashMap::new(),
        }
    }

    pub fn with_cards(cards: HashMap<CardKind, usize>) -> Self {
        let total = cards.values().sum();
        let initial_distribution = cards.clone();
        Deck { cards, total, initial_distribution }
    }

    pub fn add(&mut self, kind: CardKind, count: usize) {
        *self.cards.entry(kind).or_insert(0) += count;
        self.total += count;
    }

    pub fn set_initial_distribution(&mut self, distribution: HashMap<CardKind, usize>) {
        self.initial_distribution = distribution;
    }

    fn reshuffle(&mut self) {
        // Reset to initial distribution
        self.cards = self.initial_distribution.clone();
        self.total = self.cards.values().sum();
    }

    pub fn draw(&mut self, rng: &mut ChaCha8Rng) -> Option<CardKind> {
        if self.total == 0 {
            // Auto-reshuffle if deck is empty
            if !self.initial_distribution.is_empty() {
                self.reshuffle();
            } else {
                return None;
            }
        }

        let mut pick = rng.gen_range(0..self.total);

        for (kind, count) in &mut self.cards {
            if pick < *count {
                *count -= 1;
                self.total -= 1;
                return Some(*kind);
            }
            pick -= *count;
        }

        None
    }

    pub fn size(&self) -> usize {
        self.total
    }

    pub fn extend(&mut self, other_cards: impl IntoIterator<Item = CardKind>) {
        for kind in other_cards {
            self.add(kind, 1);
        }
    }
}
