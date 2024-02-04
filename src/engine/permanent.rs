use crate::engine::prelude::*;

use super::util::id::ID;

pub type PermanentID = ID<Permanent>;

pub struct Permanent {
    pub name: String,
    pub flavor: String,
    pub type_line: TypeLine,
    pub card: Option<CardID>,
    pub is_token: bool,
    pub owner: PlayerID,
    pub base_power: i32,
    pub base_toughness: i32,
    pub id: PermanentID,
    pub tapped: bool,
    pub abilities: Vec<AbilityID>,
    pub summoning_sickness: bool,
}

impl Permanent {
    pub fn untap(&mut self) {
        self.tapped = false;
    }

    pub fn from_card(card: &Card, id: PermanentID, owner: PlayerID) -> Self {
        Self {
            name: card.base.name.clone(),
            flavor: card.base.flavor.clone(),
            card: Some(card.id),
            is_token: false,
            owner,
            base_power: card.base.power.unwrap(),
            base_toughness: card.base.toughness.unwrap(),
            type_line: card.base.type_line.clone(),
            id,
            abilities: vec![],
            tapped: false,
            summoning_sickness: true,
        }
    }

}
