use std::fmt::Debug;

use crate::engine::prelude::*;

use super::util::id::ID;

pub type PermanentID = ID<Permanent>;

#[derive(Clone)]
pub struct Permanent {
    pub name: String,
    pub flavor: String,
    pub type_line: TypeLine,
    pub card: Option<CardID>,
    pub is_token: bool,
    pub owner: PlayerID,
    pub power_toughness: Option<PowerToughness>,
    pub id: PermanentID,
    pub tapped: bool,
    pub abilities: Vec<AbilityID>,
    pub summoning_sickness: bool,
    pub damage: i32,
    pub counters: Counters,
}

#[derive(Clone)]
pub struct PowerToughness {
    pub power: i32,
    pub toughness: i32,
    pub base_power:  i32,
    pub base_toughness:  i32,
}

impl PowerToughness {
    pub fn new(power: i32, toughness: i32) -> Self {
        Self {
            power,
            toughness,
            base_power: power,
            base_toughness: toughness,
        }
    }
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
            power_toughness: card.attrs.power_toughness.clone(),
            type_line: card.attrs.type_line.clone(),
            id,
            abilities: vec![], //TODO grab ability IDs from permanent
            tapped: false,
            summoning_sickness: true,
            damage: 0,
            counters: Counters::new(),
        }
    }

}

impl Debug for Permanent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       format!("Perm(name: {:?}, owner: {:?}, id: {:?})", self.name, self.owner, self.id).fmt(f)
    }
}
