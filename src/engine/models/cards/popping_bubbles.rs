use super::base::Card;

pub struct PoppingBubbles;

impl Card for PoppingBubbles {
    const NAME: &'static str = "Popping Bubbles";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 0;
    const PLAYABLE: bool = false;
}

