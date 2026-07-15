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
    BrowseBookMenu,
    CastSpellMenu,
    PrayMenu,
    SelectSpellDirection,
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

fn get_shop_items(shop: ShopType) -> Vec<(&'static str, u32, ItemType)> {
    match shop {
        ShopType::General => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Mage Spellbook [Beginner's Magick]", 50, ItemType::Scroll { teleport: false }),
            ("Priest Prayerbook [Beginner's Handbook]", 50, ItemType::Scroll { teleport: false }),
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
            ("Priest Prayerbook [Beginner's Handbook]", 50, ItemType::Scroll { teleport: false }),
        ],
        ShopType::Alchemy => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
        ],
        ShopType::Magic => vec![
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Scroll of Teleportation", 60, ItemType::Scroll { teleport: true }),
            ("Mage Spellbook [Beginner's Magick]", 50, ItemType::Scroll { teleport: false }),
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
        14 => format!("MANA: {:>2}/{:<2}", player.mana, player.max_mana),
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
                        } else if item.name.contains("Spellbook") || item.name.contains("Prayerbook") {
                            "Spell Book".to_string()
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
                .filter(|(_, item)| {
                    matches!(item.item_type, ItemType::Scroll {..}) &&
                    !item.name.contains("Torch") &&
                    !item.name.contains("Spellbook") &&
                    !item.name.contains("Prayerbook")
                })
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
        ScreenMode::BrowseBookMenu => {
            if row == 0 {
                return "--- BROWSE SPELLS & PRAYERS ---".to_string();
            }
            let is_mage = matches!(player.class, Class::Mage | Class::Rogue | Class::Ranger);
            if is_mage {
                let spells = &[
                    "a) Magic Missile       - 1 Mana, Level 1 (Fires projectile, 2d4 damage)",
                    "b) Phase Door           - 2 Mana, Level 1 (Short range teleport)",
                    "c) Light Area          - 3 Mana, Level 3 (Reveals radius 5 tiles)",
                    "d) Fire Bolt            - 5 Mana, Level 5 (Fires projectile, 4d6 damage)",
                ];
                if row > 0 && row <= spells.len() {
                    return spells[row - 1].to_string();
                }
            } else {
                let prayers = &[
                    "a) Detect Evil         - 1 Mana, Level 1 (Reveals monster indicators)",
                    "b) Cure Light Wounds   - 2 Mana, Level 1 (Heals player 2d8 HP)",
                    "c) Bless               - 3 Mana, Level 3 (Heals player 3d8 HP)",
                    "d) Portal              - 4 Mana, Level 4 (Teleports to random location)",
                    "e) Holy Word           - 7 Mana, Level 7 (Divine blast to all adjacent, 6d8)",
                ];
                if row > 0 && row <= prayers.len() {
                    return prayers[row - 1].to_string();
                }
            }
            if row == 7 {
                return "Press ESC to exit book browser.".to_string();
            }
            "".to_string()
        }
        ScreenMode::CastSpellMenu => {
            if row == 0 {
                return "--- CAST MAGE SPELL ---".to_string();
            }
            let spells = &[
                "a) Magic Missile       - 1 Mana, Level 1 (2d4 projectile)",
                "b) Phase Door           - 2 Mana, Level 1 (Short teleport)",
                "c) Light Area          - 3 Mana, Level 3 (Light flash)",
                "d) Fire Bolt            - 5 Mana, Level 5 (4d6 projectile)",
            ];
            if row > 0 && row <= spells.len() {
                return spells[row - 1].to_string();
            }
            if row == 6 {
                return "Select spell letter to cast, or press ESC to cancel.".to_string();
            }
            "".to_string()
        }
        ScreenMode::PrayMenu => {
            if row == 0 {
                return "--- RECITE CLERICAL PRAYER ---".to_string();
            }
            let prayers = &[
                "a) Detect Evil         - 1 Mana, Level 1 (Detect nearby monsters)",
                "b) Cure Light Wounds   - 2 Mana, Level 1 (Heals 2d8 HP)",
                "c) Bless               - 3 Mana, Level 3 (Heals 3d8 HP)",
                "d) Portal              - 4 Mana, Level 4 (Random teleport)",
                "e) Holy Word           - 7 Mana, Level 7 (Adjacent area blast, 6d8)",
            ];
            if row > 0 && row <= prayers.len() {
                return prayers[row - 1].to_string();
            }
            if row == 7 {
                return "Select prayer letter to recite, or press ESC to cancel.".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectSpellDirection => {
            if row == 0 {
                return "--- CASTING TARGETING ---".to_string();
            }
            if row == 2 {
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c):".to_string();
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
    for tile in level.tiles.iter_mut() {
        tile.visible = false;
    }
    
    if level.depth == 0 {
        for tile in level.tiles.iter_mut() {
            tile.visible = true;
            tile.remembered = true;
        }
        return;
    }
    
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
    
    let radius = player.get_light_radius();
    let max_dist = 8;
    
    for y in 0..level.height {
        for x in 0..level.width {
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
            let dist = dx.max(dy) as usize;
            
            if dist <= max_dist {
                if has_los(player.x, player.y, x, y, level) {
                    if dist <= radius {
                        if let Some(tile) = level.get_tile_mut(x, y) {
                            tile.visible = true;
                            tile.remembered = true;
                        }
                    } else {
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
    update_visibility(level, player);

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 80) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 24) / 2).max(0) as u16;

    let msg_line = format!("Message: {}\x1b[K", status_msg);
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y));
    print!("{}", msg_line);

    for map_y in 0..level.height {
        let stats_part = get_lhs_stat_line(map_y + 1, player, level);
        let mut map_part = String::new();

        if mode == ScreenMode::Dungeon {
            for map_x in 0..level.width {
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        map_part.push('@');
                    } else if tile.visible && monsters.iter().any(|m| m.x == map_x && m.y == map_y) {
                        let monster = monsters.iter().find(|m| m.x == map_x && m.y == map_y).unwrap();
                        map_part.push(monster.symbol);
                    } else if tile.visible || tile.remembered {
                        match tile.tile_type {
                            TileType::Wall => map_part.push('#'),
                            TileType::Floor => map_part.push('.'),
                            TileType::DoorClosed => map_part.push('+'),
                            TileType::DoorOpen => map_part.push('\''),
                            TileType::StairsUp => map_part.push('<'),
                            TileType::StairsDown => map_part.push('>'),
                            TileType::ShopDoor(num) => map_part.push((b'0' + num) as char),
                            TileType::Empty => map_part.push(' '),
                        }
                    } else {
                        map_part.push(' ');
                    }
                }
            }
        } else {
            map_part = format!("{:<66}", get_overlay_row_text(map_y, mode, player, shop));
        }

        let row_line = format!("{} {}\x1b[K", stats_part, map_part);
        let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 1 + map_y as u16));
        print!("{}", row_line);
    }
    
    let divider = "-------------------------------------------------------------------------------\x1b[K";
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 23));
    print!("{}", divider);
    
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
    
    if depth == max_depth {
        if !player_has_killed_balrog {
            let (bx, by) = find_passable_tile(level);
            mons.push(Monster::new("The Balrog", 'B', bx, by, 120, Dice::new(3, 8), 500));
        }
        for _ in 0..3 {
            let (gx, gy) = find_passable_tile(level);
            mons.push(Monster::new("Lich Guardian", 'L', gx, gy, 55, Dice::new(3, 6), 180));
        }
        return mons;
    }
    
    let mut active_level = depth;
    if rng.gen_bool(0.10) {
        active_level += rng.gen_range(1..=3);
    }
    
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

fn print_creation_screen(title: &str, options: &[&str]) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 18) / 2).max(0) as u16;

    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    let mut row_idx = 0;
    
    let mut print_line = |text: &str| {
        let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + row_idx));
        print!("{}", text);
        row_idx += 1;
    };

    print_line("==========================================================");
    print_line(&format!("  Moria Character Creator: {:<30}", title));
    print_line("==========================================================");
    print_line("");
    for opt in options {
        print_line(&format!("  {}", opt));
    }
    for _ in 0..(12 - options.len()) {
        print_line("");
    }
    print_line("==========================================================");
    let _ = io::stdout().flush();
}

fn run_character_creation(
    _stdout: &mut io::Stdout,
    rng: &mut impl rand::Rng,
) -> Result<Player, Box<dyn std::error::Error>> {
    // 1. Choose Race
    let race = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER RACE", &[
            "a) Human",
            "b) Half-Elf",
            "c) Elf",
            "d) Halfling",
            "e) Gnome",
            "f) Dwarf",
            "g) Half-Orc",
            "h) Half-Troll",
        ]);
        if let Event::Key(key_event) = event::read()? {
            if let KeyCode::Char(c) = key_event.code {
                match c {
                    'a' => break Race::Human,
                    'b' => break Race::HalfElf,
                    'c' => break Race::Elf,
                    'd' => break Race::Halfling,
                    'e' => break Race::Gnome,
                    'f' => break Race::Dwarf,
                    'g' => break Race::HalfOrc,
                    'h' => break Race::HalfTroll,
                    _ => {}
                }
            }
        }
    };

    // 2. Choose Class
    let class = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER CLASS", &[
            "a) Warrior",
            "b) Mage",
            "c) Priest",
            "d) Rogue",
            "e) Ranger",
            "f) Paladin",
        ]);
        if let Event::Key(key_event) = event::read()? {
            if let KeyCode::Char(c) = key_event.code {
                match c {
                    'a' => break Class::Warrior,
                    'b' => break Class::Mage,
                    'c' => break Class::Priest,
                    'd' => break Class::Rogue,
                    'e' => break Class::Ranger,
                    'f' => break Class::Paladin,
                    _ => {}
                }
            }
        }
    };

    // 3. Roll Stats loop
    let stats = loop {
        let s_str = rng.gen_range(8..=18);
        let s_int = rng.gen_range(8..=18);
        let s_wis = rng.gen_range(8..=18);
        let s_dex = rng.gen_range(8..=18);
        let s_con = rng.gen_range(8..=18);
        let s_chr = rng.gen_range(8..=18);
        let rolled = player::Attributes::new(s_str, s_int, s_wis, s_dex, s_con, s_chr);

        let mut temp_player = Player {
            name: "".to_string(),
            race,
            class,
            level: 1,
            exp: 0,
            gold: 150,
            x: 0, y: 0,
            max_hp: 15, hp: 15,
            max_mana: 0, mana: 0,
            stats: rolled.clone(),
            inventory: Vec::new(),
            equipment: Vec::new(),
            balrog_killed: false,
            base_hp_levels: vec![15; 40],
            exp_factor: 100,
        };
        temp_player.apply_race_and_class_modifiers();

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("ROLL CHARACTER ATTRIBUTES", &[
            &format!("STR:  {:>2}", temp_player.stats.strength),
            &format!("INT:  {:>2}", temp_player.stats.intelligence),
            &format!("WIS:  {:>2}", temp_player.stats.wisdom),
            &format!("DEX:  {:>2}", temp_player.stats.dexterity),
            &format!("CON:  {:>2}", temp_player.stats.constitution),
            &format!("CHR:  {:>2}", temp_player.stats.charisma),
            "",
            "Press [SPACE] to re-roll stats.",
            "Press [ENTER] to accept these characteristics.",
        ]);

        if let Event::Key(key_event) = event::read()? {
            if key_event.code == KeyCode::Enter {
                break rolled;
            }
        }
    };

    // 4. Input Name (centered on terminal)
    disable_raw_mode()?;
    execute!(io::stdout(), Show)?;
    
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 8) / 2).max(0) as u16;

    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y));
    print!("==========================================================\n");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 1));
    print!("              ENTER CHARACTER NAME                        \n");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 2));
    print!("==========================================================\n");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 4));
    print!("Name: ");
    let _ = io::stdout().flush();
    
    let mut name = String::new();
    let _ = io::stdin().read_line(&mut name);
    let trimmed_name = name.trim();
    let name_str = if trimmed_name.is_empty() { "Hero" } else { trimmed_name };

    enable_raw_mode()?;
    execute!(io::stdout(), Hide)?;

    let mut final_player = Player::new(name_str, race, class, 30, 10);
    final_player.stats = stats;
    final_player.apply_race_and_class_modifiers();
    final_player.update_max_hp_and_mana();
    final_player.hp = final_player.max_hp;
    final_player.mana = final_player.max_mana;

    Ok(final_player)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = "save.json";
    let mut rng = rand::thread_rng();
    
    // Check terminal size first before entering alternate screen
    let (cols, rows) = crossterm::terminal::size()?;
    if cols < 80 || rows < 24 {
        return Err(format!(
            "Terminal size is too small (current: {}x{}). Please resize your terminal to at least 80x24.",
            cols, rows
        ).into());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;

    let (mut player, mut level, mut monsters, mut status_msg, max_depth) = if Path::new(save_path).exists() {
        match GameState::load_from_file(save_path) {
            Ok(state) => {
                (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string(), state.max_depth)
            }
            Err(e) => {
                let rolled_max = rng.gen_range(8..=15);
                let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                lvl.generate_simple_floor();
                let p = run_character_creation(&mut stdout, &mut rng)?;
                let start_pos = find_passable_tile(&lvl);
                let mut player = p;
                player.move_to(start_pos.0, start_pos.1);
                let mons = generate_monsters_for_depth(&lvl, false);
                (player, lvl, mons, format!("Failed to load save: {}. Started new game.", e), rolled_max)
            }
        }
    } else {
        let rolled_max = rng.gen_range(8..=15);
        let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
        lvl.generate_simple_floor();
        let p = run_character_creation(&mut stdout, &mut rng)?;
        let start_pos = find_passable_tile(&lvl);
        let mut player = p;
        player.move_to(start_pos.0, start_pos.1);
        let mons = generate_monsters_for_depth(&lvl, false);
        (player, lvl, mons, "Welcome to rmoria! Find town shops or descend stairs (>)".to_string(), rolled_max)
    };

    let mut screen_mode = ScreenMode::Dungeon;
    let mut active_shop = ShopType::General;
    let mut selected_spell_idx: usize = 0;

    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    'game_loop: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop)?;
                }
                Event::Key(key_event) => {
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
                                    else if let Some(m_idx) = monsters.iter().position(|m| m.x == next_x && m.y == next_y) {
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
                                                    player.mana = (player.mana + 1).min(player.max_mana);
                                                    status_msg = "You rest and recover health/mana.".to_string();
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
                                Action::BrowseBook => {
                                    let has_book = player.inventory.iter().any(|i| i.name.contains("Spellbook") || i.name.contains("Prayerbook"));
                                    if has_book {
                                        screen_mode = ScreenMode::BrowseBookMenu;
                                        status_msg = "Browsing spells/prayers.".to_string();
                                    } else {
                                        status_msg = "You do not carry a spellbook or prayerbook!".to_string();
                                    }
                                }
                                Action::CastSpell => {
                                    let is_mage_caster = matches!(player.class, Class::Mage | Class::Rogue | Class::Ranger);
                                    if !is_mage_caster {
                                        status_msg = "Your class cannot cast mage spells!".to_string();
                                    } else {
                                        let has_book = player.inventory.iter().any(|i| i.name.contains("Mage Spellbook"));
                                        if has_book {
                                            screen_mode = ScreenMode::CastSpellMenu;
                                            status_msg = "Cast Mage Spell: select a letter.".to_string();
                                        } else {
                                            status_msg = "You need a Mage Spellbook to cast spells!".to_string();
                                        }
                                    }
                                }
                                Action::Pray => {
                                    let is_priest_caster = matches!(player.class, Class::Priest | Class::Paladin);
                                    if !is_priest_caster {
                                        status_msg = "Your class cannot recite priestly prayers!".to_string();
                                    } else {
                                        let has_book = player.inventory.iter().any(|i| i.name.contains("Priest Prayerbook"));
                                        if has_book {
                                            screen_mode = ScreenMode::PrayMenu;
                                            status_msg = "Recite Clerical Prayer: select a letter.".to_string();
                                        } else {
                                            status_msg = "You need a Priest Prayerbook to pray!".to_string();
                                        }
                                    }
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
                                } else if screen_mode == ScreenMode::BrowseBookMenu {
                                    // Browse mode exit via ESC
                                } else if screen_mode == ScreenMode::CastSpellMenu {
                                    match c {
                                        'a' => {
                                            if player.mana >= 1 {
                                                selected_spell_idx = 0;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Magic Missile: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'b' => {
                                            if player.mana >= 2 {
                                                player.mana -= 2;
                                                let dest = find_passable_tile(&level);
                                                player.move_to(dest.0, dest.1);
                                                status_msg = "You cast Phase Door and warp!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'c' => {
                                            if player.level < 3 {
                                                status_msg = "Too low level (requires Level 3)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 3 {
                                                player.mana -= 3;
                                                for y in (player.y as isize - 5)..=(player.y as isize + 5) {
                                                    for x in (player.x as isize - 5)..=(player.x as isize + 5) {
                                                        if let Some(tile) = level.get_tile_mut(x as usize, y as usize) {
                                                            tile.visible = true;
                                                            tile.remembered = true;
                                                        }
                                                    }
                                                }
                                                status_msg = "Brilliant light flashes and illuminates the area!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'd' => {
                                            if player.level < 5 {
                                                status_msg = "Too low level (requires Level 5)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 5 {
                                                selected_spell_idx = 1;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Fire Bolt: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if screen_mode == ScreenMode::PrayMenu {
                                    match c {
                                        'a' => {
                                            if player.mana >= 1 {
                                                player.mana -= 1;
                                                for m in &monsters {
                                                    if let Some(tile) = level.get_tile_mut(m.x, m.y) {
                                                        tile.remembered = true;
                                                    }
                                                }
                                                status_msg = format!("You sense the presence of {} dark entities!", monsters.len());
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'b' => {
                                            if player.mana >= 2 {
                                                player.mana -= 2;
                                                let heal = rng.gen_range(2..=16);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("You pray for healing. Restored {} HP.", heal);
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'c' => {
                                            if player.level < 3 {
                                                status_msg = "Too low level (requires Level 3)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 3 {
                                                player.mana -= 3;
                                                let heal = rng.gen_range(3..=24);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("Divine blessing heals you for {} HP.", heal);
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'd' => {
                                            if player.level < 4 {
                                                status_msg = "Too low level (requires Level 4)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 4 {
                                                player.mana -= 4;
                                                let dest = find_passable_tile(&level);
                                                player.move_to(dest.0, dest.1);
                                                status_msg = "You are pulled through a space portal!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'e' => {
                                            if player.level < 7 {
                                                status_msg = "Too low level (requires Level 7)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 7 {
                                                player.mana -= 7;
                                                let mut hit = false;
                                                let mut killed = Vec::new();
                                                for (m_idx, m) in monsters.iter_mut().enumerate() {
                                                    let dx = (player.x as isize - m.x as isize).abs();
                                                    let dy = (player.y as isize - m.y as isize).abs();
                                                    if dx <= 1 && dy <= 1 {
                                                        let dmg = rng.gen_range(6..=48);
                                                        m.take_damage(dmg);
                                                        hit = true;
                                                        if m.hp <= 0 {
                                                            killed.push(m_idx);
                                                        }
                                                    }
                                                }
                                                for &idx in killed.iter().rev() {
                                                    let exp = monsters[idx].experience_reward;
                                                    player.add_experience(exp);
                                                    monsters.remove(idx);
                                                }
                                                status_msg = if hit {
                                                    "You chant a Holy Word! Nearby enemies are scorched!".to_string()
                                                } else {
                                                    "You chant a Holy Word, but hear only whispers.".to_string()
                                                };
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if screen_mode == ScreenMode::SelectSpellDirection {
                                    let action = mapper.map_key(c);
                                    if let Action::Move(direction) = action {
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

                                        if selected_spell_idx == 0 {
                                            player.mana -= 1;
                                            let mut cx = player.x as isize + dx;
                                            let mut cy = player.y as isize + dy;
                                            status_msg = "Your Magic Missile fizzles into the dark!".to_string();
                                            loop {
                                                if let Some(tile) = level.get_tile(cx as usize, cy as usize) {
                                                    if tile.tile_type == TileType::Wall {
                                                        status_msg = "Your Magic Missile hits a wall and breaks!".to_string();
                                                        break;
                                                    }
                                                } else {
                                                    break;
                                                }
                                                if let Some(m_idx) = monsters.iter().position(|m| m.x == cx as usize && m.y == cy as usize) {
                                                    let dmg = rng.gen_range(2..=8);
                                                    let m_name = monsters[m_idx].name.clone();
                                                    let exp = monsters[m_idx].experience_reward;
                                                    status_msg = format!("Magic Missile hits {} for {} damage!", m_name, dmg);
                                                    if monsters[m_idx].take_damage(dmg) {
                                                        status_msg.push_str(" You killed it!");
                                                        player.add_experience(exp);
                                                        monsters.remove(m_idx);
                                                    }
                                                    break;
                                                }
                                                cx += dx;
                                                cy += dy;
                                            }
                                        } else {
                                            player.mana -= 5;
                                            let mut cx = player.x as isize + dx;
                                            let mut cy = player.y as isize + dy;
                                            status_msg = "Your Fire Bolt flares out into nothingness!".to_string();
                                            loop {
                                                if let Some(tile) = level.get_tile(cx as usize, cy as usize) {
                                                    if tile.tile_type == TileType::Wall {
                                                        status_msg = "Your Fire Bolt strikes a wall!".to_string();
                                                        break;
                                                    }
                                                } else {
                                                    break;
                                                }
                                                if let Some(m_idx) = monsters.iter().position(|m| m.x == cx as usize && m.y == cy as usize) {
                                                    let dmg = rng.gen_range(4..=24);
                                                    let m_name = monsters[m_idx].name.clone();
                                                    let exp = monsters[m_idx].experience_reward;
                                                    status_msg = format!("Fire Bolt incinerates {} for {} damage!", m_name, dmg);
                                                    if monsters[m_idx].take_damage(dmg) {
                                                        status_msg.push_str(" Gained experience.");
                                                        player.add_experience(exp);
                                                        monsters.remove(m_idx);
                                                    }
                                                    break;
                                                }
                                                cx += dx;
                                                cy += dy;
                                            }
                                        }

                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
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
                                        .filter(|(_, item)| {
                                            matches!(item.item_type, ItemType::Scroll {..}) &&
                                            !item.name.contains("Torch") &&
                                            !item.name.contains("Spellbook") &&
                                            !item.name.contains("Prayerbook")
                                        })
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
                _ => {}
            }
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
