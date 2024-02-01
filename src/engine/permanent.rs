use crate::engine::prelude::*;

use super::util::id::ID;

pub type PermanentID = ID<Permanent>;

pub struct Permanent {
    pub name: String,
    pub flavor: String,
    pub types: Vec<CardType>,
    pub subtypes: Vec<String>,
    pub card: Option<Card>,
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

    pub fn from_card(card: &LatentCard, id: PermanentID, owner: PlayerID) -> Self {
        Self {
            name: card.name.clone(),
            flavor: card.flavor.clone(),
            types: card.card_types.clone(),
            subtypes: card.card_subtypes.clone(),
            card: None,
            owner,
            base_power: card.power.unwrap(),
            base_toughness: card.toughness.unwrap(),
            id,
            abilities: vec![],
            tapped: false,
            summoning_sickness: true,
        }
    }

}
