


#[derive(Clone,PartialEq, Eq)]
pub struct Cost {
    mana: Vec<ManaType>,
    generic_mana: usize,
    tap: bool,
}

impl Cost {
    pub fn new() -> Self {
        Self {
            mana: vec![],
            generic_mana: 0,
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
