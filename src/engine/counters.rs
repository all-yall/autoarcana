
/// This is a basic representation of counters.
/// this does not approach a good solution.
/// but for now it will work.
#[derive(Clone)]
pub struct Counters {
    pub plus_one_plus_one: usize,
    pub min_one_min_one: usize,
    pub loyalty: usize,
    pub lore: usize,
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
