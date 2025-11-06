pub mod base;
pub mod brown_sugar_milk_tea;
pub mod lychee_tea;
pub mod mango_tea;
pub mod matcha_tea;
pub mod mochi_ice_cream;
pub mod mystery_tea;
pub mod passion_fruit_tea;
pub mod popping_bubbles;
pub mod tapioca_pearl;
pub mod thai_tea;

pub use base::{Card, OnDraftActionFn};
pub use brown_sugar_milk_tea::BrownSugarMilkTea;
pub use lychee_tea::LycheeTea;
pub use mango_tea::MangoTea;
pub use matcha_tea::MatchaTea;
pub use mochi_ice_cream::MochiIceCream;
pub use mystery_tea::MysteryTea;
pub use passion_fruit_tea::PassionFruitTea;
pub use popping_bubbles::PoppingBubbles;
pub use tapioca_pearl::TapiocaPearl;
pub use thai_tea::ThaiTea;

use std::fmt;

macro_rules! define_card_kind {
    ($($variant:ident => $card:ty),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum CardKind {
            $($variant,)*
        }

        impl CardKind {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => <$card>::NAME,)*
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $(Self::$variant => <$card>::DESCRIPTION,)*
                }
            }

            pub fn flavor_text(&self) -> &'static str {
                match self {
                    $(Self::$variant => <$card>::FLAVOR_TEXT,)*
                }
            }

            pub fn score(&self) -> u32 {
                match self {
                    $(Self::$variant => <$card>::SCORE,)*
                }
            }

            pub fn playable(&self) -> bool {
                match self {
                    $(Self::$variant => <$card>::PLAYABLE,)*
                }
            }

            pub fn on_draft(&self) -> Option<OnDraftActionFn> {
                match self {
                    $(Self::$variant => <$card>::on_draft(),)*
                }
            }

            pub fn is_fruit_tea(&self) -> bool {
                match self {
                    $(Self::$variant => <$card>::is_fruit_tea(),)*
                }
            }
        }

        impl fmt::Display for CardKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.name())
            }
        }
    };
}

define_card_kind! {
    TapiocaPearl => TapiocaPearl,
    BrownSugarMilkTea => BrownSugarMilkTea,
    ThaiTea => ThaiTea,
    MochiIceCream => MochiIceCream,
    Matcha => MatchaTea,
    MysteryTea => MysteryTea,
    PoppingBubbles => PoppingBubbles,
    MangoTea => MangoTea,
    LycheeTea => LycheeTea,
    PassionFruitTea => PassionFruitTea,
}
