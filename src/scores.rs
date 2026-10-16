use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const MAX_HIGH_SCORE_ENTRIES: usize = 1000;
pub const DEFAULT_SCORES_FILE: &str = "rmoria_scores.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub points: i64,
    pub birth_date: u64,
    pub mhp: i32,
    pub chp: i32,
    pub dungeon_depth: u32,
    pub level: u32,
    pub deepest_dungeon_depth: u32,
    pub gender: char,
    pub race: String,
    pub character_class: String,
    pub name: String,
    pub died_from: String,
}

pub fn clean_died_from(died_from: &str) -> String {
    let trimmed = died_from.trim();
    if let Some(rest) = trimmed.strip_prefix("an ") {
        rest.trim().to_string()
    } else if let Some(rest) = trimmed.strip_prefix("a ") {
        rest.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

impl HighScoreEntry {
    pub fn from_player(player: &Player, current_depth: u32, died_from: &str) -> Self {
        let points = player.calculate_total_points(current_depth);
        let gender = match player.gender {
            crate::player::Gender::Male => 'M',
            crate::player::Gender::Female => 'F',
        };
        let birth_date = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            points,
            birth_date,
            mhp: player.max_hp,
            chp: player.hp.max(0),
            dungeon_depth: current_depth,
            level: player.level,
            deepest_dungeon_depth: player.max_depth_reached,
            gender,
            race: format!("{:?}", player.race),
            character_class: format!("{:?}", player.class),
            name: player.name.clone(),
            died_from: clean_died_from(died_from),
        }
    }
}

pub fn load_high_scores<P: AsRef<Path>>(path: P) -> Vec<HighScoreEntry> {
    if let Ok(content) = fs::read_to_string(path)
        && let Ok(entries) = serde_json::from_str::<Vec<HighScoreEntry>>(&content)
    {
        return entries;
    }
    Vec::new()
}

pub fn save_high_scores<P: AsRef<Path>>(
    entries: &[HighScoreEntry],
    path: P,
) -> Result<(), std::io::Error> {
    let serialized = serde_json::to_string_pretty(entries)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, serialized)
}

pub fn record_high_score<P: AsRef<Path>>(
    new_entry: HighScoreEntry,
    path: P,
) -> Result<usize, std::io::Error> {
    let mut scores = load_high_scores(&path);

    // Exact duplicate check
    if let Some(pos) = scores.iter().position(|e| {
        e.name == new_entry.name
            && e.points == new_entry.points
            && e.level == new_entry.level
            && e.deepest_dungeon_depth == new_entry.deepest_dungeon_depth
            && e.birth_date == new_entry.birth_date
            && e.died_from == new_entry.died_from
    }) {
        return Ok(pos + 1);
    }

    // Duplicate check: if same birth_date, gender, race, class, and died_from == "(saved)"
    if let Some(existing_idx) = scores.iter().position(|e| {
        e.birth_date == new_entry.birth_date
            && e.gender == new_entry.gender
            && e.race == new_entry.race
            && e.character_class == new_entry.character_class
            && e.died_from == "(saved)"
    }) {
        scores[existing_idx] = new_entry;
        scores.sort_by_key(|a| std::cmp::Reverse(a.points));
    } else {
        let insert_pos = scores
            .iter()
            .position(|e| new_entry.points > e.points)
            .unwrap_or(scores.len());
        scores.insert(insert_pos, new_entry);
    }

    if scores.len() > MAX_HIGH_SCORE_ENTRIES {
        scores.truncate(MAX_HIGH_SCORE_ENTRIES);
    }

    let p = path.as_ref();
    save_high_scores(&scores, p)?;

    let final_pos = scores
        .iter()
        .position(|e| e.points == scores[0].points) // or position of new_entry
        .unwrap_or(0);
    Ok(final_pos + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Class, Gender, Race};

    #[test]
    fn test_high_score_entry_and_persistence() {
        let mut player =
            Player::new_with_gender("Hero", Race::Elf, Class::Mage, Gender::Female, 10, 10);
        player.exp = 5000;
        player.max_exp = 5000;
        player.gold = 2500;
        player.max_depth_reached = 15;

        let entry = HighScoreEntry::from_player(&player, 15, "a Red dragon");
        assert_eq!(entry.name, "Hero");
        assert_eq!(entry.gender, 'F');
        assert_eq!(entry.died_from, "Red dragon");
        assert!(entry.points > 5000);

        let temp_dir = std::env::temp_dir();
        let scores_file = temp_dir.join(format!("test_scores_{}.json", entry.birth_date));
        let rank = record_high_score(entry.clone(), &scores_file).unwrap();
        assert_eq!(rank, 1);

        let loaded = load_high_scores(&scores_file);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Hero");

        let _ = fs::remove_file(scores_file);
    }

    #[test]
    fn test_clean_died_from_prefixes() {
        assert_eq!(clean_died_from("a red dragon"), "red dragon");
        assert_eq!(
            clean_died_from("an ancient red dragon"),
            "ancient red dragon"
        );
        assert_eq!(clean_died_from("starvation"), "starvation");
        assert_eq!(clean_died_from("  a cave orc  "), "cave orc");
        assert_eq!(clean_died_from("Quitting"), "Quitting");
    }

    #[test]
    fn test_canonical_scoring_calculation() {
        let mut player = Player::new("Scorer", Race::Human, Class::Warrior, 10, 10);
        player.exp = 1000;
        player.max_exp = 1200;
        player.max_depth_reached = 10;
        player.gold = 500; // 500 / 100 = 5
        let current_depth = 5; // 5 * 50 = 250
        // Expected: 1200 + 100 * 10 + 5 + 0 + 250 = 2455
        let points = player.calculate_total_points(current_depth);
        assert_eq!(points, 2455);
    }

    #[test]
    fn test_high_scores_duplicate_rejection_and_sorting() {
        let temp_dir = std::env::temp_dir();
        let scores_file = temp_dir.join("test_scores_dup.json");
        let _ = fs::remove_file(&scores_file);

        let mut p1 = Player::new("Alpha", Race::Human, Class::Warrior, 10, 10);
        p1.exp = 1000;
        let e1 = HighScoreEntry::from_player(&p1, 5, "orc");

        let mut p2 = Player::new("Beta", Race::Elf, Class::Mage, 10, 10);
        p2.exp = 5000;
        let e2 = HighScoreEntry::from_player(&p2, 10, "dragon");

        // Record e1 then e2
        let rank1 = record_high_score(e1.clone(), &scores_file).unwrap();
        assert_eq!(rank1, 1);
        let rank2 = record_high_score(e2.clone(), &scores_file).unwrap();
        assert_eq!(rank2, 1); // e2 has more points so rank 1

        // Recording duplicate e1 should not add a duplicate entry
        let _ = record_high_score(e1.clone(), &scores_file);
        let loaded = load_high_scores(&scores_file);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Beta");
        assert_eq!(loaded[1].name, "Alpha");

        let _ = fs::remove_file(scores_file);
    }
}
