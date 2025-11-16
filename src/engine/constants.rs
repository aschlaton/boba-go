use std::collections::HashMap;
use std::sync::OnceLock;

const CARDS_PER_PLAYER_DATA: &[(usize, usize)] = &[
    (2, 10),
    (3, 9),
    (4, 8),
    (5, 7),
];

static CARDS_PER_PLAYER: OnceLock<HashMap<usize, usize>> = OnceLock::new();

fn cards_per_player_map() -> &'static HashMap<usize, usize> {
    CARDS_PER_PLAYER.get_or_init(|| {
        CARDS_PER_PLAYER_DATA.iter().copied().collect()
    })
}

pub fn cards_per_player(num_players: usize) -> Option<usize> {
    cards_per_player_map().get(&num_players).copied()
}

