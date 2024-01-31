use crate::engine::prelude::*;

use super::util::id::ID;

pub type PermanentID = ID<Permanent>;

pub struct Permanent {
    pub name: String,
    pub flavor: String,
    pub types: Vec<CardType>,
    pub card: Option<Card>,
    pub owner: PlayerID,
    pub base_power: i32,
    pub base_toughness: i32,
    pub id: PermanentID,
    pub tapped: bool,
    pub abilities: Vec<AbilityID>,
}

impl Permanent {
    pub fn untap(&mut self) {
        self.tapped = false;
    }
}
