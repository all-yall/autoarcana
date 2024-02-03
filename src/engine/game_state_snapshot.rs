use super::prelude::*;

struct GameStateSnapshot {
    opponent_perms: Vec<PermanentSnapshot>,
    perms: Vec<PermanentSnapshot>,
    hand: Vec<CardSnapshot>,
    graveyard: Vec<CardSnapshot>,
}

struct PermanentSnapshot {
    permanent: Attributes,
    state: State,
    id: PermanentID,
}

struct CardSnapshot {
    card: Attributes,
    id: CardID,
}

struct State {
    is_tapped: bool,
}

struct Attributes {
    name: String,
    card_type: Vec<CardType>,
    card_subtypes: Vec<String>,
    cost: Option<ManaCost>,
    abilities: Vec<String>,
    flavor: String,
    power_toughness: Option<(i32, i32)>,

}
