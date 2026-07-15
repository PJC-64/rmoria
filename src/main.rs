mod input;
mod dungeon;
mod player;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use std::io::{self, Write};
use input::{InputMapper, KeyboardProfile, Action, Direction};
use dungeon::{DungeonLevel, TileType};
use player::Player;

fn draw_map(level: &DungeonLevel, player: &Player) -> Result<(), io::Error> {
    // Basic terminal draw loop. Position cursor at 0, 0 first.
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    
    let mut map_str = String::new();
    map_str.push_str("====================================================\r\n");
    map_str.push_str("  rmoria (Umoria Rust Rewrite) - Dungeon Map Demo   \r\n");
    map_str.push_str("  Movement: qweasdzxc | Rest: s | Quit: Q           \r\n");
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
    map_str.push_str(&format!("  Player Name: {:<12} | HP: {}/{} | Pos: ({}, {})\r\n", player.name, player.hp, player.max_hp, player.x, player.y));
    
    print!("{}", map_str);
    io::stdout().flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup terminal in raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 2. Initialize a simple dungeon level (40 wide, 15 high)
    let mut level = DungeonLevel::new(40, 15);
    // Fill interior with floor
    for y in 1..14 {
        for x in 1..39 {
            if let Some(tile) = level.get_tile_mut(x, y) {
                tile.tile_type = TileType::Floor;
            }
        }
    }
    // Add some random pillars/walls in the level
    if let Some(tile) = level.get_tile_mut(10, 5) { tile.tile_type = TileType::Wall; }
    if let Some(tile) = level.get_tile_mut(10, 6) { tile.tile_type = TileType::Wall; }
    if let Some(tile) = level.get_tile_mut(25, 8) { tile.tile_type = TileType::Wall; }
    if let Some(tile) = level.get_tile_mut(25, 9) { tile.tile_type = TileType::Wall; }

    // 3. Initialize player character
    let mut player = Player::new("Hero", 20, 7);

    // Render initial state
    draw_map(&level, &player)?;

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
                            // Calculate proposed position
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

                            // Walk bounds checking
                            if let Some(tile) = level.get_tile(next_x, next_y) {
                                if tile.is_passable() || direction == Direction::Rest {
                                    player.move_to(next_x, next_y);
                                }
                            }
                            draw_map(&level, &player)?;
                        }
                        Action::Quit => {
                            break;
                        }
                        other => {
                            // Just redraw to clean input buffer
                            draw_map(&level, &player)?;
                            execute!(io::stdout(), crossterm::cursor::MoveTo(0, 19))?;
                            println!("Action not implemented: {:?}\r", other);
                            io::stdout().flush()?;
                        }
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
