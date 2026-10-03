use std::io::{self, Write};
use crossterm::execute;
use crate::{GUI_ACTIVE, GUI_STATE_TX, GuiState, ScreenMode};
use crate::dungeon::{DungeonLevel, TileType, ShopType, get_shop_items, HaggleState};
use crate::player::{Player, Item, ItemType, Class, InscribeState, format_stat};
use crate::entity::monster::Monster;

fn get_food_status_str(food: i32) -> &'static str {
    if food < 0 {
        "STARVING"
    } else if food < 300 {
        "FAINT"
    } else if food < 1000 {
        "WEAK"
    } else if food < 2000 {
        "HUNGRY"
    } else if food >= 10000 {
        "FULL"
    } else {
        "FED"
    }
}

pub fn get_bottom_status_line(player: &Player, depth: u32, is_resting: bool) -> String {
    let hunger_str = if player.food < 0 {
        "Starve"
    } else if player.food < 300 {
        "Faint "
    } else if player.food < 1000 {
        "Weak  "
    } else if player.food < 2000 {
        "Hungry"
    } else {
        "      "
    };

    let blind_str = if player.flags.blind > 0 {
        "Blind"
    } else {
        "     "
    };

    let confused_str = if player.flags.confused > 0 {
        "Confused"
    } else {
        "        "
    };

    let afraid_str = if player.flags.afraid > 0 {
        "Afraid"
    } else {
        "      "
    };

    let poisoned_str = if player.flags.poisoned > 0 {
        "Poisoned"
    } else {
        "        "
    };

    let movement_str = if player.flags.paralysis > 0 {
        "Paralysed "
    } else if is_resting {
        "Rest      "
    } else if player.searching {
        "Searching "
    } else {
        "          "
    };

    let is_slow = player.flags.slow > 0 || player.is_encumbered();
    let speed_str = if player.flags.fast > 0 && !is_slow {
        "Fast     "
    } else if is_slow && player.flags.fast == 0 {
        "Slow     "
    } else {
        "         "
    };

    let study_str = if player.new_spells_to_learn > 0 {
        "Study"
    } else {
        "     "
    };

    let recall_str = if player.flags.word_of_recall > 0 {
        "Recall"
    } else {
        "      "
    };

    let depth_str = if depth == 0 {
        "Town".to_string()
    } else {
        format!("{:>4} FT", depth * 50)
    };

    format!(
        "{:<6} {:<5} {:<8} {:<6} {:<8} {:<10} {:<9} {:<5} {:<6} {:>7}",
        hunger_str, blind_str, confused_str, afraid_str, poisoned_str, movement_str, speed_str, study_str, recall_str, depth_str
    )
}

pub fn render_gui_screen(
    level: &mut DungeonLevel,
    player: &Player,
    monsters: &[Monster],
    mode: ScreenMode,
    shop: ShopType,
    active_haggle: Option<&HaggleState>,
    active_inscribe: Option<&InscribeState>,
) -> Vec<String> {
    let mut screen = Vec::new();
    for map_y in 0..level.height {
        let mut map_part = String::new();
        if mode == ScreenMode::Dungeon || mode == ScreenMode::SelectRestCount || mode == ScreenMode::SelectBashDirection || mode == ScreenMode::SelectTunnelDirection || mode == ScreenMode::GameOver {
            for map_x in 0..level.width {
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        map_part.push('@');
                    } else if tile.visible && player.flags.blind == 0 && monsters.iter().any(|m| m.x == map_x && m.y == map_y) {
                        let monster = monsters.iter().find(|m| m.x == map_x && m.y == map_y).unwrap();
                        map_part.push(monster.symbol);
                    } else if (tile.visible || tile.remembered) && level.items.iter().any(|i| i.x == map_x && i.y == map_y) && !matches!(tile.tile_type, TileType::Wall | TileType::SecretDoor | TileType::MagmaVein { .. } | TileType::QuartzVein { .. } | TileType::Rubble) {
                        let floor_item = level.items.iter().find(|i| i.x == map_x && i.y == map_y).unwrap();
                        map_part.push(get_item_symbol(&floor_item.item));
                    } else if tile.visible || tile.remembered {
                        match tile.tile_type {
                            TileType::Wall => map_part.push('#'),
                            TileType::Floor => map_part.push('.'),
                            TileType::DoorClosed { .. } => map_part.push('+'),
                            TileType::DoorOpen => map_part.push('\''),
                            TileType::StairsUp => map_part.push('<'),
                            TileType::StairsDown => map_part.push('>'),
                            TileType::ShopDoor(num) => map_part.push((b'0' + num) as char),
                            TileType::SecretDoor => map_part.push('#'),
                            TileType::MagmaVein { .. } => map_part.push('%'),
                            TileType::QuartzVein { .. } => map_part.push('%'),
                            TileType::Rubble => map_part.push('*'),
                            TileType::Trap { detected, .. } => {
                                if detected {
                                    map_part.push('^');
                                } else {
                                    map_part.push('.');
                                }
                            }
                            TileType::Chest { .. } => map_part.push('&'),
                            TileType::GlyphOfWarding => map_part.push(';'),
                            TileType::Empty => map_part.push(' '),
                        }
                    } else {
                        map_part.push(' ');
                    }
                }
            }
        } else {
            map_part = format!("{:<66}", get_overlay_row_text(map_y, mode, player, shop, active_haggle, active_inscribe));
        }
        screen.push(map_part);
    }
    screen
}

fn get_item_symbol(item: &Item) -> char {
    if item.name.contains("Gold Pile") {
        '*'
    } else if item.name.contains("Spellbook") || item.name.contains("Prayerbook") {
        '?'
    } else {
        match &item.item_type {
            ItemType::Weapon { .. } => ')',
            ItemType::Bow { .. } => '}',
            ItemType::Missile { .. } => '{',
            ItemType::Armor { .. } => '[',
            ItemType::Potion { .. } => '!',
            ItemType::Scroll { .. } => '?',
            ItemType::Wand { .. } => '/',
            ItemType::Staff { .. } => '_',
            ItemType::Food { .. } => ',',
            ItemType::Light { .. } => '~',
            ItemType::Ring { .. } => '=',
            ItemType::Amulet { .. } => '"',
            ItemType::MagicBook { .. } => '?',
            ItemType::PrayerBook { .. } => '?',
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
        6 => format!("STR: {}", format_stat(player.stats.strength)),
        7 => format!("INT: {}", format_stat(player.stats.intelligence)),
        8 => format!("WIS: {}", format_stat(player.stats.wisdom)),
        9 => format!("DEX: {}", format_stat(player.stats.dexterity)),
        10 => format!("CON: {}", format_stat(player.stats.constitution)),
        11 => format!("CHR: {}", format_stat(player.stats.charisma)),
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
        21 => {
            if player.food < 0 {
                "STARVING".to_string()
            } else if player.food < 300 {
                "FAINT".to_string()
            } else if player.food < 1000 {
                "WEAK".to_string()
            } else if player.food < 2000 {
                "HUNGRY".to_string()
            } else if player.food >= 10000 {
                "FULL".to_string()
            } else {
                "".to_string()
            }
        }
        22 => {
            if player.searching {
                "SEARCHING".to_string()
            } else {
                "".to_string()
            }
        }
        _ => "".to_string(),
    };
    
    format!("{:<12}", text)
}

pub fn wrap_text(text: &str, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
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

fn get_overlay_row_text(row: usize, mode: ScreenMode, player: &Player, shop: ShopType, active_haggle: Option<&HaggleState>, active_inscribe: Option<&InscribeState>) -> String {
    match mode {
        ScreenMode::InscribeMenu => {
            if row == 0 {
                return "--- INSCRIBE ITEM ---".to_string();
            }
            if row == player.inventory.len() + 2 {
                return "---------------------".to_string();
            }
            if row == player.inventory.len() + 3 {
                return "Select item letter to inscribe, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= player.inventory.len() {
                let idx = row - 1;
                let item = &player.inventory[idx];
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
            }
            if player.inventory.is_empty() && row == 1 {
                return "(Your inventory is empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::Inscribing => {
            if let Some(state) = active_inscribe {
                let item = if state.is_equipment {
                    &player.equipment[state.item_index]
                } else {
                    &player.inventory[state.item_index]
                };
                match row {
                    0 => "--- INSCRIBE ITEM ---".to_string(),
                    2 => format!("Inscribing: {}", item.name),
                    4 => format!("Enter inscription: {}", state.input),
                    6 => "--------------------------------------------".to_string(),
                    7 => "Type characters to inscribe, then press ENTER.".to_string(),
                    8 => "Press ENTER on blank input to clear inscription.".to_string(),
                    9 => "Press ESC to cancel.".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        }
        ScreenMode::InventoryList => {
            if row == 0 {
                let w_quot = player.total_weight() / 10;
                let w_rem = player.total_weight() % 10;
                let cap_quot = player.carrying_capacity() / 10;
                let cap_rem = player.carrying_capacity() % 10;
                return format!("--- INVENTORY (Carrying {}.{} lbs / Capacity {}.{} lbs) ---", w_quot, w_rem, cap_quot, cap_rem);
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
                    ItemType::Bow { multiplier } => format!("Launcher, x{} dmg", multiplier),
                    ItemType::Missile { damage } => format!("Missile, {}d{} dmg", damage.num, damage.sides),
                    ItemType::Armor { ac } => format!("Armor, +{} AC", ac),
                    ItemType::Potion { heal_amount } => format!("Potion (heals {} HP)", heal_amount),
                    ItemType::Scroll { .. } => {
                        if item.name.contains("Spellbook") || item.name.contains("Prayerbook") {
                            "Spell Book".to_string()
                        } else {
                            "Scroll".to_string()
                        }
                    }
                    ItemType::Wand { charges, .. } => format!("Wand ({} charges)", charges),
                    ItemType::Staff { charges, .. } => format!("Staff ({} charges)", charges),
                    ItemType::Food { nutrition } => format!("Food (nutrition {})", nutrition),
                    ItemType::Light { fuel } => format!("Light ({} turns left)", fuel),
                    ItemType::Ring { bonus } => {
                        if *bonus != 0 {
                            format!("Ring, {:+}", bonus)
                        } else {
                            "Ring".to_string()
                        }
                    }
                    ItemType::Amulet { bonus } => {
                        if *bonus != 0 {
                            format!("Amulet, {:+}", bonus)
                        } else {
                            "Amulet".to_string()
                        }
                    }
                    ItemType::MagicBook { .. } => "Spell Book".to_string(),
                    ItemType::PrayerBook { .. } => "Holy Book".to_string(),
                };
                return format!("{}. {} ({}) x{}", (b'a' + idx as u8) as char, item.display_name(), details, item.count);
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
                let slot = item.get_equipment_slot();
                return format!("{}. {}: {}", (b'a' + idx as u8) as char, slot, item.display_name());
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
                    matches!(item.item_type, ItemType::Weapon {..} | ItemType::Bow {..} | ItemType::Armor {..} | ItemType::Light {..} | ItemType::Ring {..} | ItemType::Amulet {..})
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
                    ItemType::Bow { multiplier } => format!("Launcher, x{} dmg", multiplier),
                    ItemType::Armor { ac } => format!("Armor, +{} AC", ac),
                    ItemType::Light { fuel } => format!("Light Source ({} turns fuel)", fuel),
                    ItemType::Ring { bonus } => {
                        if *bonus != 0 {
                            format!("Ring, {:+}", bonus)
                        } else {
                            "Ring".to_string()
                        }
                    }
                    ItemType::Amulet { bonus } => {
                        if *bonus != 0 {
                            format!("Amulet, {:+}", bonus)
                        } else {
                            "Amulet".to_string()
                        }
                    }
                    _ => "Accessory".to_string(),
                };
                return format!("{}. {} ({})", (b'a' + idx as u8) as char, item.display_name(), details);
            }
            if equippable.is_empty() && row == 1 {
                return "(No equippable weapons, armors, rings, amulets or lights in inventory)".to_string();
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
                return format!("{}. {}", (b'a' + idx as u8) as char, item.display_name());
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
                    if item.identified && heal_amount > 0 {
                        return format!("{}. {} (heals {} HP) x{}", (b'a' + idx as u8) as char, item.display_name(), heal_amount, item.count);
                    } else {
                        return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
                    }
                }
            }
            if potions.is_empty() && row == 1 {
                return "(No potions in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::EatMenu => {
            if row == 0 {
                return "--- EAT FOOD SELECTION ---".to_string();
            }
            let foods: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Food {..}))
                .collect();

            if row == foods.len() + 2 {
                return "------------------------------".to_string();
            }
            if row == foods.len() + 3 {
                return "Select food letter to eat, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= foods.len() {
                let idx = row - 1;
                let (_, item) = foods[idx];
                if let ItemType::Food { nutrition } = item.item_type {
                    return format!("{}. {} (nutrition {}) x{}", (b'a' + idx as u8) as char, item.display_name(), nutrition, item.count);
                }
            }
            if foods.is_empty() && row == 1 {
                return "(No food in inventory)".to_string();
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
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
            }
            if scrolls.is_empty() && row == 1 {
                return "(No scrolls in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::BrowseBookMenu => {
            let is_prayer = player.is_priest_caster();
            if row == 0 {
                return format!("--- BROWSE {} ---", if is_prayer { "HOLY PRAYERS" } else { "MAGIC SPELLS" });
            }
            if row == 1 {
                return format!("  {:<30} {:>2} {:>4} {:>4}", "Name", "Lv", "Mana", "Fail");
            }
            let flags = if is_prayer {
                0x7FFFFFFF
            } else {
                player.carried_spell_flags(false)
            };
            let spells = crate::magic::get_spells_in_book(flags);
            if row >= 2 && row < 2 + spells.len() {
                let idx = row - 2;
                let spell_idx = spells[idx];
                if let Some(def) = crate::magic::get_spell_def(player.class, spell_idx, is_prayer) {
                    let stat_val = if is_prayer { player.stats.wisdom } else { player.stats.intelligence };
                    let fail = crate::magic::spell_chance_of_success(player.level, player.class, stat_val, player.mana, def);
                    let status = if !player.knows_spell(spell_idx) {
                        " unknown"
                    } else if (player.spells_worked & (1 << spell_idx)) == 0 {
                        " untried"
                    } else {
                        ""
                    };
                    let letter = (b'a' + idx as u8) as char;
                    return format!("  {}) {:<27} {:>2} {:>4} {:>3}%{}", letter, def.name, def.level_required, def.mana_required, fail, status);
                }
            }
            if spells.is_empty() && row == 2 {
                return "  (No books carried)".to_string();
            }
            if row == 2 + spells.len() + 1 || (spells.is_empty() && row == 4) {
                return "Press ESC or Space to exit book browser.".to_string();
            }
            "".to_string()
        }
        ScreenMode::CastSpellMenu => {
            if row == 0 {
                return "--- CAST MAGE SPELL ---".to_string();
            }
            if row == 1 {
                return format!("  {:<30} {:>2} {:>4} {:>4}", "Name", "Lv", "Mana", "Fail");
            }
            let known = player.known_spells(false);
            if row >= 2 && row < 2 + known.len() {
                let idx = row - 2;
                let spell_idx = known[idx];
                if let Some(def) = crate::magic::get_spell_def(player.class, spell_idx, false) {
                    let fail = crate::magic::spell_chance_of_success(player.level, player.class, player.stats.intelligence, player.mana, def);
                    let letter = (b'a' + idx as u8) as char;
                    return format!("  {}) {:<27} {:>2} {:>4} {:>3}%", letter, def.name, def.level_required, def.mana_required, fail);
                }
            }
            if known.is_empty() && row == 2 {
                return "  (No spells known)".to_string();
            }
            if row == 2 + known.len() + 1 || (known.is_empty() && row == 4) {
                return "Select spell letter to cast, or press ESC to cancel.".to_string();
            }
            "".to_string()
        }
        ScreenMode::PrayMenu => {
            if row == 0 {
                return "--- RECITE CLERICAL PRAYER ---".to_string();
            }
            if row == 1 {
                return format!("  {:<30} {:>2} {:>4} {:>4}", "Name", "Lv", "Mana", "Fail");
            }
            let known = player.known_spells(true);
            if row >= 2 && row < 2 + known.len() {
                let idx = row - 2;
                let spell_idx = known[idx];
                if let Some(def) = crate::magic::get_spell_def(player.class, spell_idx, true) {
                    let fail = crate::magic::spell_chance_of_success(player.level, player.class, player.stats.wisdom, player.mana, def);
                    let letter = (b'a' + idx as u8) as char;
                    return format!("  {}) {:<27} {:>2} {:>4} {:>3}%", letter, def.name, def.level_required, def.mana_required, fail);
                }
            }
            if known.is_empty() && row == 2 {
                return "  (No prayers known)".to_string();
            }
            if row == 2 + known.len() + 1 || (known.is_empty() && row == 4) {
                return "Select prayer letter to recite, or press ESC to cancel.".to_string();
            }
            "".to_string()
        }
        ScreenMode::StudyMenu => {
            let is_prayer = player.is_priest_caster();
            if row == 0 {
                return format!("--- STUDY / GAIN {}S ({} to learn) ---", if is_prayer { "PRAYER" } else { "SPELL" }, player.new_spells_to_learn);
            }
            if row == 1 {
                return format!("  {:<30} {:>2} {:>4} {:>4}", "Name", "Lv", "Mana", "Fail");
            }
            let learnable = player.learnable_spells();
            if row >= 2 && row < 2 + learnable.len() {
                let idx = row - 2;
                let spell_idx = learnable[idx];
                if let Some(def) = crate::magic::get_spell_def(player.class, spell_idx, is_prayer) {
                    let stat_val = if is_prayer { player.stats.wisdom } else { player.stats.intelligence };
                    let fail = crate::magic::spell_chance_of_success(player.level, player.class, stat_val, player.mana, def);
                    let letter = (b'a' + idx as u8) as char;
                    return format!("  {}) {:<27} {:>2} {:>4} {:>3}%", letter, def.name, def.level_required, def.mana_required, fail);
                }
            }
            if learnable.is_empty() && row == 2 {
                return "  (No learnable spells available)".to_string();
            }
            if row == 2 + learnable.len() + 1 || (learnable.is_empty() && row == 4) {
                return "Select letter to study, or press ESC to cancel.".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectSpellDirection => {
            if row == 0 {
                return "--- CASTING TARGETING ---".to_string();
            }
            if row == 2 {
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c) or ESC to cancel:".to_string();
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
                5 => format!("  STR: {:>6}   INT: {:>6}   WIS: {:>6}", format_stat(player.stats.strength), format_stat(player.stats.intelligence), format_stat(player.stats.wisdom)),
                6 => format!("  DEX: {:>6}   CON: {:>6}   CHR: {:>6}", format_stat(player.stats.dexterity), format_stat(player.stats.constitution), format_stat(player.stats.charisma)),
                8 => "Combat Stats:".to_string(),
                9 => format!("  HP:   {}/{}       Mana: {}/{}", player.hp, player.max_hp, player.mana, player.max_mana),
                10 => format!("  AC:   {:<10} Gold: {} gp", player.calculate_ac(), player.gold),
                11 => format!("  Level: {:<9} Exp:  {} exp", player.level, player.exp),
                12 => format!("  Weight: {}.{} lbs   Capacity: {}.{} lbs", player.total_weight() / 10, player.total_weight() % 10, player.carrying_capacity() / 10, player.carrying_capacity() % 10),
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
                    5 => match haggle.last_bid {
                        Some(bid) => format!("Your last offer:    {} gp", bid),
                        None => "Your last offer:    None".to_string(),
                    },
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
                    5 => match haggle.last_bid {
                        Some(ask) => format!("Your last ask:      {} gp", ask),
                        None => "Your last ask:      None".to_string(),
                    },
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
        ScreenMode::SelectOpenDirection => {
            if row == 0 {
                return "--- OPEN DOOR ---".to_string();
            }
            if row == 2 {
                return "Choose direction to open a door (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectLookDirection => {
            if row == 0 {
                return "--- LOOK ---".to_string();
            }
            if row == 2 {
                return "Choose direction to look (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectCloseDirection => {
            if row == 0 {
                return "--- CLOSE DOOR ---".to_string();
            }
            if row == 2 {
                return "Choose direction to close a door (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::DropMenu => {
            if row == 0 {
                return "--- DROP ITEM ---".to_string();
            }
            if row == player.inventory.len() + 2 {
                return "-----------------".to_string();
            }
            if row == player.inventory.len() + 3 {
                return "Select item letter to drop, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= player.inventory.len() {
                let idx = row - 1;
                let item = &player.inventory[idx];
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
            }
            if player.inventory.is_empty() && row == 1 {
                return "(Your inventory is empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::IdentifyMenu => {
            if row == 0 {
                return "--- IDENTIFY ITEM SELECTION ---".to_string();
            }
            if row == player.inventory.len() + 2 {
                return "-------------------------------".to_string();
            }
            if row == player.inventory.len() + 3 {
                return "Select item letter to identify, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= player.inventory.len() {
                let idx = row - 1;
                let item = &player.inventory[idx];
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
            }
            if player.inventory.is_empty() && row == 1 {
                return "(Your inventory is empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectThrowItem => {
            if row == 0 {
                return "--- FIRE / THROW ITEM SELECTION ---".to_string();
            }
            if row == player.inventory.len() + 2 {
                return "-----------------------------------".to_string();
            }
            if row == player.inventory.len() + 3 {
                return "Select item letter to throw/fire, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= player.inventory.len() {
                let idx = row - 1;
                let item = &player.inventory[idx];
                return format!("{}. {} x{}", (b'a' + idx as u8) as char, item.display_name(), item.count);
            }
            if player.inventory.is_empty() && row == 1 {
                return "(Your inventory is empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectThrowDirection => {
            if row == 0 {
                return "--- THROW / FIRE TARGETING ---".to_string();
            }
            if row == 2 {
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::HelpMenu => {
            let help_lines = &[
                "--- MORIA COMMANDS & KEY BINDINGS ---",
                "",
                "Movement / Rest:          q w e     (North-West / North / North-East)",
                "                          a s d     (West / REST / East)",
                "                          z x c     (South-West / South / South-East)",
                "",
                "Command Keys (Standard Profile):",
                "  h : Quaff Potion        W : Wear/Wield equipment",
                "  i : Inventory List      I : Equipment List",
                "  k : Search surrounding  D : Drop an item",
                "  o : Open closed door    C : Close open door",
                "  n : Disarm a trap       H : Character Sheet Stats",
                "  r : Read scroll         T : Take off equipment",
                "  f/t : Fire / Throw item B : Bash door/monster",
                "  b : Browse spellbook    m : Cast Mage spell",
                "  p : Recite Priest prayer G : Gain spells (study)",
                "  < : Climb up stairs     > : Climb down stairs",
                "  g : Tunnel rock/vein    F : Refill light with oil",
                "  Q : Save and Quit       ? : This Help Menu",
                "",
                "Press ESC or any key to return to the game."
            ];
            if row < help_lines.len() {
                help_lines[row].to_string()
            } else {
                "".to_string()
            }
        }
        ScreenMode::AimWandMenu => {
            if row == 0 {
                return "--- AIM WAND SELECTION ---".to_string();
            }
            let wands: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Wand {..}))
                .collect();

            if row == wands.len() + 2 {
                return "--------------------------".to_string();
            }
            if row == wands.len() + 3 {
                return "Select wand letter to aim, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= wands.len() {
                let idx = row - 1;
                let (_, item) = wands[idx];
                if let ItemType::Wand { charges, .. } = item.item_type {
                    return format!("{}. {} ({} charges)", (b'a' + idx as u8) as char, item.display_name(), charges);
                }
            }
            if wands.is_empty() && row == 1 {
                return "(No wands in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectWandDirection => {
            if row == 0 {
                return "--- AIM WAND TARGETING ---".to_string();
            }
            if row == 2 {
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectJamDirection => {
            if row == 0 {
                return "--- JAM DOOR ---".to_string();
            }
            if row == 2 {
                return "Choose direction to jam using movement keys (q/w/e/a/d/z/x/c):".to_string();
            }
            "".to_string()
        }
        ScreenMode::UseStaffMenu => {
            if row == 0 {
                return "--- USE STAFF SELECTION ---".to_string();
            }
            let staves: Vec<(usize, &Item)> = player.inventory.iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Staff {..}))
                .collect();

            if row == staves.len() + 2 {
                return "---------------------------".to_string();
            }
            if row == staves.len() + 3 {
                return "Select staff letter to use, or press ESC to cancel.".to_string();
            }
            if row > 0 && row <= staves.len() {
                let idx = row - 1;
                let (_, item) = staves[idx];
                if let ItemType::Staff { charges, .. } = item.item_type {
                    return format!("{}. {} ({} charges)", (b'a' + idx as u8) as char, item.display_name(), charges);
                }
            }
            if staves.is_empty() && row == 1 {
                return "(No staves in inventory)".to_string();
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
                return "Press item letter (a-l) to buy/haggle, 's' to Sell, ESC to exit.".to_string();
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
                    ItemType::Bow {..} => 25,
                    ItemType::Missile {..} => 3,
                    ItemType::Armor {..} => 40,
                    ItemType::Potion {..} => 15,
                    ItemType::Scroll {..} => 10,
                    ItemType::Wand {..} => 50,
                    ItemType::Staff {..} => 60,
                    ItemType::Food {..} => 5,
                    ItemType::Light {..} => 15,
                    ItemType::Ring {..} => 80,
                    ItemType::Amulet {..} => 80,
                    ItemType::MagicBook {..} => 30,
                    ItemType::PrayerBook {..} => 30,
                };
                return format!("{}. {} (sells for {} gp) x{}", (b'a' + idx as u8) as char, item.display_name(), value, item.count);
            }
            if player.inventory.is_empty() && row == 2 {
                return "(Your inventory is completely empty)".to_string();
            }
            "".to_string()
        }
        ScreenMode::Dungeon => "".to_string(),
        ScreenMode::SelectRestCount => "".to_string(),
        ScreenMode::SelectBashDirection => "".to_string(),
        ScreenMode::SelectTunnelDirection => "".to_string(),
        ScreenMode::GameOver => "".to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_map(
    level: &mut DungeonLevel,
    player: &Player,
    monsters: &[Monster],
    status_msg: &str,
    mode: ScreenMode,
    shop: ShopType,
    active_haggle: Option<&HaggleState>,
    active_inscribe: Option<&InscribeState>,
) -> Result<(), io::Error> {
    level.update_fov(player);

    if GUI_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        let screen = render_gui_screen(level, player, monsters, mode, shop, active_haggle, active_inscribe);
        let player_json = serde_json::json!({
            "name": player.name,
            "race": format!("{:?}", player.race),
            "class": format!("{:?}", player.class),
            "level": player.level,
            "exp": player.exp,
            "gold": player.gold,
            "hp": player.hp,
            "max_hp": player.max_hp,
            "mana": player.mana,
            "max_mana": player.max_mana,
            "food_status": get_food_status_str(player.food),
            "searching": player.searching,
            "depth": level.depth,
            "screen_mode": format!("{:?}", mode),
            "stats": {
                "strength": player.stats.strength,
                "intelligence": player.stats.intelligence,
                "wisdom": player.stats.wisdom,
                "dexterity": player.stats.dexterity,
                "constitution": player.stats.constitution,
                "charisma": player.stats.charisma,
            },
            "max_stats": {
                "strength": player.max_stats.strength,
                "intelligence": player.max_stats.intelligence,
                "wisdom": player.max_stats.wisdom,
                "dexterity": player.max_stats.dexterity,
                "constitution": player.max_stats.constitution,
                "charisma": player.max_stats.charisma,
            },
            "status_effects": {
                "blind": player.flags.blind > 0,
                "confused": player.flags.confused > 0,
                "afraid": player.flags.afraid > 0,
                "poisoned": player.flags.poisoned > 0,
                "paralyzed": player.flags.paralysis > 0,
                "fast": player.flags.fast > 0,
                "slow": player.flags.slow > 0,
                "word_of_recall": player.flags.word_of_recall > 0,
                "heroism": player.flags.heroism > 0,
                "blessed": player.flags.blessed > 0,
                "protect_evil": player.flags.protect_evil > 0,
                "invulnerable": player.flags.invulnerability > 0,
            }
        });
        if let Some(tx_mutex) = GUI_STATE_TX.get() {
            let tx = tx_mutex.lock().unwrap();
            let _ = tx.send(GuiState {
                screen,
                status_msg: status_msg.to_string(),
                player_json,
            });
        }
        return Ok(());
    }

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 80) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 24) / 2).max(0) as u16;

    let msg_line = format!("Message: {}\x1b[K", status_msg);
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y));
    print!("{}", msg_line);

    for map_y in 0..level.height {
        let stats_part = get_lhs_stat_line(map_y + 1, player, level);
        let mut map_part = String::new();

        if mode == ScreenMode::Dungeon || mode == ScreenMode::SelectRestCount || mode == ScreenMode::SelectBashDirection || mode == ScreenMode::SelectTunnelDirection || mode == ScreenMode::GameOver {
            for map_x in 0..level.width {
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        map_part.push('@');
                    } else if tile.visible && player.flags.blind == 0 && monsters.iter().any(|m| m.x == map_x && m.y == map_y) {
                        let monster = monsters.iter().find(|m| m.x == map_x && m.y == map_y).unwrap();
                        map_part.push(monster.symbol);
                    } else if (tile.visible || tile.remembered) && level.items.iter().any(|i| i.x == map_x && i.y == map_y) && !matches!(tile.tile_type, TileType::Wall | TileType::SecretDoor | TileType::MagmaVein { .. } | TileType::QuartzVein { .. } | TileType::Rubble) {
                        let floor_item = level.items.iter().find(|i| i.x == map_x && i.y == map_y).unwrap();
                        map_part.push(get_item_symbol(&floor_item.item));
                    } else if tile.visible || tile.remembered {
                        match tile.tile_type {
                            TileType::Wall => map_part.push('#'),
                            TileType::Floor => map_part.push('.'),
                            TileType::DoorClosed { .. } => map_part.push('+'),
                            TileType::DoorOpen => map_part.push('\''),
                            TileType::StairsUp => map_part.push('<'),
                            TileType::StairsDown => map_part.push('>'),
                            TileType::ShopDoor(num) => map_part.push((b'0' + num) as char),
                            TileType::SecretDoor => map_part.push('#'),
                            TileType::MagmaVein { .. } => map_part.push('%'),
                            TileType::QuartzVein { .. } => map_part.push('%'),
                            TileType::Rubble => map_part.push('*'),
                            TileType::Trap { detected, .. } => {
                                if detected {
                                    map_part.push('^');
                                } else {
                                    map_part.push('.');
                                }
                            }
                            TileType::Chest { .. } => map_part.push('&'),
                            TileType::GlyphOfWarding => map_part.push(';'),
                            TileType::Empty => map_part.push(' '),
                        }
                    } else {
                        map_part.push(' ');
                    }
                }
            }
        } else {
            map_part = format!("{:<66}", get_overlay_row_text(map_y, mode, player, shop, active_haggle, active_inscribe));
        }

        let row_line = format!("{} {}\x1b[K", stats_part, map_part);
        let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 1 + map_y as u16));
        print!("{}", row_line);
    }
    
    let status_bar = get_bottom_status_line(player, level.depth, false);
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 23));
    print!("{}\x1b[K", status_bar);
    
    io::stdout().flush()?;
    Ok(())
}
