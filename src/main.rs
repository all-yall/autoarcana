use core::panic;
use std::{process::exit, collections::BTreeMap, io::{self, stdout, Write}, ptr::addr_of};
use crossterm::{cursor::{
    MoveUp,
    MoveToColumn,
}};

use dyn_clone::DynClone;

use rand::prelude::*;

fn choose_options<T> (prompt: &str, options: &[(&str, T)]) -> Vec<T>
    where 
        T: Copy
    {
    if options.is_empty() {
        return vec![];
    }

    let mut options: Vec<(bool, &str, T)> = options.iter().map(|option| (false, option.0, option.1)).collect();
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        println!("{}", prompt);
        let longest_line = options.iter().enumerate().map( |(option_number, option)| { 
            let indicator = if option.0 {"*"} else {" "};
            let line = format!("[{}] {}. {}", indicator, option_number, option.1);
            println!("{}",line);
            line.len()
        })
            .chain(Some(prompt.len()).into_iter())
            .max().unwrap();
        print!("\n{}(0-{}) > ",MoveUp(1), options.len());
        let _ = stdout().flush();
        input.clear();
        let _ = stdin.read_line(&mut input);
        let input = input.strip_suffix("\n").unwrap();
        if input.is_empty() {
            break;
        }
        match input.parse::<usize>() {
            Ok(option) => {
                if let Some(option) = options.get_mut(option) {
                    option.0 ^= true;
                }
            }
            Err(err) => {}
        }
        let eraser = String::from(" ").repeat(longest_line);
        (0..(options.len()+2)).into_iter().for_each(|_| {
            print!("{}{}{}", MoveUp(1), MoveToColumn(0), eraser);
        });
        print!("{}", MoveToColumn(0));
    }

    options.into_iter().filter_map(|option| { 
        if option.0 {
            Some(option.2)
        } else {
            None
        }
    }).collect()
}


#[derive(Clone, Copy, PartialEq, Eq)]
struct PlayerID(usize);

#[derive(Clone, Copy)]
enum TurnStep {
    Untap,
    Upkeep,
    Draw,
    FirstMainPhase,
    Combat,
    SecondMainPhase,
    Discard,
    CleanUp,
}

const DEFAULT_TURN_STRUCTURE: [TurnStep; 8] = [
    TurnStep::Untap,
    TurnStep::Upkeep,
    TurnStep::Draw,
    TurnStep::FirstMainPhase,
    TurnStep::Combat,
    TurnStep::SecondMainPhase,
    TurnStep::Discard,
    TurnStep::CleanUp,
];

enum EventSource {
    Ability(AbilityID),
    GameRule(),
}

enum GameEvent {
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

#[derive(Clone)]
enum AbilityHolder {
    Card(CardID),
    Permanent(PermanentID),
}

struct Game {
    players: Vec<Player>,
    active_player: PlayerID,
    exile_zone: Deck,
    battlefield: BTreeMap<PermanentID, Permanent>,
    abilities: BTreeMap<AbilityID, Ability>,
    turn_number: usize,
    game_over: bool,
    event_stack: Vec<GameEvent>,

    next_permanent_id: PermanentID,
    next_ability_id: AbilityID,
}


impl Game {
    fn new(players: Vec<Player>, active_player: PlayerID) -> Self {
        Self {
            players,
            active_player,
            exile_zone: Deck::empty(),
            battlefield: BTreeMap::new(),
            next_permanent_id: PermanentID(0),
            turn_number: 0,
            game_over: false,
            event_stack: vec![],
            abilities: BTreeMap::new(),
            next_ability_id: AbilityID(0)
        }
    }

    /**
    * Push the given event onto the event stack and resolve it and all its
    * consequences. This will not resolve the entire event stack. The event
    * stack should hold the same value after as before.
    */
    fn event(&mut self, event: GameEvent) {
        let starting_size = self.event_stack.len();
        self.event_stack.push(event);
        while self.event_stack.len() > starting_size {
            let event = self.event_stack.pop().unwrap();
            self.default_event_handler(event);
        }
    }

    fn default_event_handler(&mut self, event: GameEvent) {
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
                println!("player {} lost.", player_id.0);
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
                let id = self.next_ability_id;
                let holder = AbilityHolder::Permanent(perm);
                let ability = Ability {
                    id,
                    holder,
                    latent_ability,
                };

                self.abilities.insert(ability.id, ability);

                self.next_ability_id.0 += 1;
            }

            AddMana(player_id, mana_type, _) => {
                self.players[player_id.0].mana_pool.push(mana_type);
            }
        }
    }

    fn handle_step_event(&mut self, turn_step: TurnStep, player_id: PlayerID) {
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

    fn main_phase(&mut self, player_id: PlayerID) {
        self.event(GameEvent::PopulateAbilities);
        loop {
            match choose_options("What action would you like to take?", &[
                ("Play Card", 1),
                ("Activate Ability", 2),
                ("Pass", 3),
            ]).first().unwrap()  {
                    // Playing a card
                    1 => {
                        let cards = self.get_player(player_id).hand.cards.iter().enumerate().map(|(idx,card)| (&card.name[..], idx)).collect::<Vec<_>>();
                        let card = match choose_options("What card would you like to play?", &cards[..]).first() {
                            Some(num) => self.get_player(player_id).hand.cards.remove(*num),
                            None => continue,
                        };
                    },
                    2 => {
                            todo!();
                    },
                    3 => {
                        todo!();
                    },
                    _=>{}
                }
        }
    }

    fn get_player(&mut self, player: PlayerID) -> &mut Player {
        self.players.get_mut(player.0).unwrap()
    }

    fn active_player(&mut self) -> &mut Player {
        self.get_player(self.active_player)
    }

    fn run(&mut self) {
        self.event(GameEvent::StartTurn(self.active_player));
    }

    fn next_player(&mut self) {
        self.active_player = PlayerID((self.active_player.0 + 1) % self.players.len());
    }



    fn get_player_id_from_ability_id(&mut self, ability_id: AbilityID) -> PlayerID {
        let ability = self.abilities.get(&ability_id).unwrap();
        match ability.holder {
            AbilityHolder::Permanent(perm) => self.get_player_id_from_perm_id(perm),
            _ => panic!("ahhh!")
        }
    }

    fn get_player_id_from_perm_id(&mut self, perm_id: PermanentID) -> PlayerID {
        let permanent = self.battlefield.get(&perm_id).unwrap();
         permanent.owner
    }

    fn get_ability_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Ability {
        self.abilities.get_mut(&ability_id).unwrap()
    }

    fn get_perm_id_from_ability_id(&mut self, ability_id: AbilityID) -> PermanentID {
        let ability = self.get_ability_from_ability_id(ability_id);
        match ability.holder {
            AbilityHolder::Permanent(perm) => perm,
            _ => panic!("ahhh!")
        }
    }

    fn get_perm_from_ability_id(&mut self, ability_id: AbilityID) -> &mut Permanent {
        let perm_id = self.get_perm_id_from_ability_id(ability_id);
        self.battlefield.get_mut(&perm_id).unwrap()
    }

}


struct Player {
    deck: Deck,
    graveyard: Deck,
    hand: Deck,
    life_total: i32,
    mana_pool: Vec<ManaType>,
}

impl Player {
    fn new(deck: Deck) -> Self {
        Self {
            deck,
            graveyard: Deck::empty(),
            hand: Deck::empty(),
            life_total: 20,
            mana_pool: vec![]
        }
    }
}

#[derive(Clone)]
struct Deck {
    cards: Vec<Card>
}

impl Deck {
    fn empty() -> Self {
        Self{ 
            cards: vec![],
        }
    }

    fn add(&mut self, card: Card) {
        self.cards.push(card)
    }

    fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CardID(usize);

#[derive(Clone)]
struct Card {
    name: String,
    flavor: String,
    card_types: Vec<CardType>,
    owner: PlayerID,
    abilities: Vec<LatentAbility>,
    id: CardID,
    power: Option<i32>,
    toughness: Option<i32>,
}

impl Card {
    fn new(name: String, flavor: String, card_types: Vec<CardType>, abilities: Vec<LatentAbility>, power: Option<i32>, toughness: Option<i32>) -> Self {
        Self {
            name,
            flavor,
            card_types,
            owner: PlayerID(usize::MAX),
            id: CardID(usize::MAX),
            abilities,
            power,
            toughness
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq)]
enum CardType {
    Basic,
    Land(ManaType),
    Creature,
    Artifact,
    Sorcery,
    Instant,
    Enchantment,

    Legendary,
    Goblin,
    Warrior,
}

#[derive(Clone,PartialEq, Eq)]
struct Cost {
    mana: Vec<ManaType>,
    generic_mana: usize,
    tap: bool,
}

impl Cost {
    fn new() -> Self {
        Self {
            mana: vec![],
            generic_mana: 0,
            tap: false,
        }
    }

    fn with_tap(mut self) -> Self {
        self.tap = true;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PermanentID(usize);
struct Permanent {
    name: String,
    flavor: String,
    types: Vec<CardType>,
    card: Option<Card>,
    owner: PlayerID,
    base_power: i32,
    base_toughness: i32,
    id: PermanentID,
    tapped: bool,
    intrinsic_abilities: Vec<LatentAbility>,
}

impl Permanent {
    fn untap(&mut self) {
        self.tapped = false;
    }
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
enum AbilityClass {
    Activated(Cost),
    Triggered,
    Static,
    Mana,
    Loyalty,
    Linked,
}

#[derive(Clone, Copy, Ord,PartialEq, PartialOrd, Eq)]
struct AbilityID(usize);

#[derive(Clone)]
struct Ability {
    holder: AbilityHolder,
    id: AbilityID,
    latent_ability: LatentAbility,
}

#[derive(Clone)]
struct LatentAbility {
    class: AbilityClass,
    description: String,
    effect: Effect,
}

#[derive(Clone)]
enum Effect {
    Continuous(Box<dyn Continuous>),
    OneShot(Box<dyn OneShot>),
}

trait Continuous: DynClone {
    fn done(&mut self) -> bool { false }
    fn listen(&mut self, ability: AbilityID, event: GameEvent, game: &mut Game) -> Vec<GameEvent>;
}
dyn_clone::clone_trait_object!(Continuous);

trait OneShot: DynClone {
    fn activate(&mut self, ability: AbilityID, game: &mut Game);
}
dyn_clone::clone_trait_object!(OneShot);


#[derive(Clone)]
struct NullEffect {}
impl NullEffect {
    fn get() -> Box<Self> {
        Box::new(Self{})
    }
}
impl Continuous for NullEffect {
    fn done(&mut self) -> bool {
        false
    }

    fn listen(&mut self, _ability: AbilityID, event: GameEvent, _game: &mut Game) -> Vec<GameEvent> {
        vec![event]
    }
}

impl OneShot for NullEffect {
    fn activate(&mut self, _ability: AbilityID, _game: &mut Game) {
    }
}

#[derive(Clone)]
struct AddManaEffect {
    mana_type: ManaType
}
impl AddManaEffect {
    fn new(mana_type: ManaType) -> Box<Self> {
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
struct MiraisMana {}
impl MiraisMana {
    fn get() -> Box<Self> { Box::new(Self{}) }
}
impl Continuous for MiraisMana {
    fn listen(&mut self, ability_id: AbilityID, event: GameEvent, game: &mut Game) -> Vec<GameEvent> {
        let mut additional = None;
        match event {
            GameEvent::AddMana(player_id_recv_mana, mana_type, EventSource::Ability(ability_id_source)) => {
                let i_am_recieving_mana = game.get_player_id_from_ability_id(ability_id) == player_id_recv_mana;
                let it_is_from_a_land = game.get_perm_from_ability_id(ability_id_source).types.iter().find(
                    |card_type| matches!(card_type, CardType::Land(_))
                ).is_some();
                if i_am_recieving_mana && it_is_from_a_land {
                    additional = Some(GameEvent::AddMana(player_id_recv_mana, mana_type, EventSource::Ability(ability_id)));
                }
            }
            _ => {}
        }
        additional.into_iter().chain(Some(event).into_iter()).collect()
    }
}

fn get_card(name: &str) -> Card {
    use CardType::*;
    use ManaType::*;
    match name {
        "mountain" => Card::new(
            "Mountain".to_string(),
            "One day, night will come to these mountains.".to_string(), 
            vec![Basic, Land(Red)],
            vec![ 
                LatentAbility {
                    class: AbilityClass::Activated(Cost::new().with_tap()),
                    description: "Add one red mana".to_string(),
                    effect: Effect::OneShot(AddManaEffect::new(Red)),
                },
            ],
            None, None,
        ),

        "miraris wake" => Card::new(
            "Mirari's Wake".to_string(),
            "Even after a false god tore the magic from Dominaria, power still radiated from the Mirari sword that slew her.".to_string(),
            vec![Enchantment],
            vec![
                LatentAbility {
                    class: AbilityClass::Static,
                    description: "Creatures you control get +1/+1.".to_string(),
                    effect: Effect::Continuous(NullEffect::get()),
                },
                LatentAbility {
                    class: AbilityClass::Static,
                    description: "Whenever you tap a land for mana, add one mana of any type that land produced.".to_string(),
                    effect: Effect::Continuous(MiraisMana::get()),
                }
            ],
            None, None,
        ),

        "goblin assailant" => Card::new(
            "Goblin Assailant".to_string(),
            "What he lacks in patience, intelligence, empathy, lucidity, hygiene, ability to follow orders, self-regard, and discernible skills, he makes up for in sheer chaotic violence.".to_string(),
            vec![Creature, Goblin, Warrior],
            vec![],
            Some(2), Some(2),
        ),

        other => panic!("no card named '{}'", other),
    }
}

fn main() {
    let mut cards = Deck::empty();
    cards.add(get_card("mountain"));
    cards.add(get_card("mountain"));
    cards.add(get_card("mountain"));
    cards.add(get_card("mountain"));
    cards.add(get_card("miraris wake"));
    cards.add(get_card("miraris wake"));
    cards.add(get_card("goblin assailant"));
    cards.add(get_card("goblin assailant"));
    let players = vec![
        Player::new(cards.clone()),
        Player::new(cards),
    ];
    let mut game = Game::new(players, PlayerID(0));
    game.run();
}
