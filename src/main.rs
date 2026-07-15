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
use dungeon::{DungeonLevel, TileType, ShopType};
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
    Shop,
    ShopSellMenu,
}

struct MonsterTemplate {
    name: &'static str,
    symbol: char,
    level: u32,
    max_hp: i32,
    damage: Dice,
    exp_reward: u32,
}

const MONSTER_DB: &[MonsterTemplate] = &[
    // Level 1
    MonsterTemplate { name: "Red Mold", symbol: 'm', level: 1, max_hp: 5, damage: Dice { num: 1, sides: 3 }, exp_reward: 5 },
    MonsterTemplate { name: "Giant Centipede", symbol: 'c', level: 1, max_hp: 7, damage: Dice { num: 1, sides: 3 }, exp_reward: 7 },
    // Level 2
    MonsterTemplate { name: "Goblin", symbol: 'g', level: 2, max_hp: 8, damage: Dice { num: 1, sides: 4 }, exp_reward: 10 },
    MonsterTemplate { name: "Giant White Rat", symbol: 'r', level: 2, max_hp: 9, damage: Dice { num: 1, sides: 4 }, exp_reward: 12 },
    // Level 3
    MonsterTemplate { name: "Orc", symbol: 'o', level: 3, max_hp: 15, damage: Dice { num: 1, sides: 6 }, exp_reward: 25 },
    MonsterTemplate { name: "Baby Dragon", symbol: 'd', level: 3, max_hp: 20, damage: Dice { num: 2, sides: 4 }, exp_reward: 35 },
    // Level 4
    MonsterTemplate { name: "Cave Troll", symbol: 'T', level: 4, max_hp: 30, damage: Dice { num: 2, sides: 6 }, exp_reward: 50 },
    MonsterTemplate { name: "Spectre", symbol: 'S', level: 4, max_hp: 22, damage: Dice { num: 1, sides: 8 }, exp_reward: 60 },
    // Level 5
    MonsterTemplate { name: "Uruk-Hai", symbol: 'U', level: 5, max_hp: 35, damage: Dice { num: 2, sides: 6 }, exp_reward: 80 },
    MonsterTemplate { name: "Giant Spider", symbol: 's', level: 5, max_hp: 28, damage: Dice { num: 1, sides: 10 }, exp_reward: 75 },
    // Level 6
    MonsterTemplate { name: "Ghost Warrior", symbol: 'W', level: 6, max_hp: 40, damage: Dice { num: 2, sides: 8 }, exp_reward: 120 },
    MonsterTemplate { name: "Lich", symbol: 'L', level: 7, max_hp: 55, damage: Dice { num: 3, sides: 6 }, exp_reward: 180 },
];

fn get_shop_items(shop: ShopType) -> Vec<(&'static str, u32, ItemType)> {
    match shop {
        ShopType::General => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Dagger", 50, ItemType::Weapon { damage: Dice::new(1, 4) }),
            ("Leather Armor", 80, ItemType::Armor { ac: 4 }),
        ],
        ShopType::Armory => vec![
            ("Leather Armor", 80, ItemType::Armor { ac: 4 }),
            ("Chain Mail", 250, ItemType::Armor { ac: 7 }),
            ("Iron Shield", 150, ItemType::Armor { ac: 3 }),
        ],
        ShopType::Weaponsmith => vec![
            ("Dagger", 50, ItemType::Weapon { damage: Dice::new(1, 4) }),
            ("Short Sword", 120, ItemType::Weapon { damage: Dice::new(1, 6) }),
            ("Broadsword", 350, ItemType::Weapon { damage: Dice::new(2, 5) }),
        ],
        ShopType::Temple => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Potion of Healing", 100, ItemType::Potion { heal_amount: 25 }),
        ],
        ShopType::Alchemy => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
        ],
        ShopType::Magic => vec![
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Scroll of Teleportation", 60, ItemType::Scroll { teleport: true }),
        ],
    }
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

fn get_overlay_row_text(row: usize, mode: ScreenMode, player: &Player, shop: ShopType) -> String {
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
                    ItemType::Scroll { .. } => {
                        if item.name.contains("Torch") {
                            "Light Source".to_string()
                        } else {
                            "Scroll".to_string()
                        }
                    }
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
                    _ => {
                        if item.name.contains("Torch") || item.name.contains("Lantern") {
                            "Light Source"
                        } else {
                            "Accessory"
                        }
                    }
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
                .filter(|(_, item)| {
                    matches!(item.item_type, ItemType::Weapon {..} | ItemType::Armor {..}) ||
                    item.name.contains("Torch") || item.name.contains("Lantern")
                })
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
                    _ => "Light Source (radius 2)".to_string(),
                };
                return format!("{}. {} ({})", (b'a' + idx as u8) as char, item.name, details);
            }
            if equippable.is_empty() && row == 1 {
                return "(No equippable weapons, armors or lights in inventory)".to_string();
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
                .filter(|(_, item)| matches!(item.item_type, ItemType::Scroll {..}) && !item.name.contains("Torch"))
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
        ScreenMode::Shop => {
            let shop_name = match shop {
                ShopType::General => "GENERAL STORE",
                ShopType::Armory => "TOWN ARMORY",
                ShopType::Weaponsmith => "WEAPONSMITH FORGE",
                ShopType::Temple => "TOWN TEMPLE",
                ShopType::Alchemy => "ALCHEMY LAB",
                ShopType::Magic => "MAGIC-USER CONCLAVE",
            };
            if row == 0 {
                return format!("--- {} (BUY) ---", shop_name);
            }
            if row == 1 {
                return format!("Your Gold: {} gp", player.gold);
            }
            let items = get_shop_items(shop);
            if row >= 3 && row < items.len() + 3 {
                let idx = row - 3;
                let (name, price, _) = &items[idx];
                return format!("{}. {:<32}  -  {} gp", (b'a' + idx as u8) as char, name, price);
            }
            if row == items.len() + 4 {
                return "--------------------------------".to_string();
            }
            if row == items.len() + 5 {
                return "Press 's' to Sell items, or press ESC to exit store.".to_string();
            }
            "".to_string()
        }
        ScreenMode::ShopSellMenu => {
            if row == 0 {
                return "--- TOWN GENERAL STORE (SELL) ---".to_string();
            }
            if row == 1 {
                return format!("Your Gold: {} gp", player.gold);
            }
            if row == player.inventory.len() + 3 {
                return "---------------------------------".to_string();
            }
            if row == player.inventory.len() + 4 {
                return "Select item letter to sell, or press ESC to return to Buy menu.".to_string();
            }
            if row > 1 && row <= player.inventory.len() + 1 {
                let idx = row - 2;
                let item = &player.inventory[idx];
                let value = match item.item_type {
                    ItemType::Weapon {..} => 25,
                    ItemType::Armor {..} => 40,
                    ItemType::Potion {..} => 15,
                    ItemType::Scroll {..} => 10,
                };
                return format!("{}. {} (sells for {} gp) x{}", (b'a' + idx as u8) as char, item.name, value, item.count);
            }
            if player.inventory.is_empty() && row == 2 {
                return "(Your inventory is completely empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::Dungeon => "".to_string(),
    }
}

fn has_los(x1: usize, y1: usize, x2: usize, y2: usize, level: &DungeonLevel) -> bool {
    let dx = (x2 as isize - x1 as isize).abs();
    let dy = (y2 as isize - y1 as isize).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    
    let mut cx = x1 as isize;
    let mut cy = y1 as isize;
    
    loop {
        if cx == x2 as isize && cy == y2 as isize {
            return true;
        }
        // Wall or Closed Door blocks LOS light
        if cx != x1 as isize || cy != y1 as isize {
            if let Some(tile) = level.get_tile(cx as usize, cy as usize) {
                if tile.tile_type == TileType::Wall || tile.tile_type == TileType::DoorClosed {
                    return false;
                }
            }
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

fn update_visibility(level: &mut DungeonLevel, player: &Player) {
    // 1. Reset all visible flags
    for tile in level.tiles.iter_mut() {
        tile.visible = false;
    }
    
    // 2. Town Level (depth 0) is fully lit
    if level.depth == 0 {
        for tile in level.tiles.iter_mut() {
            tile.visible = true;
            tile.remembered = true;
        }
        return;
    }
    
    // 3. Lit room checks: If player is inside a lit room, reveal the entire room (including 1-tile boundary walls)
    let mut lit_rooms_to_reveal = Vec::new();
    for room in &level.rooms {
        if room.is_lit {
            let player_inside = player.x >= room.x && player.x < room.x + room.w &&
                                player.y >= room.y && player.y < room.y + room.h;
            if player_inside {
                lit_rooms_to_reveal.push(room.clone());
            }
        }
    }
    
    // 4. Dungeon Level light calculations (LOS raycast within radius limit)
    let radius = player.get_light_radius();
    let max_dist = 8; // Memory limit distance
    
    for y in 0..level.height {
        for x in 0..level.width {
            // Check if this tile falls within an illuminated lit room
            let mut is_revealed_by_room = false;
            for room in &lit_rooms_to_reveal {
                if x >= room.x - 1 && x <= room.x + room.w &&
                   y >= room.y - 1 && y <= room.y + room.h {
                    is_revealed_by_room = true;
                    break;
                }
            }

            if is_revealed_by_room {
                if let Some(tile) = level.get_tile_mut(x, y) {
                    tile.visible = true;
                    tile.remembered = true;
                }
                continue;
            }

            let dx = (x as isize - player.x as isize).abs();
            let dy = (y as isize - player.y as isize).abs();
            let dist = dx.max(dy) as usize; // Chebyshev distance
            
            if dist <= max_dist {
                if has_los(player.x, player.y, x, y, level) {
                    if dist <= radius {
                        if let Some(tile) = level.get_tile_mut(x, y) {
                            tile.visible = true;
                            tile.remembered = true;
                        }
                    } else {
                        // Explored but in shadow
                        if let Some(tile) = level.get_tile_mut(x, y) {
                            tile.remembered = true;
                        }
                    }
                }
            }
        }
    }
}

fn draw_map(
    level: &mut DungeonLevel,
    player: &Player,
    monsters: &[Monster],
    status_msg: &str,
    mode: ScreenMode,
    shop: ShopType,
) -> Result<(), io::Error> {
    // Run visibility calculations before rendering
    update_visibility(level, player);

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
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        screen_buf.push('@');
                    } else if tile.visible && monsters.iter().any(|m| m.x == map_x && m.y == map_y) {
                        // Only render monsters on tiles that are actively lit/visible!
                        let monster = monsters.iter().find(|m| m.x == map_x && m.y == map_y).unwrap();
                        screen_buf.push(monster.symbol);
                    } else if tile.visible || tile.remembered {
                        // Render visible or remembered tiles
                        match tile.tile_type {
                            TileType::Wall => screen_buf.push('#'),
                            TileType::Floor => screen_buf.push('.'),
                            TileType::DoorClosed => screen_buf.push('+'),
                            TileType::DoorOpen => screen_buf.push('\''),
                            TileType::StairsUp => screen_buf.push('<'),
                            TileType::StairsDown => screen_buf.push('>'),
                            TileType::Empty => screen_buf.push(' '),
                        }
                    } else {
                        // Unexplored darkness shroud
                        screen_buf.push(' ');
                    }
                }
            }
        } else {
            let overlay_row = get_overlay_row_text(map_y, mode, player, shop);
            screen_buf.push_str(&format!("{:<66}", overlay_row));
        }
        screen_buf.push_str("\r\n");
    }
    
    screen_buf.push_str("-------------------------------------------------------------------------------\r\n");
    
    print!("{}", screen_buf);
    io::stdout().flush()?;
    Ok(())
}

fn find_passable_tile(level: &DungeonLevel) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let tx = rng.gen_range(1..(level.width - 1));
        let ty = rng.gen_range(1..(level.height - 1));
        if let Some(tile) = level.get_tile(tx, ty) {
            if tile.tile_type == TileType::Floor {
                return (tx, ty);
            }
        }
    }
    (30, 10)
}

fn generate_monsters_for_depth(level: &DungeonLevel, player_has_killed_balrog: bool) -> Vec<Monster> {
    let depth = level.depth;
    let max_depth = level.max_depth;
    
    if depth == 0 {
        // Town street monsters: give 0 experience!
        let (u_x, u_y) = find_passable_tile(level);
        let (r_x, r_y) = find_passable_tile(level);
        let (b_x, b_y) = find_passable_tile(level);
        return vec![
            Monster::new("Filthy Street Urchin", 'u', u_x, u_y, 6, Dice::new(1, 2), 0),
            Monster::new("Tavern Ruffian", 'r', r_x, r_y, 10, Dice::new(1, 4), 0),
            Monster::new("Beggar", 'b', b_x, b_y, 4, Dice::new(1, 1), 0),
        ];
    }
    
    let mut mons = Vec::new();
    let mut rng = rand::thread_rng();
    
    // Bottom Level = Boss Level
    if depth == max_depth {
        if !player_has_killed_balrog {
            let (bx, by) = find_passable_tile(level);
            mons.push(Monster::new("The Balrog", 'B', bx, by, 120, Dice::new(3, 8), 500));
        }
        
        // Spawn 3 high-level guards (Liches)
        for _ in 0..3 {
            let (gx, gy) = find_passable_tile(level);
            mons.push(Monster::new("Lich Guardian", 'L', gx, gy, 55, Dice::new(3, 6), 180));
        }
        return mons;
    }
    
    // Dungeon Levels scale selection
    let mut active_level = depth;
    
    // 10% chance of "nasty" monster spawning offset
    if rng.gen_bool(0.10) {
        active_level += rng.gen_range(1..=3);
    }
    
    // Spawn 4 monsters matching the active level cap
    for _ in 0..4 {
        let suitable_templates: Vec<&MonsterTemplate> = MONSTER_DB.iter()
            .filter(|t| t.level <= active_level)
            .collect();
            
        if !suitable_templates.is_empty() {
            let r_idx = rng.gen_range(0..suitable_templates.len());
            let t = suitable_templates[r_idx];
            let (mx, my) = find_passable_tile(level);
            mons.push(Monster::new(t.name, t.symbol, mx, my, t.max_hp, t.damage, t.exp_reward));
        }
    }
    
    mons
}

fn add_item_to_inventory(inventory: &mut Vec<Item>, item: Item) {
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == item.name && i.item_type == item.item_type) {
        existing.count += item.count;
    } else {
        inventory.push(item);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = "save.json";
    let mut rng = rand::thread_rng();
    
    // 1. Initialize terminal raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // 2. Load game state if save file exists, otherwise create new
    let (mut player, mut level, mut monsters, mut status_msg, max_depth) = if Path::new(save_path).exists() {
        match GameState::load_from_file(save_path) {
            Ok(state) => {
                (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string(), state.max_depth)
            }
            Err(e) => {
                let rolled_max = rng.gen_range(8..=15);
                let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                lvl.generate_simple_floor();
                let start_pos = find_passable_tile(&lvl);
                let p = Player::new("Hero", Race::Human, Class::Warrior, start_pos.0, start_pos.1);
                let mons = generate_monsters_for_depth(&lvl, false);
                (p, lvl, mons, format!("Failed to load save: {}. Started new game.", e), rolled_max)
            }
        }
    } else {
        let rolled_max = rng.gen_range(8..=15);
        let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
        lvl.generate_simple_floor();
        let start_pos = find_passable_tile(&lvl);
        let p = Player::new("Hero", Race::Human, Class::Warrior, start_pos.0, start_pos.1);
        let mons = generate_monsters_for_depth(&lvl, false);
        (p, lvl, mons, "New game started! Find town shops or descend stairs (>)".to_string(), rolled_max)
    };

    let mut screen_mode = ScreenMode::Dungeon;
    let mut active_shop = ShopType::General;

    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    'game_loop: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
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

                                // Check if there is a store door on Town level (depth 0)
                                let mut entered_shop = false;
                                if level.depth == 0 {
                                    if let Some(shop_info) = level.shops.iter().find(|s| s.door_x == next_x && s.door_y == next_y) {
                                        screen_mode = ScreenMode::Shop;
                                        active_shop = shop_info.shop_type;
                                        let shop_name = match active_shop {
                                            ShopType::General => "Town General Store",
                                            ShopType::Armory => "Town Armory",
                                            ShopType::Weaponsmith => "Weaponsmith Forge",
                                            ShopType::Temple => "Town Temple",
                                            ShopType::Alchemy => "Alchemy Lab",
                                            ShopType::Magic => "Magic-User Conclave",
                                        };
                                        status_msg = format!("Welcome to the {}!", shop_name);
                                        entered_shop = true;
                                    }
                                }

                                if entered_shop {
                                    // Shop entered
                                }
                                // Check if there is a monster at the destination (must be visible to attack!)
                                else if let Some(m_idx) = monsters.iter().position(|m| m.x == next_x && m.y == next_y) {
                                    // Check if the tile the monster stands on is visible (lit) to player
                                    let is_m_visible = level.get_tile(next_x, next_y).map(|t| t.visible).unwrap_or(false);
                                    
                                    if is_m_visible {
                                        let damage = player.roll_melee_damage(&mut rng);
                                        status_msg = format!("You hit {} for {} damage!", monsters[m_idx].name, damage);
                                        
                                        let monster_name = monsters[m_idx].name.clone();
                                        let exp_reward = monsters[m_idx].experience_reward;

                                        if monsters[m_idx].take_damage(damage) {
                                            status_msg.push_str(&format!(" You killed {}!", monster_name));
                                            
                                            if player.add_experience(exp_reward) {
                                                status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                            } else if exp_reward > 0 {
                                                status_msg.push_str(&format!(" Gained {} EXP.", exp_reward));
                                            } else {
                                                status_msg.push_str(" Gained 0 EXP (Town monster).");
                                            }
                                            
                                            if monster_name == "The Balrog" {
                                                player.balrog_killed = true;
                                                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                                                std::thread::sleep(std::time::Duration::from_secs(1));
                                                
                                                execute!(stdout, Show, LeaveAlternateScreen)?;
                                                disable_raw_mode()?;
                                                
                                                println!("============================================================");
                                                println!("         CONGRATULATIONS! YOU HAVE SLAIN THE BALROG!        ");
                                                println!("        You have completed the quest and won rmoria!        ");
                                                println!("============================================================");
                                                
                                                if Path::new(save_path).exists() {
                                                    let _ = fs::remove_file(save_path);
                                                }
                                                break 'game_loop;
                                            }
                                            monsters.remove(m_idx);
                                        }
                                        player_acted = true;
                                    } else {
                                        // Stepping into dark tile containing monster -> bump/attack in the dark!
                                        if let Some(tile) = level.get_tile(next_x, next_y) {
                                            if tile.is_passable() {
                                                player.move_to(next_x, next_y);
                                                status_msg = "You step into the darkness and bump into a monster!".to_string();
                                                player_acted = true;
                                            }
                                        }
                                    }
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
                                        level = DungeonLevel::new(66, 22, new_depth, max_depth);
                                        level.generate_simple_floor();
                                        monsters = generate_monsters_for_depth(&level, player.balrog_killed);
                                        
                                        let sd_pos = level.tiles.iter().enumerate()
                                            .find(|(_, t)| t.tile_type == TileType::StairsDown)
                                            .map(|(i, _)| (i % level.width, i / level.width))
                                            .unwrap_or((40, 15));
                                        player.move_to(sd_pos.0, sd_pos.1);
                                        
                                        status_msg = format!("You climb up to depth {} ({} feet).", new_depth, new_depth * 50);
                                        player_acted = true;
                                    } else {
                                        status_msg = "You see no up stairs (<) here.".to_string();
                                    }
                                }
                            }
                            Action::GoDownStairs => {
                                if let Some(tile) = level.get_tile(player.x, player.y) {
                                    if tile.tile_type == TileType::StairsDown && level.depth < max_depth {
                                        let new_depth = level.depth + 1;
                                        level = DungeonLevel::new(66, 22, new_depth, max_depth);
                                        level.generate_simple_floor();
                                        monsters = generate_monsters_for_depth(&level, player.balrog_killed);
                                        
                                        let su_pos = level.tiles.iter().enumerate()
                                            .find(|(_, t)| t.tile_type == TileType::StairsUp)
                                            .map(|(i, _)| (i % level.width, i / level.width))
                                            .unwrap_or((15, 15));
                                        player.move_to(su_pos.0, su_pos.1);
                                        
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
                                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                                
                                let state = GameState::new(player.clone(), level.clone(), monsters.clone(), max_depth);
                                if let Err(e) = state.save_to_file(save_path) {
                                    status_msg = format!("Save failed: {}", e);
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
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
                    match key_event.code {
                        KeyCode::Esc => {
                            if screen_mode == ScreenMode::ShopSellMenu {
                                screen_mode = ScreenMode::Shop;
                                status_msg = "General Store - Buy Menu.".to_string();
                            } else {
                                screen_mode = ScreenMode::Dungeon;
                                status_msg = "Returned to dungeon.".to_string();
                            }
                        }
                        KeyCode::Char(c) => {
                            if screen_mode == ScreenMode::InventoryList && c == 'i' {
                                screen_mode = ScreenMode::Dungeon;
                                status_msg = "Returned to dungeon.".to_string();
                            } else if screen_mode == ScreenMode::EquipmentList && c == 'I' {
                                screen_mode = ScreenMode::Dungeon;
                                status_msg = "Returned to dungeon.".to_string();
                            } else if screen_mode == ScreenMode::Shop {
                                match c {
                                    's' => {
                                        screen_mode = ScreenMode::ShopSellMenu;
                                        status_msg = "Sell selection mode. Select item.".to_string();
                                    }
                                    letter if letter >= 'a' && letter <= 'z' => {
                                        let idx = letter as usize - 'a' as usize;
                                        let items = get_shop_items(active_shop);
                                        if idx < items.len() {
                                            let (name, price, ref item_type) = items[idx];
                                            if player.gold >= price {
                                                player.gold -= price;
                                                let it = match item_type {
                                                    ItemType::Weapon { damage } => ItemType::Weapon { damage: *damage },
                                                    ItemType::Armor { ac } => ItemType::Armor { ac: *ac },
                                                    ItemType::Potion { heal_amount } => ItemType::Potion { heal_amount: *heal_amount },
                                                    ItemType::Scroll { teleport } => ItemType::Scroll { teleport: *teleport },
                                                };
                                                let weight = match &it {
                                                    ItemType::Weapon { .. } => 10,
                                                    ItemType::Armor { .. } => 80,
                                                    ItemType::Potion { .. } => 5,
                                                    ItemType::Scroll { .. } => 2,
                                                };
                                                add_item_to_inventory(&mut player.inventory, Item::new(name, 1, weight, it));
                                                status_msg = format!("You bought {}!", name);
                                            } else {
                                                status_msg = "You don't have enough gold!".to_string();
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            } else if screen_mode == ScreenMode::ShopSellMenu {
                                let idx = c as usize - 'a' as usize;
                                if idx < player.inventory.len() {
                                    let value = match player.inventory[idx].item_type {
                                        ItemType::Weapon {..} => 25,
                                        ItemType::Armor {..} => 40,
                                        ItemType::Potion {..} => 15,
                                        ItemType::Scroll {..} => 10,
                                    };

                                    player.gold += value;
                                    status_msg = format!("You sold 1x {} for {} gp.", player.inventory[idx].name, value);
                                    
                                    player.inventory[idx].count -= 1;
                                    if player.inventory[idx].count == 0 {
                                        player.inventory.remove(idx);
                                    }

                                    if player.inventory.is_empty() {
                                        screen_mode = ScreenMode::Shop;
                                    }
                                }
                            } else if screen_mode == ScreenMode::WearMenu {
                                let idx = c as usize - 'a' as usize;
                                let equippable_indices: Vec<usize> = player.inventory.iter()
                                    .enumerate()
                                    .filter(|(_, item)| {
                                        matches!(item.item_type, ItemType::Weapon {..} | ItemType::Armor {..}) ||
                                        item.name.contains("Torch") || item.name.contains("Lantern")
                                    })
                                    .map(|(i, _)| i)
                                    .collect();

                                if idx < equippable_indices.len() {
                                    let inv_idx = equippable_indices[idx];
                                    let item = player.inventory.remove(inv_idx);
                                    let is_weapon = matches!(item.item_type, ItemType::Weapon {..});
                                    let is_armor = matches!(item.item_type, ItemType::Armor {..});

                                    let already_equipped = player.equipment.iter().position(|eq| {
                                        if is_weapon {
                                            matches!(eq.item_type, ItemType::Weapon {..})
                                        } else if is_armor {
                                            matches!(eq.item_type, ItemType::Armor {..})
                                        } else {
                                            eq.name.contains("Torch") || eq.name.contains("Lantern")
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
                                    .filter(|(_, item)| matches!(item.item_type, ItemType::Scroll {..}) && !item.name.contains("Torch"))
                                    .map(|(i, _)| i)
                                    .collect();

                                if idx < scroll_indices.len() {
                                    let inv_idx = scroll_indices[idx];
                                    let name = player.inventory[inv_idx].name.clone();
                                    
                                    player.inventory[inv_idx].count -= 1;
                                    if player.inventory[inv_idx].count == 0 {
                                        player.inventory.remove(inv_idx);
                                    }

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
                                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                                
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
                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                } else {
                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                }
            }
        }
    }

    // 4. Restore normal terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
