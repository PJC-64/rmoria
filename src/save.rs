use crate::dungeon::DungeonLevel;
use crate::entity::monster::Monster;
use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub level: DungeonLevel,
    pub monsters: Vec<Monster>,
    pub max_depth: u32,
    #[serde(default)]
    pub level_cache: std::collections::HashMap<u32, (DungeonLevel, Vec<Monster>)>,
}

impl GameState {
    pub fn new(
        player: Player,
        level: DungeonLevel,
        monsters: Vec<Monster>,
        max_depth: u32,
        level_cache: std::collections::HashMap<u32, (DungeonLevel, Vec<Monster>)>,
    ) -> Self {
        Self {
            player,
            level,
            monsters,
            max_depth,
            level_cache,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let serialized = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let state: GameState = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(state)
    }
}
