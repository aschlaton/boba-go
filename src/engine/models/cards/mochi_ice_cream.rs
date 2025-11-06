use super::base::Card;

pub struct MochiIceCream;

impl Card for MochiIceCream {
    const NAME: &'static str = "Mochi Ice Cream";
    const DESCRIPTION: &'static str = "Points based on quantity: 1 (1 card), 3 (2 cards), 6 (3 cards), 10 (4 cards), 15 (5+ cards)";
    const FLAVOR_TEXT: &'static str = "like an ice cream filled dumpling";
    const SCORE: u32 = 0;
}
