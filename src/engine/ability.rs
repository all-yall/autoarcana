use log::warn;

use crate::engine::prelude::*;

use super::util::id::ID;


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Debug, Hash)]
pub struct AssignedAbility {
    pub perm: PermanentID,
    pub ability: AbilityID
}

impl AssignedAbility {
    pub fn new(perm: PermanentID, ability: AbilityID) -> Self {
        Self {perm, ability}
    }
}

pub enum AbilityClass {
    Static(Box<dyn QueryModifier>),
    Triggered(Box<dyn EventModifier>),
    Replacement(Box<dyn EventModifier>),
    Activated(Cost, Box<dyn OneShot>),
}

pub type AbilityID = ID<Ability>;

/// Represents replacement and addition to existing game events.
pub enum ListenResult {
    Replaced(Vec<GameEvent>),
    Triggered(GameEvent, Vec<GameEvent>),
    Ignored(GameEvent),
}

pub struct Ability {
    pub id: AbilityID,
    pub base: LatentAbility,
}

impl Ability {
    pub fn new(base: LatentAbility, id: AbilityID) -> Self {
        Self {
            base,
            id,
        }
    }

    pub fn listen(&self, perm: PermanentID, event: GameEvent, game: &Game) -> ListenResult {
        match self.base.class {
            AbilityClass::Replacement(ref a) => a.listen(self.id, perm, event, game),
            AbilityClass::Triggered(ref a) => a.listen(self.id, perm, event, game),
            _ => {
                warn!("Listen called for {:?}, which is neither Replacement, nor Triggered", self.id);
                ListenResult::Ignored(event)
            }
        }
    }

    pub fn query(&self, perm: PermanentID, query: &mut GameQuery, game: &Game)  {
        match self.base.class {
            AbilityClass::Static(ref a) => a.query(self.id, perm, query, game),
            _ => {}
        }
    }
}

pub struct LatentAbility {
    pub cost: Cost,
    pub class: AbilityClass,
    pub description: String,
}

#[derive(PartialEq, Eq)]
pub enum AbilitySpeed {
    Instant,
    Sorcery,
}

pub trait Effect{
    /// Called before the game is started. This is the one chance an effect has to modify itself.
    /// most effects don't need this, so a default implementation is provided
    fn setup(&mut self) {}
}

pub trait QueryModifier: Effect {
    fn query(&self, ability: AbilityID, perm: PermanentID, query: &mut GameQuery, game: &Game);
}

pub trait EventModifier: Effect {
    fn listen(&self, _ability: AbilityID, perm: PermanentID, event: GameEvent, _game: &Game) -> ListenResult;
}

pub trait OneShot: Effect {
    fn activate(&self, ability: AbilityID, perm: PermanentID, game: &mut Game);
}
