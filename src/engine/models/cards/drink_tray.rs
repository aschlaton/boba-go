use super::base::Card;

pub struct DrinkTray;

impl Card for DrinkTray {
    const NAME: &'static str = "Drink Tray";
    const DESCRIPTION: &'static str = "Can be activated to return it to your hand and give you the ability to draft another card for that turn";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 0;
    const PLAYABLE: bool = false;
}

