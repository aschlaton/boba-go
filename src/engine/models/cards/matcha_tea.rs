use super::base::Card;

pub struct MatchaTea;

impl Card for MatchaTea {
    const NAME: &'static str = "Matcha Tea";
    const DESCRIPTION: &'static str = "+3 points. Each set of 3 unique non-fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 3;
    const PLAYABLE: bool = false;
}

