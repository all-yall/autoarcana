use std::{collections::BTreeMap, process::exit};

use crate::{engine::prelude::*, client::GameStateSnapshot};
use crate::client::{Client, PlayerAction};

use super::util::id::{
    ID,
    IDFactory,
    IDMapper,
};

use tokio::sync::broadcast::{channel, Sender as BroadcastSender};


/// The Game object contains all game information
/// and additionally acts as a store for all game
/// objects so that they are easily found and rusts
/// borrow checker doesn't get too mad.
pub struct Game {
    pub players: Vec<Player>,
    pub active_player: PlayerID,
    pub turn_number: usize,
    pub game_over: bool,
    pub event_stack: Vec<GameEvent>,
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
            players: Vec::new(),
            turn_number: 0,
            game_over: false,
            event_stack: vec![],

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
        self.event_stack.push(event);
    }

    pub fn default_event_handler(&mut self, event: GameEvent, ability_order: &AbilityOrdering) {
        use GameEvent::*;
        match event {
            StartTurn(player_id) => {
                // in reverse because event_stack is lifo
                for turn_step in DEFAULT_TURN_STRUCTURE.iter().rev() {
                    self.event_stack.push(Step(*turn_step, player_id));
                }
            },
            Step(step, player_id) => self.handle_step_event(step, player_id, ability_order), 

            DrawCard(player_id) => {
                let drawn_card = self.cards.draw(player_id);
                // Couldn't draw a card, lose the game.
                if drawn_card.is_none() { 
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

            RegisterPermanent(perm) => {
                let id = perm.id;
                self.battlefield.insert(id, perm);
                self.push_event(EnterTheBattleField(id));
            }

            AddMana(player_id, mana_type, _) => {
                self.get_player(player_id).mana_pool.push(mana_type);
            }

            PlaySpell(as_card_play) => {
                let card_play = self.get(as_card_play.card_play);
                card_play.spawn.spawn(as_card_play.card, self);
            }

            ActivateAbility(as_ability) => {
                // TODO: this is not the best way to do this; removing the ability from
                // the map the get around the borrow checker. 
                // a better solution in which abilities are not stored in the game object,
                // or the abilities not being able to modify the game directly
                let ability = self.abilities.remove(&as_ability.ability).expect("Invalid ability ID should not be possible");
                match ability.base.class {
                    AbilityClass::Activated(_, ref ability) => ability.activate(as_ability.ability, as_ability.perm, self),
                    _ => panic!("Expected activated ability for ActivateAbility event")
                }
                self.abilities.insert(as_ability.ability, ability);
            }

            EnterTheBattleField(_) => {}
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
        let mut query = GameQuery::CardAbilities(card_id, vec![]);
        self.query(&mut query, order);
        if let GameQuery::CardAbilities(_, abilities) = query { 
            return abilities
                .into_iter()
                .map(|card_play| AssignedCardPlay::new(card_id, card_play))
                .collect()
        }
        panic!("query type was changed!");
    }

    pub fn perm_abilities(&self, perm_id: PermanentID, order: &AbilityOrdering) -> Vec<AssignedAbility> {
        let mut query = GameQuery::PermanentAbilities(perm_id, vec![]);
        self.query(&mut query, order);
        if let GameQuery::PermanentAbilities(_, abilities) = query { 
            return abilities
                .into_iter()
                .map(|ability_id| AssignedAbility::new(perm_id, ability_id))
                .collect() 
        } else {
            panic!("query type was changed!");
        }
    }


    pub fn query(&self, query: &mut GameQuery, ability_order: &AbilityOrdering) {
        match query {
            GameQuery::PermanentAbilities(perm_id, ref mut abilities) => {
                let perm_abilities = self.battlefield
                    .get(perm_id)
                    .unwrap()
                    .abilities
                    .iter();
                abilities.extend(perm_abilities);
            }

            GameQuery::CardAbilities(card_id, ref mut card_plays) => {
                card_plays.extend(self.cards.get_card(*card_id).card_plays.iter());
            }
        }

        ability_order.query(self, query);
    }

    pub fn handle_step_event(&mut self, turn_step: TurnStep, player_id: PlayerID, ability_order: &AbilityOrdering) {
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
            FirstMainPhase => self.main_phase(player_id, ability_order),
            Combat => todo!(),
            SecondMainPhase => todo!(),
            Discard => todo!(),
            CleanUp => todo!(),
        }
    }

    pub fn main_phase(&mut self, player_id: PlayerID, ability_order: &AbilityOrdering) {
        loop {
            let mut player_actions = vec![PlayerAction::Pass];

            // collect all abilities, then sort into type and if the player controls the ability
            for assigned_ability in self.all_abilities(&ability_order) {
                let player = self.get(assigned_ability.perm).owner;

                if player != player_id {continue}
                player_actions.push(
                    PlayerAction::ActivateAbility(assigned_ability, self.get(assigned_ability.ability).base.description.clone())
                );
            }

            // collect all abilities, then sort into type and if the player controls the ability
            for assigned_card_play in self.all_card_plays(&ability_order) {
                let player = self.get(assigned_card_play.card).owner;

                if player != player_id {continue}
                player_actions.push(
                    PlayerAction::CardPlay(assigned_card_play, self.get(assigned_card_play.card_play).description.clone())
                );
            }

            match self.client.choose_options(player_actions) {
                PlayerAction::CardPlay(idx, _) => todo!(),
                PlayerAction::ActivateAbility(ability_id, _) => todo!(),
                PlayerAction::Pass => continue,
            }
        }
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
        let mut ability_order =  AbilityOrdering::build_from(self);

        loop {
            let event = self.event_stack.pop().unwrap();
            let (maybe_original_event, new_events) = ability_order.listen(event, self);
            self.event_stack.extend(new_events.into_iter());

            // The event wasn't canceled, so we are now applying it.
            if let Some(event) = maybe_original_event {
                self.default_event_handler(event, &ability_order);
                // The game state has changed, so the ability order should be updated.
                ability_order = AbilityOrdering::build_from(self);
            }
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


    pub fn get_player_id_from_perm_id(&mut self, perm_id: PermanentID) -> PlayerID {
        self.get_perm_from_perm_id(perm_id).owner
    }

    pub fn get_player_id_from_card_id(&mut self, card_id: CardID) -> PlayerID {
        self.get(card_id).owner
    }

    pub fn get_perm_from_perm_id(&mut self, perm_id: PermanentID) -> &mut Permanent {
        self.battlefield.get_mut(&perm_id).unwrap()
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
