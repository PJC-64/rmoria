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
use rand::Rng;
use input::{InputMapper, KeyboardProfile, Action, Direction};
use dungeon::{DungeonLevel, TileType};
use player::{Player, Race, Class, Item, ItemType};
use save::GameState;
use dice::Dice;
use entity::monster::Monster;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMode {
    Dungeon,
    InventoryList,
    EquipmentList,
    WearMenu,
    TakeOffMenu,
    QuaffMenu,
    ReadMenu,
}

fn get_lhs_stat_line(y: usize, player: &Player, level: &DungeonLevel) -> String {
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
        17 => format!("AC:   {:>5}", player.calculate_ac()),
        18 => format!("GOLD: {:>5}", player.gold),
        20 => {
            let feet = level.depth * 50;
            if feet == 0 {
                "Town Level".to_string()
            } else {
                format!("{} feet", feet)
            }
        }
        _ => "".to_string(),
    };
    
    format!("{:<12}", text)
}

fn get_overlay_row_text(row: usize, mode: ScreenMode, player: &Player) -> String {
    match mode {
        ScreenMode::InventoryList => {
            if row == 0 {
                return "--- INVENTORY LIST ---".to_string();
            }
            if row == player.inventory.len() + 2 {
                return "----------------------".to_string();
            }
            if row == player.inventory.len() + 3 {
                return "Press ESC or 'i' to return to dungeon.".to_string();
            }
            if row > 0 && row <= player.inventory.len() {
                let idx = row - 1;
                let item = &player.inventory[idx];
                let details = match &item.item_type {
                    ItemType::Weapon { damage } => format!("Weapon, {}d{} dmg", damage.num, damage.sides),
                    ItemType::Armor { ac } => format!("Armor, +{} AC", ac),
                    ItemType::Potion { heal_amount } => format!("Potion (heals {} HP)", heal_amount),
                    ItemType::Scroll { .. } => "Scroll".to_string(),
                };
                return format!("{}. {} ({}) x{}", (b'a' + idx as u8) as char, item.name, details, item.count);
            }
            "".to_string()
        }
        ScreenMode::EquipmentList => {
            if row == 0 {
                return "--- EQUIPMENT LIST ---".to_string();
            }
            if row == player.equipment.len() + 2 {
                return "----------------------".to_string();
            }
            if row == player.equipment.len() + 3 {
                return "Press ESC or 'I' to return to dungeon.".to_string();
            }
            if row > 0 && row <= player.equipment.len() {
                let idx = row - 1;
                let item = &player.equipment[idx];
                let slot = match item.item_type {
                    ItemType::Weapon { .. } => "Weapon",
                    ItemType::Armor { .. } => "Armor",
                    _ => "Accessory",
                };
                return format!("{}. {}: {}", (b'a' + idx as u8) as char, slot, item.name);
            }
            if player.equipment.is_empty() && row == 1 {
                return "(Nothing equipped)".to_string();
            }
            "".to_string()
        }
        ScreenMode::WearMenu => {
            if row == 0 {
                return "--- WEAR / WIELD SELECTION ---".to_string();
            }
            let equippable: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Weapon {..} | ItemType::Armor {..}))
                .collect();

            if row == equippable.len() + 2 {
                return "------------------------------".to_string();
            }
            if row == equippable.len() + 3 {
                return "Select item letter to equip, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= equippable.len() {
                let idx = row - 1;
                let (_, item) = equippable[idx];
                let details = match &item.item_type {
                    ItemType::Weapon { damage } => format!("Weapon, {}d{} dmg", damage.num, damage.sides),
                    ItemType::Armor { ac } => format!("Armor, +{} AC", ac),
                    _ => "".to_string(),
                };
                return format!("{}. {} ({})", (b'a' + idx as u8) as char, item.name, details);
            }
            if equippable.is_empty() && row == 1 {
                return "(No equippable weapons or armors in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::TakeOffMenu => {
            if row == 0 {
                return "--- TAKE OFF SELECTION ---".to_string();
            }
            if row == player.equipment.len() + 2 {
                return "--------------------------".to_string();
            }
            if row == player.equipment.len() + 3 {
                return "Select item letter to take off, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= player.equipment.len() {
                let idx = row - 1;
                let item = &player.equipment[idx];
                return format!("{}. {}", (b'a' + idx as u8) as char, item.name);
            }
            if player.equipment.is_empty() && row == 1 {
                return "(Nothing equipped to take off)".to_string();
            }
            "".to_string()
        }
        ScreenMode::QuaffMenu => {
            if row == 0 {
                return "--- QUAFF POTION SELECTION ---".to_string();
            }
            let potions: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Potion {..}))
                .collect();

            if row == potions.len() + 2 {
                return "------------------------------".to_string();
            }
            if row == potions.len() + 3 {
                return "Select potion letter to drink, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= potions.len() {
                let idx = row - 1;
                let (_, item) = potions[idx];
                if let ItemType::Potion { heal_amount } = item.item_type {
                    return format!("{}. {} (heals {} HP) x{}", (b'a' + idx as u8) as char, item.name, heal_amount, item.count);
                }
            }
            if potions.is_empty() && row == 1 {
                return "(No potions in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::ReadMenu => {
            if row == 0 {
                return "--- READ SCROLL SELECTION ---".to_string();
            }
            let scrolls: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Scroll {..}))
                .collect();

            if row == scrolls.len() + 2 {
                return "-----------------------------".to_string();
            }
            if row == scrolls.len() + 3 {
                return "Select scroll letter to read, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= scrolls.len() {
                let idx = row - 1;
                let (_, item) = scrolls[idx];
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.name, item.count);
            }
            if scrolls.is_empty() && row == 1 {
                return "(No scrolls in inventory)".to_string();
            }
            "".to_string()
        }
        _ => "".to_string(),
    }
}

fn draw_map(
    level: &DungeonLevel,
    player: &Player,
    monsters: &[Monster],
    status_msg: &str,
    mode: ScreenMode,
) -> Result<(), io::Error> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
    
    let mut screen_buf = String::new();
    
    // Row 0: Message line
    screen_buf.push_str(&format!("Message: {}\x1b[K\r\n", status_msg));

    // Rows 1 to 22: LHS Stats + Space Divider + Dungeon map row or overlay row
    for map_y in 0..level.height {
        let stats_part = get_lhs_stat_line(map_y + 1, player, level);
        screen_buf.push_str(&stats_part);
        
        screen_buf.push(' ');

        if mode == ScreenMode::Dungeon {
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
        } else {
            let overlay_row = get_overlay_row_text(map_y, mode, player);
            screen_buf.push_str(&format!("{:<66}", overlay_row));
        }
        screen_buf.push_str("\r\n");
    }
    
    screen_buf.push_str("-------------------------------------------------------------------------------\r\n");
    
    print!("{}", screen_buf);
    io::stdout().flush()?;
    Ok(())
}

fn generate_monsters_for_depth(depth: u32) -> Vec<Monster> {
    if depth == 0 {
        return Vec::new();
    }
    
    let mut mons = vec![
        Monster::new("Red Mold", 'm', 20, 5, 5, Dice::new(1, 3)),
        Monster::new("Goblin", 'g', 45, 12, 8, Dice::new(1, 4)),
        Monster::new("Orc", 'o', 15, 18, 12, Dice::new(1, 6)),
    ];
    
    if depth >= 3 {
        mons.push(Monster::new("Cave Troll", 'T', 50, 16, 25, Dice::new(2, 6)));
    }
    
    mons
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = "save.json";
    let mut rng = rand::thread_rng();
    
    // 1. Initialize terminal raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 2. Load game state if save file exists, otherwise create new
    let (mut player, mut level, mut monsters, mut status_msg) = if Path::new(save_path).exists() {
        match GameState::load_from_file(save_path) {
            Ok(state) => {
                (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string())
            }
            Err(e) => {
                let p = Player::new("Hero", Race::Human, Class::Warrior, 30, 10);
                let mut lvl = DungeonLevel::new(66, 22, 0);
                lvl.generate_simple_floor();
                let mons = generate_monsters_for_depth(0);
                (p, lvl, mons, format!("Failed to load save: {}. Started new game.", e))
            }
        }
    } else {
        let p = Player::new("Hero", Race::Human, Class::Warrior, 30, 10);
        let mut lvl = DungeonLevel::new(66, 22, 0);
        lvl.generate_simple_floor();
        let mons = generate_monsters_for_depth(0);
        (p, lvl, mons, "New game started! Find stairs down (>) to enter the dungeon.".to_string())
    };

    let mut screen_mode = ScreenMode::Dungeon;
    // Render initial view
    draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    'game_loop: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // Ctrl+C override
                if key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    break 'game_loop;
                }

                let mut player_acted = false;

                if screen_mode == ScreenMode::Dungeon {
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

                                if let Some(m_idx) = monsters.iter().position(|m| m.x == next_x && m.y == next_y) {
                                    let damage = player.roll_melee_damage(&mut rng);
                                    status_msg = format!("You hit {} for {} damage!", monsters[m_idx].name, damage);
                                    
                                    if monsters[m_idx].take_damage(damage) {
                                        status_msg.push_str(&format!(" You killed {}!", monsters[m_idx].name));
                                        monsters.remove(m_idx);
                                    }
                                    player_acted = true;
                                } else {
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
                            Action::GoUpStairs => {
                                if let Some(tile) = level.get_tile(player.x, player.y) {
                                    if tile.tile_type == TileType::StairsUp && level.depth > 0 {
                                        let new_depth = level.depth - 1;
                                        level = DungeonLevel::new(66, 22, new_depth);
                                        level.generate_simple_floor();
                                        monsters = generate_monsters_for_depth(new_depth);
                                        player.move_to(40, 15);
                                        status_msg = format!("You climb up to depth {} ({} feet).", new_depth, new_depth * 50);
                                        player_acted = true;
                                    } else {
                                        status_msg = "You see no up stairs (<) here.".to_string();
                                    }
                                }
                            }
                            Action::GoDownStairs => {
                                if let Some(tile) = level.get_tile(player.x, player.y) {
                                    if tile.tile_type == TileType::StairsDown {
                                        let new_depth = level.depth + 1;
                                        level = DungeonLevel::new(66, 22, new_depth);
                                        level.generate_simple_floor();
                                        monsters = generate_monsters_for_depth(new_depth);
                                        player.move_to(15, 15);
                                        status_msg = format!("You climb down to depth {} ({} feet).", new_depth, new_depth * 50);
                                        player_acted = true;
                                    } else {
                                        status_msg = "You see no down stairs (>) here.".to_string();
                                    }
                                }
                            }
                            Action::InventoryList => {
                                screen_mode = ScreenMode::InventoryList;
                                status_msg = "Inspecting inventory.".to_string();
                            }
                            Action::EquipmentList => {
                                screen_mode = ScreenMode::EquipmentList;
                                status_msg = "Inspecting equipment.".to_string();
                            }
                            Action::WearWield => {
                                screen_mode = ScreenMode::WearMenu;
                                status_msg = "Equip item from inventory.".to_string();
                            }
                            Action::TakeOff => {
                                screen_mode = ScreenMode::TakeOffMenu;
                                status_msg = "Select item to unequip.".to_string();
                            }
                            Action::Quaff => {
                                screen_mode = ScreenMode::QuaffMenu;
                                status_msg = "Select potion to quaff.".to_string();
                            }
                            Action::ReadScroll => {
                                screen_mode = ScreenMode::ReadMenu;
                                status_msg = "Select scroll to read.".to_string();
                            }
                            Action::Quit => {
                                status_msg = "Saving game and quitting...".to_string();
                                draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;
                                
                                let state = GameState::new(player.clone(), level.clone(), monsters.clone());
                                if let Err(e) = state.save_to_file(save_path) {
                                    status_msg = format!("Save failed: {}", e);
                                    draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;
                                    std::thread::sleep(std::time::Duration::from_secs(2));
                                } else {
                                    std::thread::sleep(std::time::Duration::from_millis(800));
                                }
                                break 'game_loop;
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Menu commands processing
                    match key_event.code {
                        KeyCode::Esc => {
                            screen_mode = ScreenMode::Dungeon;
                            status_msg = "Returned to dungeon.".to_string();
                        }
                        KeyCode::Char(c) => {
                            if screen_mode == ScreenMode::InventoryList && c == 'i' {
                                screen_mode = ScreenMode::Dungeon;
                                status_msg = "Returned to dungeon.".to_string();
                            } else if screen_mode == ScreenMode::EquipmentList && c == 'I' {
                                screen_mode = ScreenMode::Dungeon;
                                status_msg = "Returned to dungeon.".to_string();
                            } else if screen_mode == ScreenMode::WearMenu {
                                let idx = c as usize - 'a' as usize;
                                let equippable_indices: Vec<usize> = player.inventory.iter()
                                    .enumerate()
                                    .filter(|(_, item)| matches!(item.item_type, ItemType::Weapon {..} | ItemType::Armor {..}))
                                    .map(|(i, _)| i)
                                    .collect();

                                if idx < equippable_indices.len() {
                                    let inv_idx = equippable_indices[idx];
                                    let item = player.inventory.remove(inv_idx);
                                    let is_weapon = matches!(item.item_type, ItemType::Weapon {..});

                                    // Take off currently equipped item in the target slot
                                    let already_equipped = player.equipment.iter().position(|eq| {
                                        if is_weapon {
                                            matches!(eq.item_type, ItemType::Weapon {..})
                                        } else {
                                            matches!(eq.item_type, ItemType::Armor {..})
                                        }
                                    });

                                    if let Some(eq_idx) = already_equipped {
                                        let old = player.equipment.remove(eq_idx);
                                        status_msg = format!("You take off {} and equip {}.", old.name, item.name);
                                        player.inventory.push(old);
                                    } else {
                                        status_msg = format!("You wear/wield {}.", item.name);
                                    }

                                    player.equipment.push(item);
                                    screen_mode = ScreenMode::Dungeon;
                                    player_acted = true;
                                }
                            } else if screen_mode == ScreenMode::TakeOffMenu {
                                let idx = c as usize - 'a' as usize;
                                if idx < player.equipment.len() {
                                    let item = player.equipment.remove(idx);
                                    status_msg = format!("You took off {}.", item.name);
                                    player.inventory.push(item);
                                    screen_mode = ScreenMode::Dungeon;
                                    player_acted = true;
                                }
                            } else if screen_mode == ScreenMode::QuaffMenu {
                                let idx = c as usize - 'a' as usize;
                                let potion_indices: Vec<usize> = player.inventory.iter()
                                    .enumerate()
                                    .filter(|(_, item)| matches!(item.item_type, ItemType::Potion {..}))
                                    .map(|(i, _)| i)
                                    .collect();

                                if idx < potion_indices.len() {
                                    let inv_idx = potion_indices[idx];
                                    let heal = if let ItemType::Potion { heal_amount } = player.inventory[inv_idx].item_type {
                                        heal_amount
                                    } else {
                                        0
                                    };
                                    
                                    player.hp = (player.hp + heal).min(player.max_hp);
                                    let name = player.inventory[inv_idx].name.clone();
                                    
                                    player.inventory[inv_idx].count -= 1;
                                    if player.inventory[inv_idx].count == 0 {
                                        player.inventory.remove(inv_idx);
                                    }

                                    status_msg = format!("You quaffed {}! Restored {} HP.", name, heal);
                                    screen_mode = ScreenMode::Dungeon;
                                    player_acted = true;
                                }
                            } else if screen_mode == ScreenMode::ReadMenu {
                                let idx = c as usize - 'a' as usize;
                                let scroll_indices: Vec<usize> = player.inventory.iter()
                                    .enumerate()
                                    .filter(|(_, item)| matches!(item.item_type, ItemType::Scroll {..}))
                                    .map(|(i, _)| i)
                                    .collect();

                                if idx < scroll_indices.len() {
                                    let inv_idx = scroll_indices[idx];
                                    let name = player.inventory[inv_idx].name.clone();
                                    
                                    player.inventory[inv_idx].count -= 1;
                                    if player.inventory[inv_idx].count == 0 {
                                        player.inventory.remove(inv_idx);
                                    }

                                    // Teleport to a random floor tile
                                    let (rx, ry) = loop {
                                        let tx = rng.gen_range(1..(level.width - 1));
                                        let ty = rng.gen_range(1..(level.height - 1));
                                        if let Some(tile) = level.get_tile(tx, ty) {
                                            if tile.tile_type == TileType::Floor {
                                                break (tx, ty);
                                            }
                                        }
                                    };
                                    player.move_to(rx, ry);

                                    status_msg = format!("You read the {}! You teleport to ({}, {}).", name, rx, ry);
                                    screen_mode = ScreenMode::Dungeon;
                                    player_acted = true;
                                }
                            }
                        }
                        _ => {}
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
                                draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;
                                
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
                    draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;
                } else {
                    draw_map(&level, &player, &monsters, &status_msg, screen_mode)?;
                }
            }
        }
    }

    // 4. Restore normal terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
