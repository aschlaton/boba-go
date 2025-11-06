pub mod base;
pub mod aloe_jelly;
pub mod brown_sugar_milk_tea;
pub mod matcha;
pub mod mochi_ice_cream;
pub mod mystery_tea;
pub mod tapioca_pearl;
pub mod thai_tea;

pub use base::{Card, OnDraftActionFn};
pub use aloe_jelly::AloeJelly;
pub use brown_sugar_milk_tea::BrownSugarMilkTea;
pub use matcha::Matcha;
pub use mochi_ice_cream::MochiIceCream;
pub use mystery_tea::MysteryTea;
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
    Matcha => Matcha,
    AloeJelly => AloeJelly,
    MysteryTea => MysteryTea,
}
