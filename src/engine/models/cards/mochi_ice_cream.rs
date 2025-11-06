use super::base::Card;

pub struct MochiIceCream;

impl Card for MochiIceCream {
    const NAME: &'static str = "Mochi Ice Cream";
    const DESCRIPTION: &'static str = "Grants 1/3/6/10/15 points based on quantity";
    const FLAVOR_TEXT: &'static str = "like an ice cream filled dumpling";
    const SCORE: u32 = 0;
}
