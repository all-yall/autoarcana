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
/// if the event is consumed, the first field should be None.
/// if additional events are fired in response to the given one, then
/// those should be put in the vector in the reverse order of evaluation.
pub type ListenResult = (Option<GameEvent>, Vec<GameEvent>);

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
            _ => (Some(event), vec![])
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
