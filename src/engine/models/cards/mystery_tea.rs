use std::collections::HashMap;
use rand_chacha::ChaCha8Rng;

use super::CardKind;
use super::base::{Card, OnDraftActionFn};
use crate::engine::deck::Deck;
use crate::engine::state::PassDirection;

pub struct MysteryTea;

impl Card for MysteryTea {
    const NAME: &'static str = "Mystery Tea";
    const DESCRIPTION: &'static str = "+2 points. After drafting this card, the remaining cards in this hand are replaced with random  cards from the deck. Each set of 3 unique non-fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 2;
    const PLAYABLE: bool = false;

    fn on_draft() -> Option<OnDraftActionFn> {
        Some(Self::on_draft_action)
    }
}

impl MysteryTea {
    fn on_draft_action(
        drafting_player_id: usize,
        num_players: usize,
        pass_direction: PassDirection,
        player_hands: &mut [HashMap<CardKind, usize>],
        deck: &mut Deck,
        rng: &mut ChaCha8Rng,
    ) -> Result<(), String> {
        // Find which player received the drafted hand (the next player)
        let receiving_player_id = match pass_direction {
            PassDirection::Left => (drafting_player_id + 1) % num_players,
            PassDirection::Right => (drafting_player_id + num_players - 1) % num_players,
        };

        // Get the hand to transform (the passed hand, now at receiving player)
        let hand_to_transform = &player_hands[receiving_player_id];
        
        // Count total cards in hand
        let hand_size: usize = hand_to_transform.values().sum();

        // Collect all cards in hand and add them back to deck
        for (kind, count) in hand_to_transform.iter() {
            for _ in 0..*count {
                deck.extend(std::iter::once(*kind));
            }
        }

        // Draw the same number of new cards from the deck
        let mut new_hand: HashMap<CardKind, usize> = HashMap::new();
        for _ in 0..hand_size {
            if let Some(card) = deck.draw(rng) {
                *new_hand.entry(card).or_insert(0) += 1;
            }
        }

        // Replace the receiving player's hand with the new hand
        player_hands[receiving_player_id] = new_hand;

        Ok(())
    }
}
