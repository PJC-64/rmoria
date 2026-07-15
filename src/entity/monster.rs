use serde::{Serialize, Deserialize};
use crate::dice::Dice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub name: String,
    pub symbol: char,
    pub x: usize,
    pub y: usize,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: Dice,
    pub experience_reward: u32,
}

impl Monster {
    pub fn new(name: &str, symbol: char, x: usize, y: usize, max_hp: i32, damage: Dice, experience_reward: u32) -> Self {
        Self {
            name: name.to_string(),
            symbol,
            x,
            y,
            hp: max_hp,
            max_hp,
            damage,
            experience_reward,
        }
    }

    /// Deal damage to monster. Returns `true` if monster died.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.hp -= amount;
        self.hp <= 0
    }

    /// Basic AI movement. Calculates standard grid step closer to player.
    /// Returns `Some((new_x, new_y))` if a valid step is calculated.
    pub fn update_ai(&self, player_x: usize, player_y: usize, level: &crate::dungeon::DungeonLevel) -> Option<(usize, usize)> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if self.symbol == 'p' {
            // Townsfolk wander semi-randomly
            if rng.gen_bool(0.70) {
                let dx = rng.gen_range(-1..=1);
                let dy = rng.gen_range(-1..=1);
                if dx == 0 && dy == 0 {
                    return None;
                }
                let next_x = (self.x as isize + dx) as usize;
                let next_y = (self.y as isize + dy) as usize;
                if let Some(tile) = level.get_tile(next_x, next_y) {
                    if tile.is_passable() {
                        return Some((next_x, next_y));
                    }
                }
                return None;
            }
        }

        let dx = (player_x as isize - self.x as isize).signum();
        let dy = (player_y as isize - self.y as isize).signum();

        let next_x = (self.x as isize + dx) as usize;
        let next_y = (self.y as isize + dy) as usize;

        if let Some(tile) = level.get_tile(next_x, next_y) {
            if tile.is_passable() {
                return Some((next_x, next_y));
            }
        }

        let next_x_only = (self.x as isize + dx) as usize;
        if let Some(tile) = level.get_tile(next_x_only, self.y) {
            if tile.is_passable() {
                return Some((next_x_only, self.y));
            }
        }

        let next_y_only = (self.y as isize + dy) as usize;
        if let Some(tile) = level.get_tile(self.x, next_y_only) {
            if tile.is_passable() {
                return Some((self.x, next_y_only));
            }
        }

        None
    }
}
