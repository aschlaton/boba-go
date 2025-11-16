use super::base::Card;

pub struct TapiocaPearl;

impl Card for TapiocaPearl {
    const NAME: &'static str = "Tapioca Pearl";
    const DESCRIPTION: &'static str = "Most Tapioca Pearls: +6 points (split if tied). Least Tapioca Pearls: -6 points (split if tied).";
    const FLAVOR_TEXT: &'static str = "";
    const SCORE: u32 = 0;
    const PLAYABLE: bool = false;
}
