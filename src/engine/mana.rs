use std::ops::{DerefMut, Deref};


#[derive(Clone,PartialEq, Eq)]
pub struct AbilityCost {
    pub cost: Cost,
    pub tap: bool,
}

impl AbilityCost {
    pub fn empty() -> Self {
        Self { cost: Cost::empty(), tap: false }
    }

    pub fn with_tap(mut self) -> Self {
        self.tap = true;
        self
    }
}

#[derive(Clone,PartialEq, Eq, Debug)]
pub struct ManaCost {
    pub mana: Vec<ManaType>,
    pub generic_mana: usize,
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
    pub mana_cost: ManaCost,
}

impl Cost {
    pub fn empty() -> Self {
        Self {
            mana_cost: ManaCost::empty(),
        }
    }

    pub fn with_mana(mut self, mana_cost: ManaCost) -> Self {
        self.mana_cost = mana_cost;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
pub enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl Into<Cost> for ManaCost {
    fn into(self) -> Cost {
        Cost::empty().with_mana(self)
    }
}

impl Into<AbilityCost> for Cost {
    fn into(self) -> AbilityCost {
        AbilityCost { cost: self, tap: false }
    }
}

impl Deref for AbilityCost {
    type Target = Cost;
    fn deref(&self) -> &Self::Target {
        &self.cost
    }
}

impl DerefMut for AbilityCost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cost
    }
}

impl Deref for Cost {
    type Target = ManaCost;
    fn deref(&self) -> &Self::Target {
        &self.mana_cost
    }
}
impl DerefMut for Cost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mana_cost
    }
}
