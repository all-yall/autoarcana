use std::{collections::HashSet, cell::Cell};
use log::warn;

use super::prelude::*;

/// This represents the ordering that abilities
/// should be applied. This includes the Layer system
/// and the ordering of replacement effects before trigger
/// effects.
pub struct AbilityOrdering {
    static_order: Vec<AssignedAbility>,
    replacement_order: Vec<AssignedAbility>,
    trigger_order: Vec<AssignedAbility>,

    // Is only for logging purposes.
    fresh: Cell<bool>,
}

impl AbilityOrdering {
    fn new() -> Self {
        Self {
            static_order: vec![],
            replacement_order: vec![],
            trigger_order: vec![],
            fresh: Cell::new(true),
        }
    }

    pub fn build_from(game: &Game) -> Self {
        let mut order = Self::new();
        let mut seen_abilities = HashSet::new();
        let mut done = true;

        while !done {
            done = true;
            let abilities = game.all_abilities(&mut order);

            // TODO sort by layer here

            for as_ability in abilities.iter() {
                let seen = !seen_abilities.insert(*as_ability);
                if seen { continue }

                let ability = game.get(as_ability.ability);
                match ability.base.class {
                    AbilityClass::Replacement(_)  => order.replacement_order.push(*as_ability),
                    AbilityClass::Triggered(_)  => order.trigger_order.push(*as_ability),
                    AbilityClass::Static(_) => {
                        order.static_order.push(*as_ability);
                        done = false;
                        break;
                    }
                    _ => {}
                }
            }
        }

        order
    }

    /// Puts query through the ability ordering to apply all 
    /// continuous effects 
    pub fn query(&self, game: &Game, query: &mut GameQuery) {
        self.check_fresh();

        for as_ability in self.static_order.iter() {
            let ability = game.abilities.get(&as_ability.ability).unwrap();
            ability.query(as_ability.perm, query, game);
        }
    }

    /// Puts event through the ability ordering to apply all trigger
    /// and replacement effects
    pub fn listen(&self, mut event: GameEvent, game: &Game) -> ListenResult {
        self.check_fresh();

        let mut new_events = Vec::new();
        for as_ability in self.replacement_order.iter().chain(self.trigger_order.iter()) {
            let ability = game.get(as_ability.ability);
            let result = ability.listen(as_ability.perm, event, game); 
            match result {
                ListenResult::Replaced(_) => return result,
                ListenResult::Triggered(ev, triggered) => {
                    event = ev;
                    new_events.extend(triggered);
                }
                ListenResult::Ignored(ev) => {
                    event = ev;
                }
            }
        }

        if new_events.is_empty() {
            ListenResult::Ignored(event)
        } else {
            // if an Some(event) is returned, then it will be applied to the game
            // state, and this ordering may no longer be fresh.
            self.fresh.set(false);

            ListenResult::Triggered(event, new_events)
        }

    }

    /// A sanity checking function; In the event that the abilityOrder 
    /// generates new events, then the current abilityOrdering should
    /// no longer be trusted, so a warning is printed
    fn check_fresh(&self) {
        if !self.fresh.get() {
            warn!("Using non-fresh ability ordering, could produce incorrect results.");
        }
    }
}
