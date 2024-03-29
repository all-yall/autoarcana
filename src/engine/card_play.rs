use std::fmt::Debug;

use super::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Debug, Hash)]
pub struct AssignedCardPlay {
    pub card: CardID,
    pub card_play: CardPlayID,
}

impl AssignedCardPlay {
    pub fn new(card: CardID, card_play: CardPlayID) -> Self {
        Self {card, card_play}
    }
}

pub type CardPlayID = ID<CardPlay>;

pub struct CardPlay {
    pub description: String,
    pub spawn: Box<dyn Spawner>,
    pub speed: AbilitySpeed,
}
impl CardPlay {
    pub fn new(spawn: Box<dyn Spawner>, description: String, speed: AbilitySpeed) -> Self {
        Self{spawn, description, speed}
    }
}

pub trait Spawner {
    fn spawn(&self, card: CardID, game: &Game) -> Object;
    fn cost(&self, card_id: CardID, game: &Game) -> Cost;
}

