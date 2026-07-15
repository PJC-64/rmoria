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

fn draw_map(level: &DungeonLevel, player: &Player, monsters: &[Monster], status_msg: &str) -> Result<(), io::Error> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    
    let mut map_str = String::new();
    map_str.push_str("====================================================\r\n");
    map_str.push_str("  rmoria (Umoria Rust Rewrite) - Combat & AI Demo   \r\n");
    map_str.push_str("  Movement: qweasdzxc | Rest: s | Quit & Save: Q    \r\n");
    map_str.push_str("====================================================\r\n");

    for y in 0..level.height {
        for x in 0..level.width {
            if x == player.x && y == player.y {
                map_str.push('@');
            } else if let Some(monster) = monsters.iter().find(|m| m.x == x && m.y == y) {
                map_str.push(monster.symbol);
            } else if let Some(tile) = level.get_tile(x, y) {
                match tile.tile_type {
                    TileType::Wall => map_str.push('#'),
                    TileType::Floor => map_str.push('.'),
                    TileType::DoorClosed => map_str.push('+'),
                    TileType::DoorOpen => map_str.push('\''),
                    TileType::StairsUp => map_str.push('<'),
                    TileType::StairsDown => map_str.push('>'),
                    TileType::Empty => map_str.push(' '),
                }
            }
        }
        map_str.push_str("\r\n");
    }
    map_str.push_str("----------------------------------------------------\r\n");
    map_str.push_str(&format!(
        "  Name: {:<8} | Race: {:<8} | Class: {:<8}\r\n  HP: {:<4}/{} | Gold: {:<5} | Pos: ({}, {})\r\n",
        player.name, format!("{:?}", player.race), format!("{:?}", player.class),
        player.hp, player.max_hp, player.gold, player.x, player.y
    ));
    map_str.push_str("----------------------------------------------------\r\n");
    map_str.push_str(&format!("  Status: {:<40}\r\n", status_msg));
    
    print!("{}", map_str);
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
                let p = Player::new("GnomeMage", Race::Gnome, Class::Mage, 20, 7);
                let mut lvl = DungeonLevel::new(40, 15);
                for y in 1..14 {
                    for x in 1..39 {
                        if let Some(tile) = lvl.get_tile_mut(x, y) {
                            tile.tile_type = TileType::Floor;
                        }
                    }
                }
                let mons = vec![
                    Monster::new("Red Mold", 'm', 15, 5, 5, Dice::new(1, 3)),
                    Monster::new("Goblin", 'g', 25, 10, 8, Dice::new(1, 4)),
                    Monster::new("Orc", 'o', 12, 12, 12, Dice::new(1, 6)),
                ];
                (p, lvl, mons, format!("Failed to load save: {}. Started new game.", e))
            }
        }
    } else {
        let p = Player::new("GnomeMage", Race::Gnome, Class::Mage, 20, 7);
        let mut lvl = DungeonLevel::new(40, 15);
        for y in 1..14 {
            for x in 1..39 {
                if let Some(tile) = lvl.get_tile_mut(x, y) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
        let mons = vec![
            Monster::new("Red Mold", 'm', 15, 5, 5, Dice::new(1, 3)),
            Monster::new("Goblin", 'g', 25, 10, 8, Dice::new(1, 4)),
            Monster::new("Orc", 'o', 12, 12, 12, Dice::new(1, 6)),
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
                                            // Recover 1 HP
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
                        // Cache monster positions to check for collisions without double borrows
                        let occupied_positions: Vec<(usize, usize)> = monsters.iter().map(|m| (m.x, m.y)).collect();

                        for monster in monsters.iter_mut() {
                            // Check if adjacent to player
                            let dx = (player.x as isize - monster.x as isize).abs();
                            let dy = (player.y as isize - monster.y as isize).abs();
                            
                            if dx <= 1 && dy <= 1 {
                                // Monster attacks player!
                                let m_damage = monster.damage.roll(&mut rng) as i32;
                                player.hp -= m_damage;
                                status_msg.push_str(&format!(" {} hits you for {}!", monster.name, m_damage));

                                if player.hp <= 0 {
                                    player.hp = 0;
                                    status_msg.push_str(" You have died! Game Over.");
                                    draw_map(&level, &player, &monsters, &status_msg)?;
                                    
                                    // Remove save file on death
                                    if Path::new(save_path).exists() {
                                        let _ = fs::remove_file(save_path);
                                    }
                                    std::thread::sleep(std::time::Duration::from_secs(3));
                                    break 'game_loop;
                                }
                            } else {
                                // Monster moves towards player using AI pathing
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
                        // Redraw map with updated player and monster states
                        draw_map(&level, &player, &monsters, &status_msg)?;
                    } else {
                        // Redraw only for message updates
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
