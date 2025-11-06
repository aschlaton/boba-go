use std::collections::HashMap;
use super::cards::CardKind;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub player_names: Vec<String>,
    pub seed: Option<u64>,
    pub round_count: usize,
    pub card_distribution: Option<HashMap<CardKind, usize>>,
}

impl Default for GameConfig {
    fn default() -> Self {
        use CardKind::*;
        let mut default_distribution = HashMap::new();
        default_distribution.insert(TapiocaPearl, 14);
        default_distribution.insert(BrownSugarMilkTea, 14);
        default_distribution.insert(ThaiTea, 12);
        default_distribution.insert(MochiIceCream, 8);
        default_distribution.insert(Matcha, 10);
        default_distribution.insert(MysteryTea, 6);
        default_distribution.insert(PoppingBubbles, 10);
        default_distribution.insert(MangoTea, 10);
        default_distribution.insert(LycheeTea, 10);
        default_distribution.insert(PassionFruitTea, 10);
        
        Self {
            player_names: Vec::new(),
            seed: None,
            round_count: 3,
            card_distribution: Some(default_distribution),
        }
    }
}

