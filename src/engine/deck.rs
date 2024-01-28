use crate::engine::prelude::*;

#[derive(Clone)]
pub struct Deck {
    pub cards: Vec<Card>
}

impl Deck {
    pub fn empty() -> Self {
        Self{ 
            cards: vec![],
        }
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card)
    }

    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

impl From<Vec<Card>> for Deck {
    fn from(value: Vec<Card>) -> Self {
        Self {cards: value}
    }
}

impl FromIterator<Card> for Deck {
    fn from_iter<T: IntoIterator<Item = Card>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
