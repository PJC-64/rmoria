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
        let path = path.as_ref();
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));

        let write_res = (|| -> Result<(), io::Error> {
            let mut file = File::create(&tmp_path)?;
            file.write_all(serialized.as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&tmp_path, path)?;
            Ok(())
        })();

        if write_res.is_err() {
            let _ = std::fs::remove_file(&tmp_path);
        }
        write_res
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Class, Player, Race};

    #[test]
    fn test_atomic_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let save_path = temp_dir.join(format!("test_save_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&save_path);

        let player = Player::new("TestSaver", Race::Human, Class::Warrior, 5, 5);
        let level = DungeonLevel::new(20, 10, 1, 50);
        let monsters = Vec::new();
        let state = GameState::new(
            player,
            level,
            monsters,
            50,
            std::collections::HashMap::new(),
        );

        assert!(state.save_to_file(&save_path).is_ok());
        assert!(save_path.exists());

        let loaded = GameState::load_from_file(&save_path);
        assert!(loaded.is_ok());
        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.player.name, "TestSaver");
        assert_eq!(loaded_state.max_depth, 50);

        let _ = std::fs::remove_file(&save_path);
    }
}
