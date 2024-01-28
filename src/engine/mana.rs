

#[derive(Clone,PartialEq, Eq)]
pub struct ManaCost {
    mana: Vec<ManaType>,
    generic_mana: usize,
}

impl ManaCost {
    pub fn new(mana: Vec<ManaType>, generic_mana: usize) -> Self {
        Self{ mana, generic_mana }
    }

    pub fn empty() -> Self {
        Self::new(vec![], 0)
    }
}

#[derive(Clone,PartialEq, Eq)]
pub struct Cost {
    mana_cost: ManaCost,
    tap: bool,
}

impl Cost {
    pub fn empty() -> Self {
        Self {
            mana_cost: ManaCost::empty(),
            tap: false,
        }
    }

    pub fn with_tap(mut self) -> Self {
        self.tap = true;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}
