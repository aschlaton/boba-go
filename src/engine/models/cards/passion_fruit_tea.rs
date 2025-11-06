use super::base::Card;

pub struct PassionFruitTea;

impl Card for PassionFruitTea {
    const NAME: &'static str = "Passion Fruit Tea";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 1;
    
    fn is_fruit_tea() -> bool {
        true
    }
}

