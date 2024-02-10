

#[derive(Clone)]
pub struct TypeLine {
    pub card_super_types: Vec<CardSuperType>,
    pub card_types: Vec<CardType>,
    pub card_sub_types: Vec<CardSubType>,
}

impl TypeLine {
    pub fn empty() -> Self {
        Self {
            card_sub_types: vec![],
            card_super_types: vec![],
            card_types: vec![],
        }
    }
}


#[derive(Clone, PartialEq, PartialOrd, Eq)]
pub enum CardType {
    Land,
    Creature,
    Artifact,
    Sorcery,
    Instant,
    Enchantment,
    Planeswalker,
}

#[derive(Clone, PartialEq, PartialOrd, Eq)]
pub enum CardSuperType {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
}

// too many to try and use an enum
pub type CardSubType = &'static str;

pub trait TypeClassQuery<T> {
    fn add(self, a_new_type: T) -> Self;
    fn is(&self, a_type: T) -> bool;
}

impl TypeClassQuery<CardSuperType> for TypeLine {
    fn add(mut self, a_new_type: CardSuperType) -> Self {
        self.card_super_types.push(a_new_type);
        self
    }

    fn is(&self, a_type: CardSuperType) -> bool {
        self.card_super_types.contains(&a_type)
    }
}


impl TypeClassQuery<CardType> for TypeLine {
    fn add(mut self, a_new_type: CardType) -> Self {
        self.card_types.push(a_new_type);
        self
    }

    fn is(&self, a_type: CardType) -> bool {
        self.card_types.contains(&a_type)
    }
}


impl TypeClassQuery<CardSubType> for TypeLine {
    fn add(mut self, a_new_type: CardSubType) -> Self {
        self.card_sub_types.push(a_new_type);
        self
    }

    fn is(&self, a_type: CardSubType) -> bool {
        self.card_sub_types.contains(&a_type)
    }
}
