use crate::engine::prelude::*;

use super::util::id::ID;

pub type CardID = ID<2>;
#[derive(Clone)]
pub struct Card {
    pub owner: PlayerID,
    pub id: CardID,
    pub base: LatentCard,
}

impl Card {
    pub fn new(base: LatentCard, id: CardID, owner: PlayerID) -> Self {
        Self {base, id, owner}
    }
}

#[derive(Clone)]
pub struct LatentCard {
    pub name: String,
    pub flavor: String,
    pub card_types: Vec<CardType>,
    pub abilities: Vec<LatentAbility>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
}

impl LatentCard {
    pub fn new(name: String, flavor: String, card_types: Vec<CardType>, abilities: Vec<LatentAbility>, power: Option<i32>, toughness: Option<i32>) -> Self {
        Self {
            name,
            flavor,
            card_types,
            abilities,
            power,
            toughness
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq)]
pub enum CardType {
    Basic,
    Land(ManaType),
    Creature,
    Artifact,
    Sorcery,
    Instant,
    Enchantment,

    Legendary,
    Goblin,
    Warrior,
}
