use super::base::Card;

pub struct PassionFruitTea;

impl Card for PassionFruitTea {
    const NAME: &'static str = "Passion Fruit Tea";
    const DESCRIPTION: &'static str = "+1 point. Each set of 3 unique fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 1;
    const PLAYABLE: bool = false;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

