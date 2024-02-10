use std::collections::{HashMap, hash_map::{Entry, OccupiedEntry}};

use super::prelude::*;

/// This checks all state based actions and queues up any spawed events.
///
/// NOTE: This function isn't part of the GameFacade struct directly so that it
/// doesn't have direct access to the private 'game' field.
/// This should ensure correct semantics when reading game state and prevent
/// writing entirely.
pub fn check_state_based_actions(facade: &GameFacade) -> Vec<GameEvent> {
    let mut ret = vec![];
    
    let perms : Vec<_> = facade.perms().map(|id|facade.observe_perm(id)).collect();

    // this is far from a complete list of state based actions, but it
    // covers the big ones.
    add_toughness_deaths(facade, &perms, &mut ret);
    add_no_loyalty_deaths(facade, &perms, &mut ret);
    add_legendary_conflicts(facade, &perms, &mut ret);
    add_cancel_popo_and_momo_counters(facade, &perms, &mut ret);
    add_sagas_with_no_lore_sacrifice(facade, &perms, &mut ret);
    add_player_loses_because_of_health(facade, &mut ret);


    ret
}

fn add_toughness_deaths(facade: &GameFacade, perms: &Vec<Permanent>, vec: &mut Vec<GameEvent>) {
    vec.extend(
        perms.iter().filter_map(|perm| {
            if should_die(facade, perm) {
                Some(GameEvent::Destroy(perm.id, EventSource::GameRule(GameRule::LethalDamage)))
            } else {
                None
            }
        }));
}

fn should_die(facade: &GameFacade, perm: &Permanent) -> bool {
    perm.type_line.is(CardType::Creature) && 
    perm.power_toughness.is_some_and(|pt| pt.toughness - perm.damage <= 0 )
}

fn add_no_loyalty_deaths(facade: &GameFacade, perms: &Vec<Permanent>, vec: &mut Vec<GameEvent>) {
    vec.extend(perms.iter().filter_map(|perm| {
        if perm.type_line.is(CardType::Planeswalker) && perm.counters.loyalty <= 0 {
            Some(GameEvent::Destroy(perm.id, EventSource::GameRule(GameRule::NoLoyalty)))
        } else {
            None
        }
    }))  
}


fn add_legendary_conflicts(facade: &GameFacade, perms: &Vec<Permanent>, vec : &mut Vec<GameEvent>) {
    let mut legend_map : HashMap<(PlayerID, &str), Vec<PermanentID>> = HashMap::new();

    for perm in perms.iter() {
        if perm.type_line.is(CardSuperType::Legendary) {
            let entry = legend_map.entry((perm.owner, &perm.name));
            match entry {
                Entry::Occupied(entry) => { entry.get().push(perm.id); }
                Entry::Vacant(entry) => { entry.insert(vec![perm.id]); }
            }
        }
    }
    vec.extend(
        legend_map.into_values().filter_map(|ids| 
            if ids.len() > 1 {
                Some(GameEvent::LegendConflict(ids))
            } else {
                None
            }
        )
    );
}

fn add_cancel_popo_and_momo_counters(facade: &GameFacade, perms: &Vec<Permanent>, vec : &mut Vec<GameEvent>) {
    for perm in perms.iter() {
        let popo = perm.counters.plus_one_plus_one;
        let momo = perm.counters.min_one_min_one;
        let min = popo.min(momo);
        if min > 0 {
            for counter_type in [CounterType::PlusOnePlusOne, CounterType::MinusOneMinusOne] {
                vec.push(GameEvent::RemoveCounters(
                    GameObjectID::Permanent(perm.id), 
                    counter_type,
                    min, 
                    EventSource::GameRule(GameRule::CancelOutCounters)
                ))
            }
        }
    }
}

fn add_sagas_with_no_lore_sacrifice(facade: &GameFacade, perms: &Vec<Permanent>, vec : &mut Vec<GameEvent>) {
    for perm in perms.iter() {
        // TODO get sagas to work.
        //if perm.type_line.is("Saga") && perm.counters.lore == perm.num_chapters {
        //    vec.push(GameEvent::Sacrifice(perm.id, GameRule::NoLore.into()))
        //}
    }
}

fn add_player_loses_because_of_health(facade: &GameFacade, vec : &mut Vec<GameEvent>) {
    vec.extend(
        facade.players().into_iter().filter_map( |player|
            if facade.player_life(player) <= 0 {
                Some(GameEvent::Lose(player, EventSource::GameRule(GameRule::NoHealth))) 
            } else {
                None
            }
        ));
}

