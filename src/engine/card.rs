use crate::engine::prelude::*;

use super::util::id::ID;

pub type CardID = ID<Card>;

#[derive(Clone)]
pub struct Card {
    pub owner: PlayerID,
    pub id: CardID,
    pub abilities: Vec<AbilityID>,
    pub base: LatentCard,
}

impl Card {
    pub fn new(base: LatentCard, id: CardID, abilities: Vec<AbilityID>, owner: PlayerID) -> Self {
        Self {base, id, abilities, owner}
    }
}

#[derive(Clone)]
pub struct LatentCard {
    pub cost: ManaCost,
    pub name: String,
    pub flavor: String,
    pub type_line: TypeLine,
    pub perm_abilities: Vec<LatentAbility>,
    pub card_abilities: Vec<LatentAbility>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
}

impl LatentCard {
    pub fn new(name: String, cost: ManaCost,  flavor: String, type_line: TypeLine, perm_abilities: Vec<LatentAbility>, card_abilities: Vec<LatentAbility>, power: Option<i32>, toughness: Option<i32>) -> Self {
        Self {
            name,
            cost,
            flavor,
            type_line,
            perm_abilities,
            card_abilities,
            power,
            toughness
        }
    }
}

