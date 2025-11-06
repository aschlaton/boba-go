use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardKind {
    TapiocaPearl,      
    BrownSugarMilkTea,
    ThaiTea,          
    MochiIceCream,     
    Matcha,           
    AloeJelly
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: u32,
    pub kind: CardKind,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CardKind::*;
        let name = match self.kind {
            TapiocaPearl      => "Tapioca Pearl",
            BrownSugarMilkTea => "Brown Sugar Milk Tea",
            ThaiTea           => "Thai Tea",
            MochiIceCream     => "Mochi Ice Cream",
            Matcha            => "Matcha",
            AloeJelly         => "Aloe Jelly",
        };
        write!(f, "{name}")
    }
}

