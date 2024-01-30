use std::{collections::BTreeMap, process::exit};

use crate::{engine::prelude::*, client::GameStateSnapshot};
use crate::client::{Client, PlayerAction};

use super::{util::id::IDFactory};

use tokio::sync::broadcast::{channel, Sender as BroadcastSender};

#[derive(Clone, Copy)]
pub enum TurnStep {
    Untap,
    Upkeep,
    Draw,
    FirstMainPhase,
    Combat,
    SecondMainPhase,
    Discard,
    CleanUp,
}

pub const DEFAULT_TURN_STRUCTURE: [TurnStep; 8] = [
    TurnStep::Untap,
    TurnStep::Upkeep,
    TurnStep::Draw,
    TurnStep::FirstMainPhase,
    TurnStep::Combat,
    TurnStep::SecondMainPhase,
    TurnStep::Discard,
    TurnStep::CleanUp,
];

pub enum EventSource {
    Ability(AbilityID),
    GameRule(),
}

pub enum GameEvent {
    StartTurn(PlayerID),
    Step(TurnStep, PlayerID),
    UntapPerm(PermanentID),
    DrawCard(PlayerID),
    Lose(PlayerID),

    AddMana(PlayerID, ManaType, EventSource),

    PopulateAbilities,
    PopulatePermanentStaticAbilities(PermanentID),
    PopulatePermanentNonStaticAbilities(PermanentID),
    RegisterPermanentAbility(PermanentID, LatentAbility),
}

pub struct Game {
    pub players: Vec<Player>,
    pub active_player: PlayerID,
    pub exile_zone: Deck,
    pub battlefield: BTreeMap<PermanentID, Permanent>,
    pub abilities: BTreeMap<AbilityID, Ability>,
    pub turn_number: usize,
    pub game_over: bool,
    pub event_stack: Vec<GameEvent>,

    perm_ids: IDFactory<PermanentID>,
    ability_ids: IDFactory<AbilityID>,

    // send game updates to clients
    state_update_sender: BroadcastSender<GameStateSnapshot>,
    client: Client,
}


impl Game {
    pub fn new(decks: Vec<Vec<LatentCard>>) -> Self {

        let mut card_ids = IDFactory::new();
        let players : Vec<_> = decks.into_iter().zip(IDFactory::new())
            .map(|(deck, player_id)| {
                Player::new(
                deck.into_iter().map(|base| {
                    Card::new(base, card_ids.get_id(), player_id)
                }).collect(), player_id)
        }).collect();

        let active_player = players[0].id;

        let cap = 1;
        let (state_update_sender, state_update_receiver) = channel(cap);

        let client = Client::launch(active_player, state_update_receiver).expect("unable to launch client");

        Self {
            players,
            active_player,
            exile_zone: Deck::empty(),
            battlefield: BTreeMap::new(),
            turn_number: 0,
            game_over: false,
            event_stack: vec![],
            abilities: BTreeMap::new(),

            perm_ids: IDFactory::new(),
            ability_ids: IDFactory::new(),

            state_update_sender,
            client,
        }
    }

    /**
    * Push the given event onto the event stack and resolve it and all its
    * consequences. This will not resolve the entire event stack. The event
    * stack should hold the same value after as before.
    */
    pub fn event(&mut self, event: GameEvent) {
        let starting_size = self.event_stack.len();
        self.event_stack.push(event);
        while self.event_stack.len() > starting_size {
            let event = self.event_stack.pop().unwrap();
            self.default_event_handler(event);
        }
    }

    pub fn default_event_handler(&mut self, event: GameEvent) {
        use GameEvent::*;
        match event {
            StartTurn(player_id) => {
                for turn_step in DEFAULT_TURN_STRUCTURE.iter().rev() {
                    self.event_stack.push(Step(*turn_step, player_id));
                }
            },
            Step(step, player_id) => self.handle_step_event(step, player_id), 

            DrawCard(player_id) => {
                let player = self.get_player(player_id);
                if let Some(card) = player.deck.pop() {
                    player.hand.add(card);
                } else {
                    self.event_stack.push(Lose(player_id));
                }
            }

            UntapPerm(perm_id) => {
                self.battlefield.get_mut(&perm_id).unwrap().untap()
            }

            Lose(player_id) => {
                println!("player {:?} lost.", player_id);
                exit(0);
            }

            PopulateAbilities => {
                self.battlefield.values().for_each(|perm|
                    self.event_stack.push(PopulatePermanentStaticAbilities(perm.id))
                );
                self.battlefield.values().for_each(|perm|
                    self.event_stack.push(PopulatePermanentNonStaticAbilities(perm.id))
                );
            }

            PopulatePermanentStaticAbilities(perm) => {
                for latent_ability in self.battlefield.get(&perm).unwrap().intrinsic_abilities.iter() {
                    if AbilityClass::Static == latent_ability.class {
                        self.event_stack.push(RegisterPermanentAbility(perm, latent_ability.clone()))     
                    }
                }
            }

            PopulatePermanentNonStaticAbilities(perm) => {
                for latent_ability in self.battlefield.get(&perm).unwrap().intrinsic_abilities.iter() {
                    if AbilityClass::Static != latent_ability.class {
                        self.event_stack.push(RegisterPermanentAbility(perm, latent_ability.clone()))
                    }
                }
            }

            RegisterPermanentAbility(perm, latent_ability) => {
                let id = self.ability_ids.get_id();
                let holder = AbilityHolder::Permanent(perm);
                let ability = Ability {
                    id,
                    holder,
                    latent_ability,
                };

                self.abilities.insert(ability.id, ability);
            }

            AddMana(player_id, mana_type, _) => {
                self.get_player(player_id).mana_pool.push(mana_type);
            }
        }
    }

    pub fn handle_step_event(&mut self, turn_step: TurnStep, player_id: PlayerID) {
        use TurnStep::*;
        use GameEvent::*;
        match turn_step {
            Untap => {
                self.battlefield.values().for_each(|perm|
                    if player_id == perm.owner && perm.tapped {
                        self.event_stack.push(UntapPerm(perm.id))
                    }
                )
            }
            Upkeep => {},
            Draw => self.event_stack.push(DrawCard(player_id)),
            FirstMainPhase => self.main_phase(player_id),
            Combat => todo!(),
            SecondMainPhase => todo!(),
            Discard => todo!(),
            CleanUp => todo!(),
        }
    }

    pub fn main_phase(&mut self, player_id: PlayerID) {
        self.event(GameEvent::PopulateAbilities);
        loop {
            let mut player_actions = vec![PlayerAction::Pass];
            // the player may play each card in hand
            let playable_cards = self.get_player(player_id).hand.cards.iter().enumerate().map(|(idx, card)| (idx, card.base.name.clone())).collect::<Vec<_>>();

            // each card in is a potential player action
            for (idx, card_name) in playable_cards.into_iter() {
                player_actions.push(PlayerAction::CardPlay(idx, card_name));
            }

            match self.client.choose_options(player_actions) {
                PlayerAction::CardPlay(idx, _) => {
                    self.get_player(player_id).hand.cards.remove(idx);
                }
                PlayerAction::Pass => continue,
            }
        }
    }

    pub fn get_player(&mut self, id: PlayerID) -> &mut Player {
        self.players.iter_mut().find(|player| player.id == id).unwrap()
    }

    pub fn active_player(&mut self) -> &mut Player {
        self.get_player(self.active_player)
    }

    pub fn run(&mut self) {
        self.event(GameEvent::StartTurn(self.active_player));
    }

    pub fn next_player(&mut self) {
        let idx = self.players.iter()
            .enumerate()
            .find(|player| player.1.id == self.active_player)
            .unwrap().0;
        let new_idx = (idx + 1) % self.players.len();

        self.active_player = self.players[new_idx].id; 
    }



    pub fn get_player_id_from_ability_id(&mut self, ability_id: AbilityID) -> PlayerID {
        let ability = self.abilities.get(&ability_id).unwrap();
        match ability.holder {
            AbilityHolder::Permanent(perm) => self.get_player_id_from_perm_id(perm),
            _ => panic!("ahhh!")
        }
    }

    pub fn get_player_id_from_perm_id(&mut self, perm_id: PermanentID) -> PlayerID {
        let permanent = self.battlefield.get(&perm_id).unwrap();
         permanent.owner
    }

    pub fn get_ability_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Ability {
        self.abilities.get_mut(&ability_id).unwrap()
    }

    pub fn get_perm_id_from_ability_id(&mut self, ability_id: AbilityID) -> PermanentID {
        let ability = self.get_ability_from_ability_id(ability_id);
        match ability.holder {
            AbilityHolder::Permanent(perm) => perm,
            _ => panic!("ahhh!")
        }
    }

    pub fn get_perm_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Permanent {
        let perm_id = self.get_perm_id_from_ability_id(ability_id);
        self.battlefield.get_mut(&perm_id).unwrap()
    }

}
