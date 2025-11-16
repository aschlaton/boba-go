use super::base::Card;

pub struct PoppingBubbles;

impl Card for PoppingBubbles {
    const NAME: &'static str = "Popping Bubbles";
    const DESCRIPTION: &'static str = "Pairs with fruit teas to triple their points. Each Popping Bubbles can boost one fruit tea.";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 0;
    const PLAYABLE: bool = false;
}

