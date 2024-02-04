use crate::engine::prelude::*;

use super::util::id::ID;

pub type PlayerID = ID<Player>;

pub struct Player {
    pub id: PlayerID,
    pub life_total: i32,
    pub mana_pool: Vec<ManaType>,
}

impl Player {
    pub fn new(id: PlayerID) -> Self {
        Self {
            life_total: 20,
            mana_pool: vec![],
            id,
        }
    }
}
