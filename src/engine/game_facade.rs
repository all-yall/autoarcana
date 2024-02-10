use super::prelude::*;

/// The point of this struct is to obscure the game object and only allow 
/// interacting with it in appropriate ways for abilities and player actions.
/// This means that instead of changing the game  directly, events are queued.
/// And that the instead of reading the game directly, queries are run.
pub struct GameFacade<'a> {
    game: &'a Game,
    ability_order: AbilityOrdering,
    queued_events: Vec<GameEvent>,
}

impl<'a> GameFacade<'a> {
    pub fn new(game: &'a Game) -> Self {
        let ability_order = AbilityOrdering::build_from(game);
        let ret = Self {
            game,
            ability_order,
            queued_events: Vec::new(),
        };

        ret
    }

    pub fn perms(&self) -> Box<dyn Iterator<Item=PermanentID>> {
        Box::new(self.game.battlefield.keys().cloned())
    }


    pub fn players(&self) -> Box<dyn Iterator<Item=PlayerID>> {
        Box::new(self.game.players.iter().map(|player| player.id))
    }

    pub fn player_life(&self, player: PlayerID) -> i32 {
        // TODO maybe should be query
        self.game.get(player).life_total
    }

    pub fn observe_perm(&self, perm: PermanentID) -> Permanent {
        let query = self.game.query(
            ObservePermQuery::new(self.game.get(perm).clone()),
            &self.ability_order);
        query.perm
    }


    pub fn get_queued_events(self) -> Vec<GameEvent> {
        self.queued_events
    }
}


