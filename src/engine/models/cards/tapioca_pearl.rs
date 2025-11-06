use super::base::Card;

pub struct TapiocaPearl;

impl Card for TapiocaPearl {
    const NAME: &'static str = "Tapioca Pearl";
    const DESCRIPTION: &'static str = "placeholder";
    const FLAVOR_TEXT: &'static str = "placeholder";
    const SCORE: u32 = 0;
    const PLAYABLE: bool = false;
}
