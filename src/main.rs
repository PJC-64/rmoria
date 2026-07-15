mod input;
mod dungeon;
mod player;
mod save;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use std::io::{self, Write};
use std::path::Path;
use input::{InputMapper, KeyboardProfile, Action, Direction};
use dungeon::{DungeonLevel, TileType};
use player::{Player, Race, Class};
use save::GameState;

fn draw_map(level: &DungeonLevel, player: &Player, status_msg: &str) -> Result<(), io::Error> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    
    let mut map_str = String::new();
    map_str.push_str("====================================================\r\n");
    map_str.push_str("  rmoria (Umoria Rust Rewrite) - Save / Load Demo   \r\n");
    map_str.push_str("  Movement: qweasdzxc | Rest: s | Quit & Save: Q    \r\n");
    map_str.push_str("====================================================\r\n");

    for y in 0..level.height {
        for x in 0..level.width {
            if x == player.x && y == player.y {
                map_str.push('@');
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
        "  Name: {:<8} | Race: {:<8} | Class: {:<8}\r\n  HP: {}/{} | Gold: {:<5} | Pos: ({}, {})\r\n",
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
    
    // 1. Initialize terminal raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 2. Load game state if save file exists, otherwise create new
    let (mut player, mut level, mut status_msg) = if Path::new(save_path).exists() {
        match GameState::load_from_file(save_path) {
            Ok(state) => {
                (state.player, state.level, "Save game loaded successfully!".to_string())
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
                (p, lvl, format!("Failed to load save: {}. Started new game.", e))
            }
        }
    } else {
        // Create Gnome Mage
        let p = Player::new("GnomeMage", Race::Gnome, Class::Mage, 20, 7);
        let mut lvl = DungeonLevel::new(40, 15);
        for y in 1..14 {
            for x in 1..39 {
                if let Some(tile) = lvl.get_tile_mut(x, y) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
        (p, lvl, "New game started! Gnome Mage created.".to_string())
    };

    // Render initial view
    draw_map(&level, &player, &status_msg)?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // Ctrl+C override
                if key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                if let KeyCode::Char(c) = key_event.code {
                    let action = mapper.map_key(c);
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

                            if let Some(tile) = level.get_tile(next_x, next_y) {
                                if tile.is_passable() || direction == Direction::Rest {
                                    player.move_to(next_x, next_y);
                                    status_msg = format!("Moved to ({}, {})", next_x, next_y);
                                } else {
                                    status_msg = "Ouch! You bumped into a wall.".to_string();
                                }
                            }
                            draw_map(&level, &player, &status_msg)?;
                        }
                        Action::Quit => {
                            status_msg = "Saving game and quitting...".to_string();
                            draw_map(&level, &player, &status_msg)?;
                            
                            // Save state
                            let state = GameState::new(player.clone(), level.clone());
                            if let Err(e) = state.save_to_file(save_path) {
                                status_msg = format!("Save failed: {}", e);
                                draw_map(&level, &player, &status_msg)?;
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(800));
                            }
                            break;
                        }
                        other => {
                            status_msg = format!("Action not implemented: {:?}", other);
                            draw_map(&level, &player, &status_msg)?;
                        }
                    }
                }
            }
        }
    }

    // 3. Restore normal terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
