mod input;
mod dungeon;
mod player;
mod save;
mod dice;
mod entity;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use std::io::{self, Write};
use std::path::Path;
use std::fs;
use input::{InputMapper, KeyboardProfile, Action, Direction};
use dungeon::{DungeonLevel, TileType};
use player::{Player, Race, Class};
use save::GameState;
use dice::Dice;
use entity::monster::Monster;

fn get_lhs_stat_line(y: usize, player: &Player) -> String {
    let rank = match player.class {
        Class::Warrior => "Rookie",
        Class::Mage => "Apprentice",
        Class::Priest => "Believer",
        Class::Rogue => "Footpad",
        Class::Ranger => "Runner",
        Class::Paladin => "Gallant",
    };

    let text = match y {
        2 => format!("{:?}", player.race),
        3 => format!("{:?}", player.class),
        4 => format!("'{}'", rank),
        6 => format!("STR:  {:>5}", player.stats.strength),
        7 => format!("INT:  {:>5}", player.stats.intelligence),
        8 => format!("WIS:  {:>5}", player.stats.wisdom),
        9 => format!("DEX:  {:>5}", player.stats.dexterity),
        10 => format!("CON:  {:>5}", player.stats.constitution),
        11 => format!("CHR:  {:>5}", player.stats.charisma),
        13 => format!("LEV:  {:>5}", player.level),
        14 => format!("MANA: {:>5}", player.mana),
        15 => format!("MHP:  {:>5}", player.max_hp),
        16 => format!("CHP:  {:>5}", player.hp),
        17 => format!("AC:   {:>5}", 10), // Base AC
        18 => format!("GOLD: {:>5}", player.gold),
        _ => "".to_string(),
    };
    
    format!("{:<12}", text)
}

fn draw_map(level: &DungeonLevel, player: &Player, monsters: &[Monster], status_msg: &str) -> Result<(), io::Error> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    
    let mut screen_buf = String::new();
    
    // Row 0: Message line (Status/combat log, padded to 79 chars to clear previous text)
    screen_buf.push_str(&format!("Message: {:<70}\r\n", status_msg));

    // Rows 1 to 22: LHS Stats + Space Divider + Dungeon map row
    for map_y in 0..level.height {
        // 1. Get stats string (exactly 12 characters wide)
        let stats_part = get_lhs_stat_line(map_y + 1, player);
        screen_buf.push_str(&stats_part);
        
        // 2. Add the 1 space separator column
        screen_buf.push(' ');

        // 3. Add the dungeon map row (66 characters wide)
        for map_x in 0..level.width {
            if map_x == player.x && map_y == player.y {
                screen_buf.push('@');
            } else if let Some(monster) = monsters.iter().find(|m| m.x == map_x && m.y == map_y) {
                screen_buf.push(monster.symbol);
            } else if let Some(tile) = level.get_tile(map_x, map_y) {
                match tile.tile_type {
                    TileType::Wall => screen_buf.push('#'),
                    TileType::Floor => screen_buf.push('.'),
                    TileType::DoorClosed => screen_buf.push('+'),
                    TileType::DoorOpen => screen_buf.push('\''),
                    TileType::StairsUp => screen_buf.push('<'),
                    TileType::StairsDown => screen_buf.push('>'),
                    TileType::Empty => screen_buf.push(' '),
                }
            }
        }
        screen_buf.push_str("\r\n");
    }
    
    // Row 23: Bottom border
    screen_buf.push_str("-------------------------------------------------------------------------------\r\n");
    
    print!("{}", screen_buf);
    io::stdout().flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = "save.json";
    let mut rng = rand::thread_rng();
    
    // 1. Initialize terminal raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 2. Load game state if save file exists, otherwise create new
    let (mut player, level, mut monsters, mut status_msg) = if Path::new(save_path).exists() {
        match GameState::load_from_file(save_path) {
            Ok(state) => {
                (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string())
            }
            Err(e) => {
                let p = Player::new("Hero", Race::Human, Class::Warrior, 30, 10);
                let mut lvl = DungeonLevel::new(66, 22);
                for y in 1..21 {
                    for x in 1..65 {
                        if let Some(tile) = lvl.get_tile_mut(x, y) {
                            tile.tile_type = TileType::Floor;
                        }
                    }
                }
                let mons = vec![
                    Monster::new("Red Mold", 'm', 20, 5, 5, Dice::new(1, 3)),
                    Monster::new("Goblin", 'g', 45, 12, 8, Dice::new(1, 4)),
                    Monster::new("Orc", 'o', 15, 18, 12, Dice::new(1, 6)),
                ];
                (p, lvl, mons, format!("Failed to load save: {}. Started new game.", e))
            }
        }
    } else {
        let p = Player::new("Hero", Race::Human, Class::Warrior, 30, 10);
        let mut lvl = DungeonLevel::new(66, 22);
        for y in 1..21 {
            for x in 1..65 {
                if let Some(tile) = lvl.get_tile_mut(x, y) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
        let mons = vec![
            Monster::new("Red Mold", 'm', 20, 5, 5, Dice::new(1, 3)),
            Monster::new("Goblin", 'g', 45, 12, 8, Dice::new(1, 4)),
            Monster::new("Orc", 'o', 15, 18, 12, Dice::new(1, 6)),
        ];
        (p, lvl, mons, "New game started! Defeat the monsters.".to_string())
    };

    // Render initial view
    draw_map(&level, &player, &monsters, &status_msg)?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    'game_loop: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // Ctrl+C override
                if key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    break 'game_loop;
                }

                if let KeyCode::Char(c) = key_event.code {
                    let action = mapper.map_key(c);
                    let mut player_acted = false;

                    match action {
                        Action::Move(direction) => {
                            let (dx, dy) = match direction {
                                Direction::NorthWest => (-1, -1),
                                Direction::North => (0, -1),
                                Direction::NorthEast => (1, -1),
                                Direction::West => (-1, 0),
                                Direction::Rest => (0, 0),
                                Direction::East => (1, 0),
                                Direction::SouthWest => (-1, 1),
                                Direction::South => (0, 1),
                                Direction::SouthEast => (1, 1),
                            };

                            let next_x = (player.x as isize + dx) as usize;
                            let next_y = (player.y as isize + dy) as usize;

                            // 1. Check if there is a monster at the destination
                            if let Some(m_idx) = monsters.iter().position(|m| m.x == next_x && m.y == next_y) {
                                // Player attacks!
                                let damage = player.roll_melee_damage(&mut rng);
                                status_msg = format!("You hit {} for {} damage!", monsters[m_idx].name, damage);
                                
                                if monsters[m_idx].take_damage(damage) {
                                    status_msg.push_str(&format!(" You killed {}!", monsters[m_idx].name));
                                    monsters.remove(m_idx);
                                }
                                player_acted = true;
                            } else {
                                // 2. Normal movement or rest
                                if let Some(tile) = level.get_tile(next_x, next_y) {
                                    if tile.is_passable() || direction == Direction::Rest {
                                        player.move_to(next_x, next_y);
                                        if direction == Direction::Rest {
                                            player.hp = (player.hp + 1).min(player.max_hp);
                                            status_msg = "You rest and recover health.".to_string();
                                        } else {
                                            status_msg = format!("Moved to ({}, {})", next_x, next_y);
                                        }
                                        player_acted = true;
                                    } else {
                                        status_msg = "Ouch! You bumped into a wall.".to_string();
                                    }
                                }
                            }
                        }
                        Action::Quit => {
                            status_msg = "Saving game and quitting...".to_string();
                            draw_map(&level, &player, &monsters, &status_msg)?;
                            
                            let state = GameState::new(player.clone(), level.clone(), monsters.clone());
                            if let Err(e) = state.save_to_file(save_path) {
                                status_msg = format!("Save failed: {}", e);
                                draw_map(&level, &player, &monsters, &status_msg)?;
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(800));
                            }
                            break 'game_loop;
                        }
                        other => {
                            status_msg = format!("Action not implemented: {:?}", other);
                        }
                    }

                    // 3. Trigger monster turns if player did a turn-consuming action
                    if player_acted {
                        let occupied_positions: Vec<(usize, usize)> = monsters.iter().map(|m| (m.x, m.y)).collect();

                        for monster in monsters.iter_mut() {
                            let dx = (player.x as isize - monster.x as isize).abs();
                            let dy = (player.y as isize - monster.y as isize).abs();
                            
                            if dx <= 1 && dy <= 1 {
                                let m_damage = monster.damage.roll(&mut rng) as i32;
                                player.hp -= m_damage;
                                status_msg.push_str(&format!(" {} hits you for {}!", monster.name, m_damage));

                                if player.hp <= 0 {
                                    player.hp = 0;
                                    status_msg.push_str(" You have died! Game Over.");
                                    draw_map(&level, &player, &monsters, &status_msg)?;
                                    
                                    if Path::new(save_path).exists() {
                                        let _ = fs::remove_file(save_path);
                                    }
                                    std::thread::sleep(std::time::Duration::from_secs(3));
                                    break 'game_loop;
                                }
                            } else {
                                if let Some((mx, my)) = monster.update_ai(player.x, player.y, &level) {
                                    let occupied_by_player = mx == player.x && my == player.y;
                                    let occupied_by_monster = occupied_positions.iter().any(|&(ox, oy)| ox == mx && oy == my);
                                    
                                    if !occupied_by_player && !occupied_by_monster {
                                        monster.x = mx;
                                        monster.y = my;
                                    }
                                }
                            }
                        }
                        draw_map(&level, &player, &monsters, &status_msg)?;
                    } else {
                        draw_map(&level, &player, &monsters, &status_msg)?;
                    }
                }
            }
        }
    }

    // 4. Restore normal terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
