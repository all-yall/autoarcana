use crate::engine::prelude::*;

use super::util::id::ID;

pub type PlayerID = ID<4>;

pub struct Player {
    pub id: PlayerID,
    pub deck: Deck,
    pub graveyard: Deck,
    pub hand: Deck,
    pub life_total: i32,
    pub mana_pool: Vec<ManaType>,
}

impl Player {
    pub fn new(deck: Deck, id: PlayerID) -> Self {
        Self {
            deck,
            graveyard: Deck::empty(),
            hand: Deck::empty(),
            life_total: 20,
            mana_pool: vec![],
            id,
        }
    }
}
