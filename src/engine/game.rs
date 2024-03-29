use std::{
    collections::BTreeMap, 
    process::exit,
    fmt::Debug,
};

use super::{prelude::*, state_based_actions::check_state_based_actions};

use crate::client::{Client, PlayerAction, GameStateSnapshot};

use super::util::id::{
    ID,
    IDFactory,
    IDMapper,
};

use log::{info, warn, error};
use tokio::sync::broadcast::{channel, Sender as BroadcastSender};

#[derive(Debug)]
pub enum FailureReason {
    CouldntPayCost,
    IllegalAbilityClass
}

/// The Game object contains all game information
/// and additionally acts as a store for all game
/// objects so that they are easily found and rusts
/// borrow checker doesn't get too mad.
pub struct Game {
    pub player_with_last_action: PlayerID,
    pub active_player: PlayerID,
    pub turn_step: TurnStep,

    pub players: Vec<Player>,
    pub turn_number: usize,
    pub game_over: bool,
    pub event_stack: Vec<GameEvent>,
    pub game_stack: Vec<Object>,
    pub cards: CardStore,

    pub battlefield: BTreeMap<PermanentID, Permanent>,
    pub abilities: BTreeMap<AbilityID, Ability>,
    pub card_plays: BTreeMap<CardPlayID, CardPlay>,


    pub perm_ids: IDFactory<PermanentID>,
    pub ability_ids: IDFactory<AbilityID>,
    pub card_play_ids: IDFactory<CardPlayID>,

    // send game updates to clients
    state_update_sender: BroadcastSender<GameStateSnapshot>,
    client: Client,
}


impl Game {
    pub fn new(decks: Vec<Vec<LatentCard>>) -> Self {

        let card_ids = IDFactory::new();
        let player_ids = IDFactory::new()
            .take(decks.len())
            .collect::<Vec<_>>();
        let active_player = *player_ids.first().expect("Need at least on player in the game");

        let cap = 1;
        let (state_update_sender, state_update_receiver) = channel(cap);

        let client = Client::launch(active_player, state_update_receiver).expect("unable to launch client");

        let mut game = Self {
            active_player,
            player_with_last_action: active_player,
            turn_step: TurnStep::Upkeep,

            players: Vec::new(),
            turn_number: 0,
            game_over: false,
            event_stack: vec![],


            game_stack: Vec::new(),
            card_plays: BTreeMap::new(),
            battlefield: BTreeMap::new(),
            abilities: BTreeMap::new(),
            cards: CardStore::new(&player_ids),

            perm_ids: IDFactory::new(),
            ability_ids: IDFactory::new(),
            card_play_ids: IDFactory::new(),

            state_update_sender,
            client,
        };

        let players : Vec<_> = decks
            .into_iter()
            .zip(player_ids)
            .map(|(deck, player_id)| {
                deck.into_iter().for_each(|base| {
                    let card_id = card_ids.get_id();
                    let  LatentCard {attributes, perm_abilities, card_plays} = base;

                    let perm_ability_ids = perm_abilities
                        .into_iter()
                        .map(|ability| game.add_ability(ability))
                        .collect();

                    let card_play_ids = card_plays
                        .into_iter()
                        .map(|card_play| game.add_card_play(card_play))
                        .collect();

                    let card = Card::new(
                        attributes, 
                        card_id, 
                        perm_ability_ids, 
                        card_play_ids,
                        player_id
                    );

                    game.cards.put_card(card, Zone::Deck(player_id));
                });
                Player::new(player_id)
        }).collect();

        game.players = players;

        game
    }

    /// Queues an event to be processed by the game. Keep in mind that 
    /// events are processed in a Last-in First-out order.
    pub fn push_event(&mut self, event: GameEvent) {
        info!("   Event pushed {:?}", event);
        self.event_stack.push(event);
    }

    pub fn push_events<I: IntoIterator<Item = GameEvent>>(&mut self, events: I) {
        for event in events.into_iter() {
            self.push_event(event)
        }
    }

    pub fn default_event_handler(&mut self, event: GameEvent) {
        use GameEvent::*;
        match event {
            Step(step) => self.handle_step_event(step), 

            DrawCard(player_id) => {
                let drawn_card = self.cards.draw(player_id);
                // Couldn't draw a card, lose the game.
                if drawn_card.is_none() { 
                    self.push_event(Lose(player_id, GameRule::CouldntDraw.into()));
                }
            }

            UntapPerm(perm_id) => {
                self.battlefield.get_mut(&perm_id).unwrap().untap()
            }

            Lose(player_id, _reason) => {
                println!("player {:?} lost.", player_id);
                exit(0);
            }

            RegisterPermanent(perm) => {
                let id = perm.id;
                if !perm.is_token {
                    if let Some(card) = perm.card {
                        self.cards.move_to_zone(card, Zone::Battlefield);
                    } else {
                        warn!("Non-token permanent has no card {:?}", perm);
                    }
                }
                self.battlefield.insert(id, perm);
                self.push_event(EnterTheBattleField(id));
            }

            AddMana(player_id, mana_type, _) => {
                self.get_mut(player_id).mana_pool.push(mana_type);
            }

            PlaySpell(as_card_play) => {
                let card_play = self.get(as_card_play.card_play);
                let object = card_play.spawn.spawn(as_card_play.card, self);

                if let ObjectResolve::CreateLand(perm) = object.resolve {
                    // Lands skip the stack and enter the battlefield directly
                    self.push_event(RegisterPermanent(perm));
                } else {
                    self.game_stack.push(object);
                    self.cards.move_to_zone(as_card_play.card, Zone::Stack);
                }
            }

            ActivateAbility(as_ability) => {
                // TODO: this is not the best way to do this; removing the ability from
                // the map the get around the borrow checker. 
                // a better solution in which abilities are not stored in the game object,
                // or the abilities not being able to modify the game directly
                // TODO: activated abilities should go on the stack, not be run directly
                let ability = self.abilities.remove(&as_ability.ability).expect("Invalid ability ID should not be possible");
                match ability.base.class {
                    AbilityClass::Activated(_, ref ability) => ability.activate(as_ability.ability, as_ability.perm, self),
                    _ => {
                        error!("Expected activated ability for ActivateAbility event {:?}", as_ability); 
                        panic!();
                    }
                }
                self.abilities.insert(as_ability.ability, ability);
            }

            EnterTheBattleField(_) => {}
            LegendConflict(_) => todo!(),
            Destroy(_, _) => todo!(),
            Sacrifice(_, _) => todo!(),
            AddCounters(_, _, _, _) => todo!(),
            RemoveCounters(_, _, _, _) => todo!(),


            GivePriority(player) => {
                self.player_with_last_action = player;
                self.try_give_priority(player);
            }

            PassPriority(player) => {
                if player == self.player_with_last_action {
                    self.push_event(TryResolveStackObject)
                } else {
                    let ability_order = self.build_ability_order();
                    self.priority(player, ability_order);
                }
            },

            TryResolveStackObject =>  {
                if let Some(object) = self.game_stack.pop() {
                    self.push_event(GivePriority(self.active_player));
                    match object.resolve {
                        ObjectResolve::CreatePerm(perm) => {
                            self.push_event(RegisterPermanent(perm));

                            if let Some(card) = object.card {
                                self.cards.move_to_zone(card, Zone::Battlefield);
                            }
                        }
                        ObjectResolve::AbilityActivate(_) => todo!(),
                        ObjectResolve::CreateLand(perm) => error!("Land was on the stack, this shouldn't happen. {:?}", perm),
                    }
                }
            }

            NextStep => {
                let current_step = DEFAULT_TURN_STRUCTURE
                    .iter()
                    .enumerate()
                    .find(|turn| self.turn_step == *turn.1)
                    .expect("Current Turn should be in turn structure.").0;
                let next_step = current_step + 1;

                let next_step_event = if next_step == DEFAULT_TURN_STRUCTURE.len() {
                    StartTurn(self.next_player(self.active_player))
                } else {
                    Step(DEFAULT_TURN_STRUCTURE[next_step])
                };

                if self.event_stack.len() != 0 {
                    warn!("When changing turns, event stack should be empty. Instead, it holds {} events", self.event_stack.len())
                }
                self.push_event(next_step_event)
            },

            StartTurn(player) => {
                self.active_player = player;
                self.push_event(Step(DEFAULT_TURN_STRUCTURE[0]))
            },

            TapPerm(perm) => {
                let perm = self.get_mut(perm);
                if perm.tapped {
                    warn!("Tried to tap already tapped permanent!")
                }
                perm.tapped = true;
            }

            PayMana(player, cost) => {
                let player = self.get_mut(player);

                for mana in cost.mana.iter() {
                    let idx = player.mana_pool.iter().position(|m| m == mana);
                    match idx {
                        Some(idx) => {player.mana_pool.remove(idx);}
                        None => {error!("Tried to pay {:?} out of {:?} mana pool, but it wasn't there", mana, player.id);}
                    }
                }

                for _ in 0..cost.generic_mana {
                    if let None = player.mana_pool.pop() {
                        error!("Tried to pay generic mana out of {:?} mana pool, but it wasn't there", player.id)
                    }
                }
            },
        }
    }

    pub fn build_ability_order(&mut self) -> AbilityOrdering {
        AbilityOrdering::build_from(self)
    }

    pub fn all_abilities(&self, order: &AbilityOrdering) -> Vec<AssignedAbility> {
        self.battlefield
            .keys()
            .flat_map(|perm_id| self.perm_abilities(*perm_id, order).into_iter())
            .collect()
    }

    pub fn all_card_plays(&self, order: &AbilityOrdering) -> Vec<AssignedCardPlay> {
        self.players
            .iter()
            .flat_map(|player| self.cards.hand(player.id).into_iter())
            .flat_map(|card| self.card_plays(card.id, order).into_iter())
            .collect()
    }

    pub fn card_plays(&self, card_id: CardID, order: &AbilityOrdering) -> Vec<AssignedCardPlay> {
        let card_plays = self.query(
            CardPlaysQuery::new(card_id), 
            order);

        return card_plays.card_plays
            .into_iter()
            .map(|card_play| AssignedCardPlay::new(card_id, card_play))
            .collect()
    }

    pub fn perm_abilities(&self, perm_id: PermanentID, order: &AbilityOrdering) -> Vec<AssignedAbility> {
        let perm_abilities = self.query(
             PermAbilityQuery::new(perm_id), 
             order);

        return perm_abilities.abilities
            .into_iter()
            .map(|ability_id| AssignedAbility::new(perm_id, ability_id))
            .collect() 
    }


    pub fn query<Q: GameQueryVariant>(&self, query: Q, order: &AbilityOrdering) -> Q 
    where <Q as TryFrom<GameQuery>>::Error : Debug {
        let mut query = query.into();
        self.raw_query(&mut query, order);
        Q::try_from(query).expect("Game query variant changed.")
    }

    fn raw_query(&self, query: &mut GameQuery, ability_order: &AbilityOrdering) {
        match query {
            GameQuery::PermAbilities(ref mut query) => {
                let base_abilities = self
                    .get(query.id)
                    .abilities
                    .iter();
                query.abilities.extend(base_abilities);
            }

            GameQuery::CardPlays(ref mut query) => {
                query.card_plays.extend(self.cards.get_card(query.id).card_plays.iter());
            }

            GameQuery::ObservePerm(ref mut query) => {
                // TODO account for counters here
            }
        }

        ability_order.query(self, query);
    }

    pub fn handle_step_event(&mut self, turn_step: TurnStep) {
        use TurnStep::*;
        use GameEvent::*;

        self.turn_step = turn_step;
        self.push_event(NextStep);

        // Whether or not this turn gives priority
        match turn_step {
            Untap => {

               let events: Vec<_> = self.battlefield.values().filter_map(|perm|
                    (self.active_player == perm.owner && perm.tapped)
                        .then_some(UntapPerm(perm.id))
                ).collect();

                self.push_events(events)
            }

            Upkeep => {
            },

            Draw => {
                self.push_event(GivePriority(self.active_player));
                self.push_event(DrawCard(self.active_player));
            }

            FirstMainPhase => {
                self.push_event(GivePriority(self.active_player));
            }

            Combat =>  {
                self.push_event(GivePriority(self.active_player));
            }

            SecondMainPhase => {
                self.push_event(GivePriority(self.active_player));
            }

            Discard => {
                // TODO implement this
            }

            CleanUp => {
                //todo use events
                for perm in self.battlefield.values_mut() {
                    if let Some(ref mut pt) = perm.power_toughness {
                        pt.toughness = pt.base_toughness;
                        pt.power = pt.base_power;
                    }
                }
            }
        }
    }

    pub fn try_give_priority(&mut self, player: PlayerID) {
        // First check state based actions.
        let facade = GameFacade::new(self);
        let before_give_priority_events = check_state_based_actions(&facade);
        if before_give_priority_events.len() > 0 {
            self.push_event(GameEvent::GivePriority(player));
            self.push_events(before_give_priority_events);
            return;
        }

        // if there were none, then give the player priority
        self.priority(player, facade.into());
    }

    pub fn priority(&mut self, player_id: PlayerID, ability_order: AbilityOrdering) {
        loop {
            let action = self.get_player_action(player_id, &ability_order);

            match self.try_do_player_action(player_id, action) {
                Ok(()) => return,
                Err(reason) => warn!("Player chose invalid option; {:?}", reason),
            }
        }
    }

    fn try_do_player_action(&mut self, player_id: PlayerID, action: PlayerAction) -> Result<(), FailureReason> {

        match action {
            PlayerAction::Pass => {
                self.push_event(GameEvent::PassPriority(self.next_player(player_id)));
            }


            PlayerAction::CardPlay(as_card_play, _) => {
                // TODO should be a query
                let cost = self.get(as_card_play.card_play).spawn.cost(as_card_play.card, self);
                let events = self.try_pay_cost(player_id, cost)?;

                self.push_event(GameEvent::GivePriority(player_id));
                self.push_event(GameEvent::PlaySpell(as_card_play));
                self.push_events(events);
            }


            PlayerAction::ActivateAbility(as_ability, _) => {
                // TODO should be a query
                let cost = match self.get(as_ability.ability).base.class {
                    AbilityClass::Activated(ref cost, _) => cost.clone(),
                    _ => Err(FailureReason::IllegalAbilityClass)?,
                };

                let events = self.try_pay_ability_cost(player_id, as_ability.perm, cost)?;

                self.push_event(GameEvent::GivePriority(player_id));
                self.push_event(GameEvent::ActivateAbility(as_ability));
                self.push_events(events);
            }
        };

            
        Ok(())
    }

    fn try_pay_ability_cost(&mut self, player: PlayerID, perm: PermanentID, cost: AbilityCost) -> Result<Vec<GameEvent>, FailureReason> {
        let mut ret = self.try_pay_cost(player, cost.cost)?;
        let perm = self.get(perm);
        if cost.tap {
            if perm.tapped || (perm.type_line.is(CardType::Creature) && perm.summoning_sickness) {
                Err(FailureReason::CouldntPayCost)?;
            } else {
                ret.push(GameEvent::TapPerm(perm.id));
            }
        }
        Ok(ret)
    }

    fn try_pay_cost(&mut self, player: PlayerID, cost: Cost) -> Result<Vec<GameEvent>, FailureReason> {
        self.try_pay_mana_cost(player, cost.mana_cost)
    }

    fn try_pay_mana_cost(&mut self, player: PlayerID, cost: ManaCost) -> Result<Vec<GameEvent>, FailureReason> {
        let mut mana_pool = self.get(player).mana_pool.clone();

        for mana in cost.mana.iter() {
            let pos = mana_pool.iter().position(|m| m == mana);
            match pos {
                Some(idx) => {mana_pool.remove(idx);}
                None => {Err(FailureReason::CouldntPayCost)?;}
            }
        }

        // TODO let the player choose what mana
        for _ in 0..cost.generic_mana { 
            if let None =  mana_pool.pop() {
                Err(FailureReason::CouldntPayCost)?
            }
        }

        Ok(vec![GameEvent::PayMana(player, cost)])
    }

    fn get_player_action(&mut self, player_id: PlayerID, ordering: &AbilityOrdering) -> PlayerAction {

        let mut player_actions = vec![PlayerAction::Pass];

        let keep_sorcery_speed = self.can_play_sorceries(player_id);

        // collect all abilities, then sort into type and if the player controls the ability
        // TODO Sort based on speed! (not implemented for abilities rn)
        for assigned_ability in self.all_abilities(ordering) {
            let player = self.get(assigned_ability.perm).owner;

            if player != player_id {continue}

            match self.get(assigned_ability.ability).base.class {
                AbilityClass::Activated(ref cost, _) => {
                    player_actions.push(
                        PlayerAction::ActivateAbility(
                            assigned_ability, 
                            self.get(assigned_ability.ability).base.description.clone()));
                }
                _ => {}
            }
        }

        // collect all abilities, then sort into type and if the player controls the ability
        for assigned_card_play in self.all_card_plays(ordering) {
            let player = self.get(assigned_card_play.card).owner;

            // only card plays for cards the current player owns
            if player != player_id {continue}
            // only card plays that match the player's allowed speed
            if self.get(assigned_card_play.card_play).speed == AbilitySpeed::Sorcery && !keep_sorcery_speed { continue }

            player_actions.push(
                PlayerAction::CardPlay(assigned_card_play, self.get(assigned_card_play.card_play).description.clone())
            );
        }
        return self.client.choose_options(player_actions)

            /*
        // TODO implement checking of cost and payment + rejection if not good
        let new_event = match self.client.choose_options(player_actions) {
            PlayerAction::CardPlay(as_card_play, _) => GameEvent::PlaySpell(as_card_play),
            PlayerAction::ActivateAbility(as_ability, _) => GameEvent::ActivateAbility(as_ability),
            PlayerAction::Pass => {
            },
        };

        self.push_event(GameEvent::GivePriority(player_id));
        self.push_event(new_event);
        */
    }

    fn can_play_sorceries(&self, player: PlayerID) -> bool {
        player == self.active_player 
        && self.turn_step.is_main_phase() 
        && self.game_stack.is_empty()
    }

    fn add_ability(&mut self, ability: LatentAbility) -> AbilityID {
        let id = self.ability_ids.get_id();
        let ability = Ability::new(ability, id);
        self.abilities.insert(id, ability);
        id
    }

    fn add_card_play(&mut self, card_play: CardPlay) -> CardPlayID {
        let id = self.card_play_ids.get_id();
        self.card_plays.insert(id, card_play);
        id
    }

    pub fn run(&mut self) {
        info!("gameloop: Starting game loop");
        self.push_event(GameEvent::StartTurn(self.active_player));
        let mut ability_order =  AbilityOrdering::build_from(self);

        loop {
            let event = self.event_stack.pop().unwrap();
            info!("gameloop: {} events queued and Current event {:?}", self.event_stack.len(), event);

            let event = match ability_order.listen(event, self) {
                ListenResult::Replaced(new_evs) => {
                    info!("Event Replaced with {} event(s)", new_evs.len());
                    self.push_events(new_evs);
                    continue;
                }
                ListenResult::Triggered(ev, new_evs) => {
                    info!("Event triggered {} event(s)", new_evs.len());
                    self.push_events(new_evs);
                    ev
                }
                ListenResult::Ignored(ev) => ev
            };

            // The event wasn't canceled, so we are now applying it.
            self.default_event_handler(event);
            // The game state has changed, so the ability order should be updated.
            ability_order = AbilityOrdering::build_from(self);
        }
    }

    pub fn next_player(&self, before: PlayerID) -> PlayerID {
        let idx = self.players.iter()
            .enumerate()
            .find(|player| player.1.id == before)
            .unwrap().0;
        let new_idx = (idx + 1) % self.players.len();

        self.players[new_idx].id
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
impl IDMapper<CardPlay> for Game {
    fn get(&self, id: ID<CardPlay>) -> &CardPlay { self.card_plays.get(&id).unwrap() }
    fn get_mut(&mut self, id: ID<CardPlay>) -> &mut CardPlay { self.card_plays.get_mut(&id).unwrap() }
}

impl IDMapper<Card> for Game {
    fn get(&self, id: ID<Card>) -> &Card { 
        self.cards.get_card(id)
    }

    fn get_mut(&mut self, id: ID<Card>) -> &mut Card { 
        self.cards.get_card_mut(id)
    }
}

impl IDMapper<Permanent> for Game {
    fn get(&self, id: ID<Permanent>) -> &Permanent { self.battlefield.get(&id).unwrap() }
    fn get_mut(&mut self, id: ID<Permanent>) -> &mut Permanent { self.battlefield.get_mut(&id).unwrap() }
}
