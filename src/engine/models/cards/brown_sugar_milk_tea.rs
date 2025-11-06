use super::base::Card;

pub struct BrownSugarMilkTea;

impl Card for BrownSugarMilkTea {
    const NAME: &'static str = "Brown Sugar Milk Tea";
    const DESCRIPTION: &'static str = "+1 point. Each set of 3 unique non-fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "a classic blend";
    const SCORE: u32 = 1;
    const PLAYABLE: bool = false;
}
