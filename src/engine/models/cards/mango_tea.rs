use super::base::Card;

pub struct MangoTea;

impl Card for MangoTea {
    const NAME: &'static str = "Mango Tea";
    const DESCRIPTION: &'static str = "+2 points. Each set of 3 unique fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 2;
    const PLAYABLE: bool = false;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

