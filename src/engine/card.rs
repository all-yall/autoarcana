use crate::engine::prelude::*;

use super::util::id::ID;

pub type CardID = ID<Card>;

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
    pub cost: ManaCost,
    pub name: String,
    pub flavor: String,
    pub card_types: Vec<CardType>,
    pub card_subtypes: Vec<String>,
    pub abilities: Vec<LatentAbility>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
}

impl LatentCard {
    pub fn new(name: String, cost: ManaCost,  flavor: String, card_types: Vec<CardType>, card_subtypes: Vec<String>, abilities: Vec<LatentAbility>, power: Option<i32>, toughness: Option<i32>) -> Self {
        Self {
            name,
            cost,
            flavor,
            card_types,
            card_subtypes,
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
