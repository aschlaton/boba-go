use super::base::Card;

pub struct LycheeTea;

impl Card for LycheeTea {
    const NAME: &'static str = "Lychee Tea";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 3;
    const PLAYABLE: bool = false;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

