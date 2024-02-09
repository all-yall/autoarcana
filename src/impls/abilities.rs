use crate::engine::prelude::*;

#[derive(Clone)]
pub struct NullEffect {}
impl NullEffect {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }
}

impl Effect for NullEffect {}

impl EventModifier for NullEffect {
    fn listen(&self, _ability: AbilityID, perm: PermanentID, event: GameEvent, _game: &Game) -> ListenResult {
        (Some(event), vec![])
    }
}

impl QueryModifier for NullEffect {
    fn query(&self, _: AbilityID, _: PermanentID, _: &mut GameQuery, _: &Game) {
    }
}

impl OneShot for NullEffect {
    fn activate(&self, _: AbilityID, _: PermanentID, _: &mut Game) {
    }
}

#[derive(Clone)]
pub struct AddManaEffect {
    mana_type: ManaType
}
impl AddManaEffect {
    pub fn new(mana_type: ManaType) -> Box<Self> {
        Box::new(Self {mana_type})
    }
}

impl Effect for AddManaEffect {}
impl OneShot for AddManaEffect {
    fn activate(&self, _: AbilityID, perm: PermanentID, game: &mut Game) {
        let player_id = game.get(perm).owner;
        game.push_event(GameEvent::AddMana(player_id, self.mana_type, EventSource::Permanent(perm)))
    }
}

#[derive(Clone)]
pub struct MiraisMana {}
impl MiraisMana {
    pub fn new() -> Box<Self> { Box::new(Self{}) }
}

impl Effect for MiraisMana {}
impl EventModifier for MiraisMana {
    fn listen(&self, _: AbilityID, perm: PermanentID, event: GameEvent, game: &Game) -> ListenResult {
        let mut additional = vec![];
        match event {
            GameEvent::AddMana(player_id_recv_mana, mana_type, EventSource::Permanent(perm_source)) => {
                // TODO change this to use queries instead of reading game state.
                // This ability will not work correctly with static abilities that
                // change ownership and/or types
                let i_am_the_reciever = game.get(perm).owner == player_id_recv_mana;
                let mana_is_from_a_land = game.get(perm_source).type_line.is(CardType::Land);
                if i_am_the_reciever && mana_is_from_a_land {
                    additional.push(
                        GameEvent::AddMana(
                            player_id_recv_mana, 
                            mana_type, 
                            EventSource::Permanent(perm)));
                }
            }
            _ => {}
        }

        (Some(event), additional)
    }
}
