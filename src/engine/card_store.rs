use std::collections::BTreeMap;
use super::prelude::*;



#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub enum Zone {
    Hand(PlayerID),
    Graveyard(PlayerID),
    Deck(PlayerID),
    Exile,
    Battlefield,
    Stack,
}

/// This keeps track of all the cards and which zone they are in.
/// This keeps track of ordering as well. For some zones such as
/// battlefield and exile this doesn't matter, but for deck this 
/// matters.
pub struct CardStore {
    id_to_card: BTreeMap<CardID, (Card, Zone)>,
    zone_to_cards: BTreeMap<Zone, Vec<CardID>> 
}

impl CardStore {
    pub fn new(players: &Vec<PlayerID>) -> Self {
        let mut ret = Self {
            id_to_card: BTreeMap::new(),
            zone_to_cards: BTreeMap::new(),
        };

        use Zone::*;

        for zone in vec![Exile, Battlefield, Stack] {
            ret.zone_to_cards.insert(zone, vec![]);
        }

        for player in players {
            ret.zone_to_cards.insert(Hand(*player), vec![]);
            ret.zone_to_cards.insert(Graveyard(*player), vec![]);
            ret.zone_to_cards.insert(Deck(*player), vec![]);
        }

        ret
    }

    pub fn get_cards(&self, zone: Zone) -> Vec<&Card> {
        self.zone_to_cards
            .get(&zone)
            .expect("Zone doesn't exist")
            .iter()
            .map(|card_id| 
                &self.id_to_card.get(card_id)
                    .expect("card store fields are inconsistent").0
            ).collect()
    }

    pub fn get_card(&self, id: CardID) -> &Card {
        &self.get_unwrap(id).0
    }
    
    pub fn get_card_mut(&mut self, id: CardID) -> &mut Card {
        &mut self.get_mut_unwrap(id).0
    }

    pub fn get_zone(&self, id: CardID) -> Zone {
        self.get_unwrap(id).1
    }

    pub fn get(&self, id: CardID) -> (&Card, Zone) {
        let &(ref card, zone) = self.get_unwrap(id);
        (card, zone)
    }

    /// Move the card corresponding to the given id to the given zone
    pub fn move_to_zone(&mut self, id: CardID, zone: Zone) {
        let card = self.take_card(id);
        self.put_card(card, zone);
    }

    fn take_card(&mut self, id: CardID) -> Card {
        let card = self.id_to_card.remove(&id).expect("Card id doesn't exist");
        let cards = self.zone_to_cards.get_mut(&card.1).expect("zone doesn't exist");
        let pos = cards.iter().position(|&card_id| card_id == id).expect("card store fields are inconsistent");
        cards.remove(pos);
        card.0
    }

    pub fn hand(&self, id: PlayerID) -> Vec<&Card> {
        self.get_cards(Zone::Deck(id))
    }

    /// Moves card from player's deck to the player's hand (if possible)
    /// returns id of drawn card. If you couldn't draw, returns None.
    pub fn draw(&mut self, id: PlayerID) -> Option<CardID> {
        let maybe_card_id = self.zone_to_cards.get_mut(&Zone::Deck(id)).unwrap().pop();
        if let Some(card_id) = maybe_card_id {
            self.zone_to_cards.get_mut(&Zone::Hand(id)).unwrap().push(card_id);
            Some(card_id)
        } else {
            None
        }
    }

    pub fn shuffle(&mut self, id: PlayerID) {
        let deck = self.zone_to_cards.get_mut(&Zone::Deck(id));
        todo!() //blah, randomness
    }

    pub fn put_card(&mut self, card: Card, zone: Zone) {
        self.zone_to_cards.get_mut(&zone).unwrap().push(card.id);
        self.id_to_card.insert(card.id, (card, zone));
    }

    fn get_unwrap(&self, id: CardID) -> &(Card, Zone) {
        self.id_to_card.get(&id).expect("Card id doesn't exist")
    }

    fn get_mut_unwrap(&mut self, id: CardID) -> &mut (Card, Zone) {
        self.id_to_card.get_mut(&id).expect("Card id doesn't exist")
    }
}
