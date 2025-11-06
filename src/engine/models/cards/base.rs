use std::collections::HashMap;
use rand_chacha::ChaCha8Rng;

use super::CardKind;
use crate::engine::deck::Deck;

/// Function signature for on_draft actions
/// Takes the drafting player context and mutable access to all player hands
/// Actions fire when a card is drafted (selected by a player)
/// Actions can modify any player's hand directly
pub type OnDraftActionFn = fn(
    drafting_player_id: usize,
    num_players: usize,
    pass_direction: crate::engine::state::PassDirection,
    player_hands: &mut [HashMap<CardKind, usize>],
    deck: &mut Deck,
    rng: &mut ChaCha8Rng,
) -> Result<(), String>;

/// Card trait - all cards must implement this
pub trait Card {
    const NAME: &'static str;
    const SCORE: u32;
    const PLAYABLE: bool = true;

    fn on_draft() -> Option<OnDraftActionFn> {
        None
    }
}
