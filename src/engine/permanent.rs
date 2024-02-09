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
            name: card.attrs.name.clone(),
            flavor: card.attrs.flavor.clone(),
            card: Some(card.id),
            is_token: false,
            owner,
            base_power: card.attrs.power_toughness.unwrap().0,
            base_toughness: card.attrs.power_toughness.unwrap().1,
            type_line: card.attrs.type_line.clone(),
            id,
            abilities: vec![], //TODO grab ability IDs from permanent
            tapped: false,
            summoning_sickness: true,
        }
    }

}
