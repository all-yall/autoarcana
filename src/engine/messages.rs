use super::prelude::*;

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
    Permanent(PermanentID),
    Player(PlayerID),
    GameRule,
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

    PlaySpell(AssignedCardPlay),
    ActivateAbility(AssignedAbility),

    AddMana(PlayerID, ManaType, EventSource),

    RegisterPermanent(Permanent),
    EnterTheBattleField(PermanentID),
}


/// This represents any read of game state that could
/// be modified by other abilities. Mostly continuous
/// effects are what should be considered.
pub enum GameQuery {
    PermanentAbilities(PermanentID, Vec<AbilityID>),
    CardAbilities(CardID, Vec<CardPlayID>),
}

