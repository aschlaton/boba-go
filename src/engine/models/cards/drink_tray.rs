use super::base::Card;

pub struct DrinkTray;

impl Card for DrinkTray {
    const NAME: &'static str = "Drink Tray";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 0;
}

