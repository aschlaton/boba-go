#[derive(Debug, Clone)]
pub struct GameConfig {
    pub player_names: Vec<String>,
    pub seed: Option<u64>,
    pub round_count: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_names: Vec::new(),
            seed: None,
            round_count: 3,
        }
    }
}

