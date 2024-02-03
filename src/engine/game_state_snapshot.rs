use super::prelude::*;

pub struct GameStateSnapshot {
    pub opponent_perms: Vec<PermanentSnapshot>,
    pub perms: Vec<PermanentSnapshot>,
    pub hand: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
}

pub struct PermanentSnapshot {
    pub permanent: Attributes,
    pub state: State,
    pub id: PermanentID,
}

pub struct CardSnapshot {
    pub card: Attributes,
    pub id: CardID,
}

pub struct State {
    pub is_tapped: bool,
}

pub struct Attributes {
    pub name: String,
    pub card_type: Vec<CardType>,
    pub card_subtypes: Vec<String>,
    pub cost: Option<ManaCost>,
    pub abilities: Vec<String>,
    pub flavor: String,
    pub power_toughness: Option<(i32, i32)>,
}
