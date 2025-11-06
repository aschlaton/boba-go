use super::base::Card;

pub struct MangoTea;

impl Card for MangoTea {
    const NAME: &'static str = "Mango Tea";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 2;
    const PLAYABLE: bool = false;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

