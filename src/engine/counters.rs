
/// This is a basic representation of counters.
/// this does not approach a good solution.
/// but for now it will work.
#[derive(Clone)]
pub struct Counters {
    pub plus_one_plus_one: u32,
    pub min_one_min_one: u32,
    pub loyalty: u32,
    pub lore: u32,
}

impl Counters {
    pub fn new() -> Self {
        Self {
            plus_one_plus_one: 0,
            loyalty: 0,
            min_one_min_one: 0,
            lore: 0,
        }
    }
}
