use crate::engine::prelude::*;

use super::util::id::ID;

pub type CardID = ID<Card>;

pub struct Card {
    pub owner: PlayerID,
    pub id: CardID,
    pub perm_abilities: Vec<AbilityID>,
    pub card_plays: Vec<CardPlayID>,
    pub attrs: Attributes,
}

impl Card {
    pub fn new(attrs: Attributes, id: CardID, perm_abilities: Vec<AbilityID>, card_plays: Vec<CardPlayID>, owner: PlayerID) -> Self {
        Self {
            attrs,
            id, 
            perm_abilities, 
            card_plays, // todo insert actual card plays!!
            owner
        }
    }
}

pub struct LatentCard {
    pub attributes: Attributes,
    pub perm_abilities: Vec<LatentAbility>,
    pub card_plays: Vec<CardPlay>,
}

pub struct Attributes {
    pub name: String,
    pub type_line: TypeLine,
    pub cost: Option<ManaCost>,
    pub flavor: String,
    pub power_toughness: Option<PowerToughness>,
}

impl LatentCard {
    pub fn new(name: String, cost: ManaCost,  flavor: String, type_line: TypeLine, perm_abilities: Vec<LatentAbility>, card_plays: Vec<CardPlay>, power: Option<i32>, toughness: Option<i32>) -> Self {
        let power_toughness = power.map(|p| toughness.map(|t| PowerToughness::new(p,t))).unwrap_or(None);
        let cost = Some(cost);
        let attributes = Attributes { 
            name,
            cost,
            flavor,
            type_line,
            power_toughness,
        };
        Self {
            attributes,
            perm_abilities,
            card_plays,
        }
    }
}

