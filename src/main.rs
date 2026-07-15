mod input;
mod dungeon;
mod player;
mod save;
mod dice;
mod entity;

use serde::{Serialize, Deserialize};

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
use dungeon::{DungeonLevel, TileType, ShopType, FloorItem, tile::TrapType, generate_random_floor_item};
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
    CharacterStatsMenu,
    SelectDisarmDirection,
    BarterBuyMenu,
    BarterSellMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaggleState {
    pub item: Item,
    pub item_index: usize,
    pub initial_price: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub current_asking: u32,
    pub last_bid: u32,
    pub insults: u32,
    pub offers_count: u32,
    pub comment: String,
    pub player_input: String,
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

fn get_item_symbol(item: &Item) -> char {
    if item.name.contains("Gold Pile") {
        '*'
    } else if item.name.contains("Spellbook") || item.name.contains("Prayerbook") {
        '?'
    } else {
        match &item.item_type {
            ItemType::Weapon { .. } => ')',
            ItemType::Armor { .. } => '[',
            ItemType::Potion { .. } => '!',
            ItemType::Scroll { .. } => '?',
        }
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

fn wrap_text(text: &str, limit: usize) -> Vec<String> {
    let mut words = text.split_whitespace();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    while let Some(word) = words.next() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= limit {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

fn get_overlay_row_text(row: usize, mode: ScreenMode, player: &Player, shop: ShopType, active_haggle: Option<&HaggleState>) -> String {
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
        ScreenMode::SelectDisarmDirection => {
            if row == 0 {
                return "--- TRAP DISARMING ---".to_string();
            }
            if row == 2 {
                return "Choose direction of the trap to disarm (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::CharacterStatsMenu => {
            let split_history = wrap_text(&player.history, 62);
            match row {
                0 => format!("--- CHARACTER SHEET: {} ---", player.name),
                2 => format!("Race:      {:?}      Class:      {:?}", player.race, player.class),
                4 => "Attributes:".to_string(),
                5 => format!("  STR: {:>2}   INT: {:>2}   WIS: {:>2}", player.stats.strength, player.stats.intelligence, player.stats.wisdom),
                6 => format!("  DEX: {:>2}   CON: {:>2}   CHR: {:>2}", player.stats.dexterity, player.stats.constitution, player.stats.charisma),
                8 => "Combat Stats:".to_string(),
                9 => format!("  HP:   {}/{}       Mana: {}/{}", player.hp, player.max_hp, player.mana, player.max_mana),
                10 => format!("  AC:   {:<10} Gold: {} gp", player.calculate_ac(), player.gold),
                11 => format!("  Level: {:<9} Exp:  {} exp", player.level, player.exp),
                13 => "Racial History & Background:".to_string(),
                r if r >= 14 && r < 14 + split_history.len() => {
                    format!("  {}", split_history[r - 14])
                }
                r if r == 14 + split_history.len() + 1 => {
                    "--------------------------------------------".to_string()
                }
                r if r == 14 + split_history.len() + 2 => {
                    "Press ESC to return to the dungeon.".to_string()
                }
                _ => "".to_string(),
            }
        }
        ScreenMode::BarterBuyMenu => {
            if let Some(haggle) = active_haggle {
                match row {
                    0 => format!("--- BARTERING (BUY): {} ---", haggle.item.name),
                    2 => format!("Shopkeeper: \"{}\"", haggle.comment),
                    3 => format!("Current Ask Price:  {} gp", haggle.current_asking),
                    5 => format!("Your last offer:    {} gp", haggle.last_bid),
                    6 => format!("Patience left:      {}", 3 - haggle.insults),
                    8 => format!("Enter your offer:   {}", haggle.player_input),
                    10 => "--------------------------------------------".to_string(),
                    11 => "Type your bid (digits) and press ENTER to submit.".to_string(),
                    12 => "Press ENTER on blank input to accept the shopkeeper's price.".to_string(),
                    13 => "Press ESC to cancel and exit.".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        }
        ScreenMode::BarterSellMenu => {
            if let Some(haggle) = active_haggle {
                match row {
                    0 => format!("--- BARTERING (SELL): {} ---", haggle.item.name),
                    2 => format!("Shopkeeper: \"{}\"", haggle.comment),
                    3 => format!("Current Offer:      {} gp", haggle.current_asking),
                    5 => format!("Your last ask:      {} gp", haggle.last_bid),
                    6 => format!("Patience left:      {}", 3 - haggle.insults),
                    8 => format!("Enter your price:   {}", haggle.player_input),
                    10 => "--------------------------------------------".to_string(),
                    11 => "Type your price (digits) and press ENTER to submit.".to_string(),
                    12 => "Press ENTER on blank input to accept the shopkeeper's price.".to_string(),
                    13 => "Press ESC to cancel and exit.".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
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
                if tile.tile_type == TileType::Wall || tile.tile_type == TileType::DoorClosed || tile.tile_type == TileType::SecretDoor {
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
    active_haggle: Option<&HaggleState>,
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
                    } else if (tile.visible || tile.remembered) && level.items.iter().any(|i| i.x == map_x && i.y == map_y) && !matches!(tile.tile_type, TileType::Wall | TileType::SecretDoor) {
                        let floor_item = level.items.iter().find(|i| i.x == map_x && i.y == map_y).unwrap();
                        map_part.push(get_item_symbol(&floor_item.item));
                    } else if tile.visible || tile.remembered {
                        match tile.tile_type {
                            TileType::Wall => map_part.push('#'),
                            TileType::Floor => map_part.push('.'),
                            TileType::DoorClosed => map_part.push('+'),
                            TileType::DoorOpen => map_part.push('\''),
                            TileType::StairsUp => map_part.push('<'),
                            TileType::StairsDown => map_part.push('>'),
                            TileType::ShopDoor(num) => map_part.push((b'0' + num) as char),
                            TileType::SecretDoor => map_part.push('#'),
                            TileType::Trap { detected, .. } => {
                                if detected {
                                    map_part.push('^');
                                } else {
                                    map_part.push('.');
                                }
                            }
                            TileType::Empty => map_part.push(' '),
                        }
                    } else {
                        map_part.push(' ');
                    }
                }
            }
        } else {
            map_part = format!("{:<66}", get_overlay_row_text(map_y, mode, player, shop, active_haggle));
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
            history: "".to_string(),
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
    let mut active_haggle: Option<HaggleState> = None;

    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

    'game_loop: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
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

                                                let is_boss = monster_name == "The Balrog" || monster_name.contains("Lich");
                                                if is_boss || rng.gen_bool(0.30) {
                                                    let drop_item = if monster_name == "The Balrog" {
                                                        Item::new("Broadsword of Slaying", 1, 150, ItemType::Weapon { damage: Dice::new(3, 6) })
                                                    } else {
                                                        generate_random_floor_item(level.depth, &mut rng)
                                                    };
                                                    level.items.push(FloorItem { x: next_x, y: next_y, item: drop_item.clone() });
                                                    status_msg.push_str(&format!(" It dropped a {}!", drop_item.name));
                                                }
                                                
                                                if monster_name == "The Balrog" {
                                                    player.balrog_killed = true;
                                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
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
                                                let mut trigger_trap = None;
                                                if let TileType::Trap { trap_type, .. } = tile.tile_type {
                                                    trigger_trap = Some(trap_type);
                                                }

                                                player.move_to(next_x, next_y);
                                                
                                                if direction == Direction::Rest {
                                                    player.hp = (player.hp + 1).min(player.max_hp);
                                                    player.mana = (player.mana + 1).min(player.max_mana);
                                                    status_msg = "You rest and recover health/mana.".to_string();
                                                } else {
                                                    status_msg = format!("Moved to ({}, {})", next_x, next_y);
                                                }

                                                if let Some(tt) = trigger_trap {
                                                    match tt {
                                                        TrapType::Arrow => {
                                                            let dmg = rng.gen_range(1..=6);
                                                            player.hp -= dmg;
                                                            status_msg = format!("Click! You set off an arrow trap! You take {} damage!", dmg);
                                                        }
                                                        TrapType::PoisonGas => {
                                                            let dmg = rng.gen_range(2..=8);
                                                            player.hp -= dmg;
                                                            status_msg = format!("Click! Poison gas fills the corridor! You take {} damage!", dmg);
                                                        }
                                                        TrapType::Teleport => {
                                                            let dest = find_passable_tile(&level);
                                                            player.move_to(dest.0, dest.1);
                                                            status_msg = "Click! A teleport trap warps you to another location!".to_string();
                                                        }
                                                    }
                                                    if let Some(t) = level.get_tile_mut(player.x, player.y) {
                                                        t.tile_type = TileType::Floor;
                                                    }
                                                }

                                                let item_idx = level.items.iter().position(|i| i.x == player.x && i.y == player.y);
                                                if let Some(idx) = item_idx {
                                                    let floor_item = level.items.remove(idx);
                                                    let item = floor_item.item;
                                                    if item.name.contains("Gold Pile") {
                                                        let gold_amount = item.name.split('[')
                                                            .nth(1)
                                                            .and_then(|s| s.split(' ').next())
                                                            .and_then(|s| s.parse::<u32>().ok())
                                                            .unwrap_or(20);
                                                        player.gold += gold_amount;
                                                        status_msg = format!("You picked up {} gold pieces.", gold_amount);
                                                    } else {
                                                        status_msg = format!("You picked up a {}.", item.name);
                                                        add_item_to_inventory(&mut player.inventory, item);
                                                    }
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
                                Action::CharacterStats => {
                                    screen_mode = ScreenMode::CharacterStatsMenu;
                                    status_msg = "Viewing character sheet.".to_string();
                                }
                                Action::Disarm => {
                                    screen_mode = ScreenMode::SelectDisarmDirection;
                                    status_msg = "Disarm trap: choose direction...".to_string();
                                }
                                Action::SearchOneTurn => {
                                    let is_rogue = player.class == Class::Rogue;
                                    let is_elf = matches!(player.race, Race::Elf | Race::HalfElf);
                                    let trap_chance = if is_rogue { 0.70 } else { 0.30 };
                                    let door_chance = if is_rogue { 0.60 } else if is_elf { 0.40 } else { 0.20 };
                                    
                                    let mut found_something = false;

                                    for dy in -1..=1 {
                                        for dx in -1..=1 {
                                            if dx == 0 && dy == 0 { continue; }
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;
                                            
                                            if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                match tile.tile_type {
                                                    TileType::SecretDoor => {
                                                        if rng.gen_bool(door_chance) {
                                                            tile.tile_type = TileType::DoorClosed;
                                                            status_msg = "You found a secret door!".to_string();
                                                            found_something = true;
                                                        }
                                                    }
                                                    TileType::Trap { ref mut detected, .. } => {
                                                        if !*detected && rng.gen_bool(trap_chance) {
                                                            *detected = true;
                                                            status_msg = "You detected a trap!".to_string();
                                                            found_something = true;
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    
                                    if !found_something {
                                        status_msg = "You search but find nothing.".to_string();
                                    }
                                    player_acted = true;
                                }
                                Action::Quit => {
                                    status_msg = "Saving game and quitting...".to_string();
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
                                    
                                    let state = GameState::new(player.clone(), level.clone(), monsters.clone(), max_depth);
                                    if let Err(e) = state.save_to_file(save_path) {
                                        status_msg = format!("Save failed: {}", e);
                                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
                                        std::thread::sleep(std::time::Duration::from_secs(2));
                                    } else {
                                        std::thread::sleep(std::time::Duration::from_millis(800));
                                    }
                                    break 'game_loop;
                                }
                                _ => {}
                            }
                        }
                    } else if screen_mode == ScreenMode::BarterBuyMenu {
                        if let Some(ref mut haggle) = active_haggle {
                            match key_event.code {
                                KeyCode::Esc => {
                                    screen_mode = ScreenMode::Shop;
                                    status_msg = "Transaction cancelled.".to_string();
                                    active_haggle = None;
                                }
                                KeyCode::Backspace => {
                                    haggle.player_input.pop();
                                }
                                KeyCode::Enter => {
                                    if haggle.player_input.is_empty() {
                                        let final_price = haggle.current_asking;
                                        if player.gold >= final_price {
                                            player.gold -= final_price;
                                            add_item_to_inventory(&mut player.inventory, haggle.item.clone());
                                            status_msg = format!("Done! You bought it for {} gp.", final_price);
                                        } else {
                                            status_msg = "You cannot afford that price!".to_string();
                                        }
                                        screen_mode = ScreenMode::Shop;
                                        active_haggle = None;
                                    } else if let Ok(bid) = haggle.player_input.parse::<u32>() {
                                        if bid >= haggle.current_asking {
                                            let final_price = haggle.current_asking;
                                            if player.gold >= final_price {
                                                player.gold -= final_price;
                                                add_item_to_inventory(&mut player.inventory, haggle.item.clone());
                                                status_msg = format!("Accepted! Bought for {} gp.", final_price);
                                            } else {
                                                status_msg = "You cannot afford that price!".to_string();
                                            }
                                            screen_mode = ScreenMode::Shop;
                                            active_haggle = None;
                                        } else if bid <= haggle.last_bid {
                                            haggle.insults += 1;
                                            if haggle.insults >= 3 {
                                                status_msg = "The shopkeeper gets angry and kicks you out!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                active_haggle = None;
                                            } else {
                                                haggle.comment = "Stop insulting me with such low offers!".to_string();
                                                haggle.player_input.clear();
                                            }
                                        } else {
                                            let diff = haggle.current_asking - bid;
                                            let adj = diff * rng.gen_range(15..=25) / 100;
                                            haggle.current_asking = (haggle.current_asking - adj).max(haggle.min_price);
                                            haggle.last_bid = bid;
                                            haggle.offers_count += 1;
                                            haggle.player_input.clear();
                                            if haggle.current_asking == haggle.min_price {
                                                haggle.comment = "This is my final offer!".to_string();
                                            } else {
                                                haggle.comment = format!("How about {} gp?", haggle.current_asking);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    if haggle.player_input.len() < 7 {
                                        haggle.player_input.push(c);
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if screen_mode == ScreenMode::BarterSellMenu {
                        if let Some(ref mut haggle) = active_haggle {
                            match key_event.code {
                                KeyCode::Esc => {
                                    screen_mode = ScreenMode::ShopSellMenu;
                                    status_msg = "Transaction cancelled.".to_string();
                                    active_haggle = None;
                                }
                                KeyCode::Backspace => {
                                    haggle.player_input.pop();
                                }
                                KeyCode::Enter => {
                                    if haggle.player_input.is_empty() {
                                        let final_price = haggle.current_asking;
                                        player.gold += final_price;
                                        status_msg = format!("Done! You sold the item for {} gp.", final_price);
                                        
                                        let inv_idx = haggle.item_index;
                                        if inv_idx < player.inventory.len() {
                                            player.inventory[inv_idx].count -= 1;
                                            if player.inventory[inv_idx].count == 0 {
                                                player.inventory.remove(inv_idx);
                                            }
                                        }
                                        screen_mode = ScreenMode::ShopSellMenu;
                                        active_haggle = None;
                                    } else if let Ok(ask) = haggle.player_input.parse::<u32>() {
                                        if ask <= haggle.current_asking {
                                            let final_price = haggle.current_asking;
                                            player.gold += final_price;
                                            status_msg = format!("Accepted! Sold for {} gp.", final_price);
                                            
                                            let inv_idx = haggle.item_index;
                                            if inv_idx < player.inventory.len() {
                                                player.inventory[inv_idx].count -= 1;
                                                if player.inventory[inv_idx].count == 0 {
                                                    player.inventory.remove(inv_idx);
                                                }
                                            }
                                            screen_mode = ScreenMode::ShopSellMenu;
                                            active_haggle = None;
                                        } else if ask >= haggle.last_bid {
                                            haggle.insults += 1;
                                            if haggle.insults >= 3 {
                                                status_msg = "The shopkeeper gets angry at your greed and kicks you out!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                active_haggle = None;
                                            } else {
                                                haggle.comment = "You are asking way too much! Be reasonable!".to_string();
                                                haggle.player_input.clear();
                                            }
                                        } else {
                                            let diff = ask - haggle.current_asking;
                                            let adj = diff * rng.gen_range(15..=25) / 100;
                                            haggle.current_asking = (haggle.current_asking + adj).min(haggle.max_price);
                                            haggle.last_bid = ask;
                                            haggle.offers_count += 1;
                                            haggle.player_input.clear();
                                            if haggle.current_asking == haggle.max_price {
                                                haggle.comment = "That is my absolute final offer!".to_string();
                                            } else {
                                                haggle.comment = format!("I can go up to {} gp.", haggle.current_asking);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    if haggle.player_input.len() < 7 {
                                        haggle.player_input.push(c);
                                    }
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
                                    // Exit browse via ESC
                                } else if screen_mode == ScreenMode::CharacterStatsMenu {
                                    // Exit character sheet via ESC
                                } else if screen_mode == ScreenMode::SelectDisarmDirection {
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
                                        let sx = (player.x as isize + dx) as usize;
                                        let sy = (player.y as isize + dy) as usize;

                                        let mut disarmed = false;
                                        let mut trap_info = None;
                                        if let Some(tile) = level.get_tile(sx, sy) {
                                            if let TileType::Trap { detected, trap_type } = tile.tile_type {
                                                if detected {
                                                    trap_info = Some(trap_type);
                                                }
                                            }
                                        }

                                        if let Some(trap_type) = trap_info {
                                            let is_rogue = player.class == Class::Rogue;
                                            let check_chance = if is_rogue { 0.85 } else { 0.40 };
                                            if rng.gen_bool(check_chance) {
                                                if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                    tile.tile_type = TileType::Floor;
                                                }
                                                status_msg = "You successfully disarmed the trap.".to_string();
                                            } else {
                                                status_msg = "You set off the trap while trying to disarm it!".to_string();
                                                match trap_type {
                                                    TrapType::Arrow => {
                                                        let dmg = rng.gen_range(1..=6);
                                                        player.hp -= dmg;
                                                        status_msg.push_str(&format!(" Arrow hits you for {}!", dmg));
                                                    }
                                                    TrapType::PoisonGas => {
                                                        let dmg = rng.gen_range(2..=8);
                                                        player.hp -= dmg;
                                                        status_msg.push_str(&format!(" Poison gas hits you for {}!", dmg));
                                                    }
                                                    TrapType::Teleport => {
                                                        let dest = find_passable_tile(&level);
                                                        player.move_to(dest.0, dest.1);
                                                        status_msg.push_str(" You are teleported!");
                                                    }
                                                }
                                                if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                    tile.tile_type = TileType::Floor;
                                                }
                                            }
                                            disarmed = true;
                                        }
                                        if !disarmed {
                                            status_msg = "No detected trap in that direction.".to_string();
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
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
                                                    if tile.tile_type == TileType::Wall || tile.tile_type == TileType::SecretDoor {
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
                                                    if tile.tile_type == TileType::Wall || tile.tile_type == TileType::SecretDoor {
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
                                                if player.gold >= price * 9 / 10 {
                                                    let base_p = price;
                                                    let h_item = Item::new(name, 1, match item_type {
                                                        ItemType::Weapon {..} => 10,
                                                        ItemType::Armor {..} => 80,
                                                        ItemType::Potion {..} => 5,
                                                        _ => 2,
                                                    }, item_type.clone());
                                                    
                                                    active_haggle = Some(HaggleState {
                                                        item: h_item,
                                                        item_index: idx,
                                                        initial_price: base_p * 13 / 10,
                                                        min_price: base_p * 9 / 10,
                                                        max_price: base_p * 15 / 10,
                                                        current_asking: base_p * 13 / 10,
                                                        last_bid: base_p * 5 / 10,
                                                        insults: 0,
                                                        offers_count: 0,
                                                        comment: "I will sell this for...".to_string(),
                                                        player_input: String::new(),
                                                    });
                                                    screen_mode = ScreenMode::BarterBuyMenu;
                                                    status_msg = format!("Bartering for {}.", name);
                                                } else {
                                                    status_msg = "You don't have enough gold to make a serious offer!".to_string();
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if screen_mode == ScreenMode::ShopSellMenu {
                                    let idx = c as usize - 'a' as usize;
                                    if idx < player.inventory.len() {
                                        let item = &player.inventory[idx];
                                        let base_val = match item.item_type {
                                            ItemType::Weapon {..} => 25,
                                            ItemType::Armor {..} => 40,
                                            ItemType::Potion {..} => 15,
                                            ItemType::Scroll {..} => 10,
                                        };

                                        active_haggle = Some(HaggleState {
                                            item: item.clone(),
                                            item_index: idx,
                                            initial_price: base_val * 7 / 10,
                                            min_price: base_val * 5 / 10,
                                            max_price: base_val * 11 / 10,
                                            current_asking: base_val * 7 / 10,
                                            last_bid: base_val * 15 / 10,
                                            insults: 0,
                                            offers_count: 0,
                                            comment: "I can pay...".to_string(),
                                            player_input: String::new(),
                                        });
                                        screen_mode = ScreenMode::BarterSellMenu;
                                        status_msg = format!("Bartering to sell {}.", item.name);
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
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
                                    
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
                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
                    } else {
                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref())?;
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
