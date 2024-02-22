use std::{fmt::Debug, vec};

use super::prelude::*;

#[derive(Debug)]
pub enum GameObject {
    Permanent(Permanent),
    Object, // Sorceries/Instants
}

#[derive(Debug)]
pub enum GameObjectID {
    Permanent(PermanentID),
    Object, // Sorceries/Instants
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl TurnStep {
    pub fn is_main_phase(self) -> bool {
        self == Self::FirstMainPhase || self == Self::SecondMainPhase
    }
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

#[derive(Debug)]
pub enum EventSource {
    Permanent(PermanentID),
    Player(PlayerID),
    GameRule(GameRule),
}

#[derive(Debug)]
pub enum GameRule {


    // State Based Actions
    
    /// Destroy because of Damage
    LethalDamage,
    /// Lose because of no health
    NoHealth,
    /// A plainswalker dies with no loyalty
    NoLoyalty,
    /// Cancel out plus and minus Counters
    CancelOutCounters,
    /// If you draw but can't, then you lose
    CouldntDraw,
    /// Sagas with no lore counters are sacrificed
    NoLore,
}

impl Into<EventSource> for GameRule {
    fn into(self) -> EventSource { EventSource::GameRule(self) }
}

/// This represents any game modification event
/// that is relevant to other abilities, and could
/// potentially be modified by them.
#[derive(Debug)]
pub enum GameEvent {
    /// Start the given player's turn
    StartTurn(PlayerID),

    /// Start the given step of the active player's turn
    Step(TurnStep),

    /// Untap the permanent
    UntapPerm(PermanentID),

    /// Taps the permanent
    TapPerm(PermanentID),

    /// Draw a card from the top of the deck
    DrawCard(PlayerID),

    /// Play the card. This assumes the cost has alreaady been paid.
    PlaySpell(AssignedCardPlay),

    /// Activates the ability. This assumes cost has already been paid.
    ActivateAbility(AssignedAbility),

    /// Pays mana out of given player's pool. 
    /// Assumes player can pay it, otherwise its an error
    PayMana(PlayerID, ManaCost),

    /// GameRule firing saying one player can't control multiple of the
    /// same legendary. The player will need to put one in its owners 
    /// graveyard.
    LegendConflict(Vec<PermanentID>),


    /// the player gains mana
    AddMana(PlayerID, ManaType, EventSource),

    /// The given permanent is destroyed and placed in the graveyard.
    Destroy(PermanentID, EventSource),

    /// The given permanent was sacrificed 
    Sacrifice(PermanentID, EventSource),

    /// The given player will lose
    Lose(PlayerID, EventSource),

    /// Add counters of given kind on object
    AddCounters(GameObjectID, CounterType, u32, EventSource),

    /// Remove counters of given kind on object
    RemoveCounters(GameObjectID, CounterType, u32, EventSource),

    /// Permanent is registered, then Enter the Battlefield event is fired.
    RegisterPermanent(Permanent),

    /// The permanent has entered the battlefield.
    EnterTheBattleField(PermanentID),

    /// The game gives the player priority
    GivePriority(PlayerID),

    /// The game tries to pass prioirity. It might 
    /// resolve into a 'TryResolveStackObject' action.
    PassPriority(PlayerID),

    /// The game tries to resolve the top object
    /// on the stack. If stack is empty, does nothing
    TryResolveStackObject,

    /// The game moves onto the next step
    NextStep,
}

impl GameEvent {
    pub fn triggered(self, another: Self) -> ListenResult {
        ListenResult::Triggered(self, vec![another]) 
    }

    pub fn ignored(self) -> ListenResult {
        ListenResult::Ignored(self)
    }

    pub fn replaced(self, by: Self) -> ListenResult {
        ListenResult::Replaced(vec![by])
    }
}

#[derive(Debug)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
}

/// This represents any read of game state that could
/// be modified by other abilities. Mostly continuous
/// effects are what should be considered.
#[derive(Debug)]
pub enum GameQuery {
    PermAbilities(PermAbilityQuery),
    ObservePerm(ObservePermQuery),
    CardPlays(CardPlaysQuery),
}


#[derive(Debug)]
pub struct PermAbilityQuery {
    pub id: PermanentID,
    pub abilities: Vec<AbilityID>
}

impl PermAbilityQuery {
    pub fn new(id: PermanentID) -> Self {
        PermAbilityQuery{
            id,
            abilities: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ObservePermQuery {
    pub perm: Permanent
}

impl ObservePermQuery {
    pub fn new(perm: Permanent) -> Self {
        ObservePermQuery{
            perm
        }
    }
}

#[derive(Debug)]
pub struct CardPlaysQuery {
    pub id: CardID,
    pub card_plays: Vec<CardPlayID>,
}

impl CardPlaysQuery {
    pub fn new(id: CardID) -> Self {
        CardPlaysQuery { 
            id , 
            card_plays: vec![]
        }
    }
}



pub trait GameQueryVariant: Debug + TryFrom<GameQuery> + Into<GameQuery> {}

/// Implements TryFrom<GameQuery> for Query structs.
macro_rules! make_game_query_variant {
    ($variant: ty, $wrapping_enum_variant: tt) => {
        impl GameQueryVariant for $variant {}

        impl TryFrom<GameQuery> for $variant {
            type Error = GameQuery;
            fn try_from(value: GameQuery) -> Result<Self, Self::Error> {
                match value {
                    GameQuery::$wrapping_enum_variant(ret) => Ok(ret),
                    a => Err(a),
                }
            }
        }

        impl Into<GameQuery> for $variant {
            fn into(self) -> GameQuery {
                GameQuery::$wrapping_enum_variant(self) 
            } 
        }
    };
}

make_game_query_variant!(PermAbilityQuery, PermAbilities);
make_game_query_variant!(ObservePermQuery, ObservePerm);
make_game_query_variant!(CardPlaysQuery, CardPlays);
