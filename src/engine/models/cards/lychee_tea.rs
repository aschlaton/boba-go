use super::base::Card;

pub struct LycheeTea;

impl Card for LycheeTea {
    const NAME: &'static str = "Lychee Tea";
    const DESCRIPTION: &'static str = "+3 points. Each set of 3 unique fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 3;
    const PLAYABLE: bool = false;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

