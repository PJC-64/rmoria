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
    #[serde(default)]
    pub stunned: u32,
    #[serde(default)]
    pub was_attacked: bool,
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
            stunned: 0,
            was_attacked: false,
        }
    }

    /// Deal damage to monster. Returns `true` if monster died.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.hp -= amount;
        self.hp <= 0
    }

    /// Basic AI movement. Calculates standard grid step closer to player.
    /// Returns `Some((new_x, new_y))` if a valid step is calculated.
    pub fn update_ai(
        &self,
        player_x: usize,
        player_y: usize,
        level: &crate::dungeon::DungeonLevel,
        depth: u32,
        killed_town_npcs: u32,
        threshold: u32,
    ) -> Option<(usize, usize)> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let is_hostile = depth > 0 || self.was_attacked || killed_town_npcs > threshold;

        if self.symbol == 'p' {
            // Townsfolk wander semi-randomly
            let wander_chance = if is_hostile { 0.70 } else { 1.00 };
            if rng.gen_bool(wander_chance) {
                let dx = rng.gen_range(-1..=1);
                let dy = rng.gen_range(-1..=1);
                if dx == 0 && dy == 0 {
                    return None;
                }
                let next_x = (self.x as isize + dx) as usize;
                let next_y = (self.y as isize + dy) as usize;
                if let Some(tile) = level.get_tile(next_x, next_y)
                    && tile.is_passable() {
                        return Some((next_x, next_y));
                    }
                return None;
            }
        }

        if !is_hostile {
            return None;
        }

        let dx = (player_x as isize - self.x as isize).signum();
        let dy = (player_y as isize - self.y as isize).signum();

        let next_x = (self.x as isize + dx) as usize;
        let next_y = (self.y as isize + dy) as usize;

        if let Some(tile) = level.get_tile(next_x, next_y)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((next_x, next_y));
            }

        let next_x_only = (self.x as isize + dx) as usize;
        if let Some(tile) = level.get_tile(next_x_only, self.y)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((next_x_only, self.y));
            }

        let next_y_only = (self.y as isize + dy) as usize;
        if let Some(tile) = level.get_tile(self.x, next_y_only)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((self.x, next_y_only));
            }

        None
    }
}

#[derive(Debug, Clone)]
pub struct MonsterTemplate {
    pub name: &'static str,
    pub symbol: char,
    pub level: u32,
    pub max_hp: i32,
    pub damage: Dice,
    pub exp_reward: u32,
}

pub const MONSTER_DB: &[MonsterTemplate] = &[
    MonsterTemplate { name: "Red Mold", symbol: 'm', level: 1, max_hp: 5, damage: Dice { num: 1, sides: 3 }, exp_reward: 5 },
    MonsterTemplate { name: "Giant Centipede", symbol: 'c', level: 1, max_hp: 7, damage: Dice { num: 1, sides: 3 }, exp_reward: 7 },
    MonsterTemplate { name: "Goblin", symbol: 'g', level: 2, max_hp: 8, damage: Dice { num: 1, sides: 4 }, exp_reward: 10 },
    MonsterTemplate { name: "Giant White Rat", symbol: 'r', level: 2, max_hp: 9, damage: Dice { num: 1, sides: 4 }, exp_reward: 12 },
    MonsterTemplate { name: "Orc", symbol: 'o', level: 3, max_hp: 15, damage: Dice { num: 1, sides: 6 }, exp_reward: 25 },
    MonsterTemplate { name: "Baby Dragon", symbol: 'd', level: 3, max_hp: 20, damage: Dice { num: 2, sides: 4 }, exp_reward: 35 },
    MonsterTemplate { name: "Cave Troll", symbol: 'T', level: 4, max_hp: 30, damage: Dice { num: 2, sides: 6 }, exp_reward: 50 },
    MonsterTemplate { name: "Spectre", symbol: 'S', level: 4, max_hp: 22, damage: Dice { num: 1, sides: 8 }, exp_reward: 60 },
    MonsterTemplate { name: "Uruk-Hai", symbol: 'U', level: 5, max_hp: 35, damage: Dice { num: 2, sides: 6 }, exp_reward: 80 },
    MonsterTemplate { name: "Giant Spider", symbol: 's', level: 5, max_hp: 28, damage: Dice { num: 1, sides: 10 }, exp_reward: 75 },
    MonsterTemplate { name: "Ghost Warrior", symbol: 'W', level: 6, max_hp: 40, damage: Dice { num: 2, sides: 8 }, exp_reward: 120 },
    MonsterTemplate { name: "Lich", symbol: 'L', level: 7, max_hp: 55, damage: Dice { num: 3, sides: 6 }, exp_reward: 180 },
];
