use crate::engine::prelude::*;

#[derive(Clone)]
pub struct NullEffect {}
impl NullEffect {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }
}

impl Continuous for NullEffect {
    fn done(&mut self) -> bool {
        false
    }

    fn listen(&mut self, _ability: AbilityID, event: GameEvent, _game: &mut Game) -> ListenResult {
        (Some(event), vec![])
    }
}

impl OneShot for NullEffect {
    fn activate(&mut self, _ability: AbilityID, _game: &mut Game) {
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

impl OneShot for AddManaEffect {
    fn activate(&mut self, ability_id: AbilityID, game: &mut Game) {
        let player_id = game.get_player_id_from_ability_id(ability_id);
        game.event(GameEvent::AddMana(player_id, self.mana_type, EventSource::Ability(ability_id)))
    }
}

#[derive(Clone)]
pub struct MiraisMana {}
impl MiraisMana {
    pub fn new() -> Box<Self> { Box::new(Self{}) }
}
impl Continuous for MiraisMana {
    fn listen(&mut self, ability_id: AbilityID, event: GameEvent, game: &mut Game) -> ListenResult {
        let mut additional = vec![];
        match event {
            GameEvent::AddMana(player_id_recv_mana, mana_type, EventSource::Ability(ability_id_source)) => {
                let i_am_recieving_mana = game.get_player_id_from_ability_id(ability_id) == player_id_recv_mana;
                let it_is_from_a_land = game.get_perm_from_ability_id(ability_id_source).types.iter().find(
                    |card_type| matches!(card_type, CardType::Land(_))
                ).is_some();
                if i_am_recieving_mana && it_is_from_a_land {
                    additional.push(
                        GameEvent::AddMana(
                            player_id_recv_mana, 
                            mana_type, 
                            EventSource::Ability(ability_id)));
                }
            }
            _ => {}
        }

        (Some(event), additional)
    }
}

#[derive(Clone)]
pub struct CastSpell {}

impl CastSpell {
    pub fn new() -> Box<dyn OneShot> {
        Box::new(Self{})
    }

    pub fn ability() -> LatentAbility {
        LatentAbility { 
            class: AbilityClass::Activated(Cost::empty(), Self::new()), 
            description: "Play spell".into() 
        }
    }
}

impl OneShot for CastSpell {
    fn activate(&mut self, ability_id: AbilityID, game: &mut Game) {
        let card_id = game.get_card_id_from_ability_id(ability_id);
        let card = game.take_card_from_card_id(card_id);
        let permanent = Permanent::from_card(&card.base, game.perm_ids.get_id(), card.owner);
        game.push_event(GameEvent::RegisterPermanent(permanent, card.base.perm_abilities.clone()));
    }
}
