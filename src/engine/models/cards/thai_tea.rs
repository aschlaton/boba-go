use super::base::Card;

pub struct ThaiTea;

impl Card for ThaiTea {
    const NAME: &'static str = "Thai Tea";
    const DESCRIPTION: &'static str = "+2 points. Each set of 3 unique non-fruit teas gives a +5 bonus!";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 2;
    const PLAYABLE: bool = false;
}
