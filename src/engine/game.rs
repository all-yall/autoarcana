use std::{collections::{BTreeMap, HashSet}, process::exit};

use crate::{engine::prelude::*, client::GameStateSnapshot};
use crate::client::{Client, PlayerAction};
use log::warn;

use super::util::id::{
    ID,
    IDFactory,
    IDMapper,
};

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
    Player(PlayerID),
    GameRule,
}

#[derive(Eq, PartialEq)]
pub enum Zone {
    Hand,
    Exile,
    Battlefield,
    Graveyard,
}

/// This represents any game modification event
/// that is relevant to other abilities, and could
/// potentially be modified by them.
pub enum GameEvent {
    StartTurn(PlayerID),
    Step(TurnStep, PlayerID),
    UntapPerm(PermanentID),
    DrawCard(PlayerID),
    Lose(PlayerID),

    PlaySpell(AbilityID),
    ActivateAbility(AbilityID),

    AddMana(PlayerID, ManaType, EventSource),

    RegisterPermanent(Permanent, Vec<LatentAbility>),
    EnterTheBattleField(PermanentID),
}


/// This represents any read of game state that could
/// be modified by other abilities. Mostly continuous
/// effects are what should be considered.
pub enum GameQuery {
    PermanentAbilities(PermanentID, Vec<AbilityID>),
    CardAbilities(CardID, Vec<AbilityID>),
}


/// The Game object contains all game information
/// and additionally acts as a store for all game
/// objects so that they are easily found and rusts
/// borrow checker doesn't get too mad.
pub struct Game {
    pub players: Vec<Player>,
    pub active_player: PlayerID,
    pub exile_zone: Deck,
    pub battlefield: BTreeMap<PermanentID, Permanent>,
    pub abilities: BTreeMap<AbilityID, Ability>,
    pub turn_number: usize,
    pub game_over: bool,
    pub event_stack: Vec<GameEvent>,

    pub perm_ids: IDFactory<PermanentID>,
    pub ability_ids: IDFactory<AbilityID>,

    // send game updates to clients
    state_update_sender: BroadcastSender<GameStateSnapshot>,
    client: Client,
}

/// This represents the ordering that abilities
/// should be applied. This includes the Layer system
/// and the ordering of replacement effects before trigger
/// effects.
pub struct AbilityOrdering {
    static_order: Vec<AbilityID>,
    replacement_order: Vec<AbilityID>,
    trigger_order: Vec<AbilityID>,
    fresh: bool,
}

impl AbilityOrdering {
    fn new() -> Self {
        Self {
            static_order: vec![],
            replacement_order: vec![],
            trigger_order: vec![],
            fresh: true
        }
    }

    fn build_from(game: &mut Game) -> Self {
        let mut order = Self::new();
        let mut seen_ids = HashSet::new();
        let mut done = true;

        while !done {
            let abilities = game.all_abilities(&mut order);

            for ability_id in abilities.iter() {
                let seen = !seen_ids.insert(*ability_id);
                if seen {
                    continue
                }

                let ability = game.get(*ability_id);
                match ability.base.class {
                    AbilityClass::Replacement(_)  => order.replacement_order.push(*ability_id),
                    AbilityClass::Triggered(_)  => order.trigger_order.push(*ability_id),
                    AbilityClass::Static(_) => {
                        order.static_order.push(*ability_id);
                        break;
                    }
                    _ => {}
                }
            }

            done = true;
        }

        order
    }

    /// Puts query through the ability ordering to apply all 
    /// continuous effects 
    pub fn query(&mut self, game: &Game, query: &mut GameQuery) {
        self.check_fresh();

        for ability_id in self.static_order.iter() {
            let ability = game.abilities.get(ability_id).unwrap();
            ability.query(query, game);
        }
    }

    /// Puts event through the ability ordering to apply all trigger
    /// and replacement effects
    pub fn listen(&mut self, mut event: GameEvent, game: &mut Game) -> ListenResult {
        self.check_fresh();

        let mut new_events = Vec::new();
        for ability_id in self.replacement_order.iter().chain(self.trigger_order.iter()) {
            let mut ability = game.abilities.remove(ability_id).unwrap();
            let result = ability.listen(event, game); 
            game.abilities.insert(*ability_id, ability);

            new_events.extend(result.1);
            if let Some(ev) = result.0 {
                event = ev;
            } else {
                self.fresh = false;
                return (None, new_events);
            }
        }
        self.fresh = self.fresh && new_events.is_empty();

        (Some(event), new_events)
    }

    /// A sanity checking function; In the event that the abilityOrder 
    /// generates new events, then the current abilityOrdering should
    /// no longer be trusted, so a warning is printed
    fn check_fresh(&self) {
        if !self.fresh {
            warn!("Using non-fresh ability ordering, could produce incorrect results.");
        }
    }

    fn add_from(&mut self, abilities: Vec<AbilityID>, game: &mut Game) -> bool {
        self.check_fresh();

        // TODO sort by layer here


        return false;
    }
}

impl Game {
    pub fn new(decks: Vec<Vec<LatentCard>>) -> Self {

        let mut card_ids = IDFactory::new();
        let mut player_ids = IDFactory::new().peekable();
        let active_player = *player_ids.peek().unwrap();

        let cap = 1;
        let (state_update_sender, state_update_receiver) = channel(cap);

        let client = Client::launch(active_player, state_update_receiver).expect("unable to launch client");

        let mut game = Self {
            active_player,
            players: Vec::new(),
            exile_zone: Deck::empty(),
            turn_number: 0,
            game_over: false,
            event_stack: vec![],
            battlefield: BTreeMap::new(),
            abilities: BTreeMap::new(),

            perm_ids: IDFactory::new(),
            ability_ids: IDFactory::new(),

            state_update_sender,
            client,
        };

        let players : Vec<_> = decks
            .into_iter()
            .zip(player_ids)
            .map(|(deck, player_id)| {
                Player::new(
                    deck
                        .into_iter()
                        .map(|base| {
                            let card_id = card_ids.get_id();
                            let card_abilities = base.card_abilities
                                .iter()
                                .map(|ability| game.add_ability(ability.clone(), AbilityHolder::Card(card_id)))
                                .collect();
                        Card::new(base, card_id, card_abilities, player_id)
                }).collect(), player_id)
        }).collect();

        game.players = players;

        game
    }

    /// Queues an event to be processed by the game. Keep in mind that 
    /// events are processed in a Last-in First-out order.
    pub fn push_event(&mut self, event: GameEvent) {
        self.event_stack.push(event);
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

            RegisterPermanent(perm, abilities) => {
                let id = perm.id;
                self.battlefield.insert(id, perm);
                let abilities = abilities.into_iter().map(|ability| {
                    self.add_ability(ability, AbilityHolder::Permanent(id))
                }).collect();
                self.get_perm_from_perm_id(id).abilities = abilities;
                self.push_event(EnterTheBattleField(id));
            }

            AddMana(player_id, mana_type, _) => {
                self.get_player(player_id).mana_pool.push(mana_type);
            }

            PlaySpell(ability_id) => {
                let mut ability = self.get(ability_id).clone();
                match ability.base.class {
                    AbilityClass::Activated(_, ref mut ability) => ability.activate(ability_id, self),
                    _ => panic!("Expected activated ability for PlaySpell event")
                }
            }

            ActivateAbility(ability_id) => {
                let mut ability = self.get(ability_id).clone();
                match ability.base.class {
                    AbilityClass::Activated(_, ref mut ability) => ability.activate(ability_id, self),
                    _ => panic!("Expected activated ability for ActivateAbility event")
                }
            }

            EnterTheBattleField(_) => {}
        }
    }

    pub fn build_ability_order(&mut self) -> AbilityOrdering {
        let mut ordering = AbilityOrdering::new();

        let mut unfinished = false;
        while unfinished {
            let abilities = self.all_abilities(&mut ordering);
            unfinished = ordering.add_from(abilities, self);
        }

        return ordering;
    }

    pub fn all_abilities(&self, order: &mut AbilityOrdering) -> Vec<AbilityID> {
        let mut perm = self.all_perm_abilities(order);
        let card = self.all_card_abilities(order);
        perm.extend(card);
        perm
    }

    pub fn all_perm_abilities(&self, order: &mut AbilityOrdering) -> Vec<AbilityID> {
        self.battlefield
            .keys()
            .flat_map(|perm_id| self.perm_abilities(*perm_id, order).into_iter())
            .collect()
    }

    pub fn all_card_abilities(&self, order: &mut AbilityOrdering) -> Vec<AbilityID> {
        self.players
            .iter()
            .flat_map(|player| player.hand.cards.iter())
            .flat_map(|card| self.card_abilities(card.id, order).into_iter())
            .collect()
    }

    pub fn card_abilities(&self, card_id: CardID, order: &mut AbilityOrdering) -> Vec<AbilityID> {
        let mut query = GameQuery::CardAbilities(card_id, vec![]);
        self.query(&mut query, order);
        if let GameQuery::CardAbilities(_, abilities) = query { return abilities }
        panic!("query type was changed!");
    }

    pub fn perm_abilities(&self, perm_id: PermanentID, order: &mut AbilityOrdering) -> Vec<AbilityID> {
        let mut query = GameQuery::PermanentAbilities(perm_id, vec![]);
        self.query(&mut query, order);
        if let GameQuery::PermanentAbilities(_, abilities) = query { return abilities }
        panic!("query type was changed!");
    }


    pub fn query(&self, query: &mut GameQuery, ability_ordering: &mut AbilityOrdering) {
        match query {
            GameQuery::PermanentAbilities(perm_id, ref mut abilities) => {
                let perm_abilities = self.battlefield
                    .get(perm_id)
                    .unwrap()
                    .abilities
                    .iter();
                abilities.extend(perm_abilities);
            }

            GameQuery::CardAbilities(card_id, ref mut abilities) => {
                let card_abilities = self.players.iter()
                    .flat_map(|player| player.hand.cards.iter())
                    .find(|card| card.id == *card_id)
                    .unwrap().abilities.iter();

                abilities.extend(card_abilities);
            }
        }

        ability_ordering.query(self, query);
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
        let mut ability_order = self.build_ability_order();

        loop {
            let abilities = self.all_abilities(&mut ability_order);

            let mut player_actions = vec![PlayerAction::Pass];

            // collect all abilities, then sort into type and if the player controls the ability
            for ability_id in abilities.into_iter() {
                let ability_holder = self.get(ability_id).holder.clone();
                match ability_holder {
                    AbilityHolder::Card(card_id) =>  {
                        let player = self.get_player_id_from_card_id(card_id);
                        if player != player_id {continue}
                        player_actions.push(
                            PlayerAction::CardPlay(ability_id, self.get(ability_id).base.description.clone())
                        );
                    }
                    AbilityHolder::Permanent(perm_id) =>  {
                        let player = self.get_player_id_from_perm_id(perm_id);
                        if player != player_id {continue}
                        player_actions.push(
                            PlayerAction::ActivateAbility(ability_id, self.get(ability_id).base.description.clone())
                        );
                    }
                }
            }

            match self.client.choose_options(player_actions) {
                PlayerAction::CardPlay(idx, _) => todo!(),
                PlayerAction::ActivateAbility(ability_id, _) => todo!(),
                PlayerAction::Pass => continue,
            }
        }
    }

    fn add_ability(&mut self, ability: LatentAbility, holder: AbilityHolder) -> AbilityID {
        let id = self.ability_ids.get_id();
        let ability = Ability::new(ability, id, holder);
        self.abilities.insert(id, ability);
        id
    }
    
    fn remove_ability(&mut self, id: AbilityID) {
        self.abilities.remove(&id);
    }

    pub fn get_player(&mut self, id: PlayerID) -> &mut Player {
        self.players.iter_mut().find(|player| player.id == id).unwrap()
    }

    pub fn active_player(&mut self) -> &mut Player {
        self.get_player(self.active_player)
    }

    pub fn run(&mut self) {
        self.push_event(GameEvent::StartTurn(self.active_player));
        loop {
            let event = self.event_stack.pop().unwrap();
            self.default_event_handler(event);
        }
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
        self.get_perm_from_perm_id(perm_id).owner
    }

    pub fn get_player_id_from_card_id(&mut self, card_id: CardID) -> PlayerID {
        self.get(card_id).owner
    }

    pub fn get_perm_from_perm_id(&mut self, perm_id: PermanentID) -> &mut Permanent {
        self.battlefield.get_mut(&perm_id).unwrap()
    }

    pub fn get_perm_id_from_ability_id(&mut self, ability_id: AbilityID) -> PermanentID {
        let ability = self.get(ability_id);
        match ability.holder {
            AbilityHolder::Permanent(perm) => perm,
            _ => panic!("asserted that ability was held by permanent when it was not.")
        }
    }

    pub fn get_perm_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Permanent {
        let perm_id = self.get_perm_id_from_ability_id(ability_id);
        self.battlefield.get_mut(&perm_id).unwrap()
    }

    pub fn get_card_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Card {
        let card_id = self.get_card_id_from_ability_id(ability_id);
        self.get_mut(card_id)
    }

    pub fn get_card_id_from_ability_id(&mut self, ability_id: AbilityID) -> CardID {
        let ability = self.get(ability_id);
        match ability.holder {
             AbilityHolder::Card(card) => card,
            _ => panic!("asserted that ability was held by card when it was not.")
        }
    }

    pub fn take_card_from_card_id(&mut self, card_id: CardID) -> Card {
        for player in self.players.iter_mut() {
            let position = player.hand.cards.iter().position(|card| card.id == card_id);
            if let Some(pos) = position { 
                return player.hand.cards.remove(pos);
            }
        }
        panic!("card id {:?} is not found in any hand", card_id);
    }
}

impl IDMapper<Ability> for Game {
    fn get(&self, id: ID<Ability>) -> &Ability { self.abilities.get(&id).unwrap() }
    fn get_mut(&mut self, id: ID<Ability>) -> &mut Ability { self.abilities.get_mut(&id).unwrap() }
}

impl IDMapper<Player> for Game {
    fn get(&self, id: ID<Player>) -> &Player { self.players.iter().find(|player| player.id == id).unwrap() }
    fn get_mut(&mut self, id: ID<Player>) -> &mut Player { self.players.iter_mut().find(|player| player.id == id).unwrap() }
}

impl IDMapper<Card> for Game {
    fn get(&self, id: ID<Card>) -> &Card { 
        self.players
            .iter()
            .find_map(|player| 
                player.hand.cards
                    .iter()
                    .find(|card| card.id == id)
            ).unwrap()
    }

    fn get_mut(&mut self, id: ID<Card>) -> &mut Card { 
        self.players
            .iter_mut()
            .find_map(|player| 
                player.hand.cards
                    .iter_mut()
                    .find(|card| card.id == id)
            ).unwrap()
    }
}

impl IDMapper<Permanent> for Game {
    fn get(&self, id: ID<Permanent>) -> &Permanent { self.battlefield.get(&id).unwrap() }
    fn get_mut(&mut self, id: ID<Permanent>) -> &mut Permanent { self.battlefield.get_mut(&id).unwrap() }
}
