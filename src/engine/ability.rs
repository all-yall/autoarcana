use dyn_clone::DynClone;

use crate::engine::prelude::*;

use super::util::id::ID;

#[derive(Clone)]
pub enum AbilityHolder {
    Card(CardID),
    Permanent(PermanentID),
}


/*
* Activated
*   Have a cost and an effect
*
* Triggered
*   Listen and add event on event
*
* Static
*   Listen and replace/delete/modify events.
*   Change characteristics of other cards
*
* Mana
*   No Target and could add mana and not loyalty ability
*
* Loyalty
*   Had by planeswalkers
*/
#[derive(Clone,PartialEq, Eq)]
pub enum AbilityClass {
    Activated(Cost),
    Triggered,
    Static,
    Mana,
    Loyalty,
    Linked,
}

pub type AbilityID = ID<2>;

#[derive(Clone)]
pub struct Ability {
    pub holder: AbilityHolder,
    pub id: AbilityID,
    pub latent_ability: LatentAbility,
}

#[derive(Clone)]
pub struct LatentAbility {
    pub class: AbilityClass,
    pub description: String,
    pub effect: Effect,
}

#[derive(Clone)]
pub enum Effect {
    Continuous(Box<dyn Continuous>),
    OneShot(Box<dyn OneShot>),
}


pub trait Continuous: DynClone {
    fn done(&mut self) -> bool { false }
    fn listen(&mut self, ability: AbilityID, event: GameEvent, game: &mut Game) -> Vec<GameEvent>;
}
dyn_clone::clone_trait_object!(Continuous);

pub trait OneShot: DynClone {
    fn activate(&mut self, ability: AbilityID, game: &mut Game);
}
dyn_clone::clone_trait_object!(OneShot);
