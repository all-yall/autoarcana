use dyn_clone::DynClone;

use crate::engine::prelude::*;

use super::util::id::ID;

#[derive(Clone)]
pub enum AbilityHolder {
    Card(CardID),
    Permanent(PermanentID),
}


#[derive(Clone)]
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

#[derive(Clone)]
pub struct Ability {
    pub holder: AbilityHolder,
    pub id: AbilityID,
    pub base: LatentAbility,
}

impl Ability {
    pub fn new(base: LatentAbility, id: AbilityID, holder: AbilityHolder) -> Self {
        Self {
            base,
            id,
            holder,
        }
    }

    pub fn listen(&mut self, event: GameEvent, game: &mut Game) -> ListenResult {
        match self.base.class {
            AbilityClass::Replacement(ref mut a) => a.listen(self.id, event, game),
            AbilityClass::Triggered(ref mut a) => a.listen(self.id, event, game),
            _ => (Some(event), vec![])
        }
    }

    pub fn query(&self, query: &mut GameQuery, game: &Game)  {
        match self.base.class {
            AbilityClass::Static(ref a) => a.query(self.id, query, game),
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct LatentAbility {
    pub class: AbilityClass,
    pub description: String,
}

pub trait QueryModifier: DynClone {
    fn query(&self, ability: AbilityID, query: &mut GameQuery, game: &Game);
}

pub trait EventModifier: DynClone {
    fn listen(&mut self, _ability: AbilityID, event: GameEvent, _game: &mut Game) -> ListenResult;
}

pub trait OneShot: DynClone {
    fn activate(&mut self, ability: AbilityID, game: &mut Game);
}

dyn_clone::clone_trait_object!(EventModifier);
dyn_clone::clone_trait_object!(QueryModifier);
dyn_clone::clone_trait_object!(OneShot);
