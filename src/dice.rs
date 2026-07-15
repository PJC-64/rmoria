use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dice {
    pub num: u32,
    pub sides: u32,
}

impl Dice {
    pub fn new(num: u32, sides: u32) -> Self {
        Self { num, sides }
    }

    /// Roll the dice using a random number generator.
    pub fn roll<R: Rng>(&self, rng: &mut R) -> u32 {
        let mut total = 0;
        for _ in 0..self.num {
            total += rng.gen_range(1..=self.sides);
        }
        total
    }
}
