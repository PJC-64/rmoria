use crate::dungeon::{DungeonLevel, HaggleState, ShopType, TileType, get_shop_items};
use crate::entity::monster::Monster;
use crate::player::{Class, InscribeState, Item, ItemType, Player, format_stat};
use crate::{GUI_ACTIVE, GUI_STATE_TX, GuiState, ScreenMode};
use crossterm::execute;
use std::io::{self, Write};

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

    let halluc_str = if player.flags.image > 0 {
        "Halluc"
    } else {
        "      "
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

    let speed_str = player.speed_status_string();

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
        "{:<6} {:<5} {:<8} {:<6} {:<8} {:<6} {:<10} {:<9} {:<5} {:<6} {:>7}",
        hunger_str,
        blind_str,
        confused_str,
        afraid_str,
        poisoned_str,
        halluc_str,
        movement_str,
        speed_str,
        study_str,
        recall_str,
        depth_str
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
    let (center_x, center_y) = if mode == ScreenMode::LocateMap {
        let cx = (player.x as isize + player.locate_offset.0)
            .clamp(0, level.width.saturating_sub(1) as isize) as usize;
        let cy = (player.y as isize + player.locate_offset.1)
            .clamp(0, level.height.saturating_sub(1) as isize) as usize;
        (cx, cy)
    } else {
        (player.x, player.y)
    };
    let (view_top, view_bottom, view_left, view_right) =
        level.get_viewport_bounds(center_x, center_y);
    let view_h = view_bottom.saturating_sub(view_top);
    let mut screen = Vec::new();
    for screen_y in 0..view_h {
        let map_y = view_top + screen_y;
        let mut map_part = String::new();
        if mode == ScreenMode::Dungeon
            || mode == ScreenMode::SelectRestCount
            || mode == ScreenMode::SelectBashDirection
            || mode == ScreenMode::SelectTunnelDirection
            || mode == ScreenMode::LocateMap
        {
            for map_x in view_left..view_right {
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        map_part.push('@');
                    } else if tile.visible
                        && player.flags.blind == 0
                        && monsters.iter().any(|m| m.x == map_x && m.y == map_y)
                    {
                        let monster = monsters
                            .iter()
                            .find(|m| m.x == map_x && m.y == map_y)
                            .unwrap();
                        if player.flags.image > 0 {
                            let mut rng = rand::thread_rng();
                            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
                            map_part.push(
                                chars[rand::Rng::gen_range(&mut rng, 0..chars.len())] as char,
                            );
                        } else {
                            map_part.push(monster.symbol);
                        }
                    } else if (tile.visible || tile.remembered)
                        && level.items.iter().any(|i| i.x == map_x && i.y == map_y)
                        && !matches!(
                            tile.tile_type,
                            TileType::Wall
                                | TileType::SecretDoor
                                | TileType::MagmaVein { .. }
                                | TileType::QuartzVein { .. }
                                | TileType::Rubble
                                | TileType::StairsUp
                                | TileType::StairsDown
                        )
                    {
                        let floor_item = level
                            .items
                            .iter()
                            .find(|i| i.x == map_x && i.y == map_y)
                            .unwrap();
                        if player.flags.image > 0 {
                            let mut rng = rand::thread_rng();
                            let chars = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
                            map_part.push(
                                chars[rand::Rng::gen_range(&mut rng, 0..chars.len())] as char,
                            );
                        } else {
                            map_part.push(get_item_symbol(&floor_item.item));
                        }
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
            map_part = format!(
                "{:<66}",
                get_overlay_row_text(
                    screen_y,
                    mode,
                    player,
                    shop,
                    active_haggle,
                    active_inscribe,
                    level
                )
            );
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
    let rank = if player.total_winner || player.balrog_killed {
        "Magnificent"
    } else {
        player.rank_title()
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
        14 => format!("EXP:  {:>5}", player.exp),
        15 => format!("MANA: {:>2}/{:<2}", player.mana, player.max_mana),
        16 => format!("MHP:  {:>5}", player.max_hp),
        17 => format!("CHP:  {:>5}", player.hp),
        18 => format!("AC:   {:>5}", player.calculate_ac()),
        19 => format!("GOLD: {:>5}", player.gold),
        21 => {
            if player.total_winner || player.balrog_killed {
                "*Winner*".to_string()
            } else {
                let feet = level.depth * 50;
                if feet == 0 {
                    "Town Level".to_string()
                } else {
                    format!("{} feet", feet)
                }
            }
        }
        22 => {
            if player.searching {
                "SEARCHING".to_string()
            } else if player.food < 0 {
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

pub const TOMBSTONE_ASCII: [&str; 20] = [
    "               _______________________",
    "              /                       \\         ___",
    "             /                         \\ ___   /   \\      ___",
    "            /            RIP            \\   \\  :   :     /   \\",
    "           /                             \\  : _;,,,;_    :   :",
    "          /                               \\,;_          _;,,,;_",
    "         |               the               |   ___",
    "         |                                 |  /   \\",
    "         |                                 |  :   :",
    "         |                                 | _;,,,;_   ____",
    "         |            Level :              |          /    \\",
    "         |                                 |          :    :",
    "         |                                 |          :    :",
    "         |        Died on Level :          |         _;,,,,;_",
    "         |            killed by            |",
    "         |                                 |",
    "         |                                 |",
    "        *|   *     *     *    *   *     *  | *",
    "________)/\\_)_/___(\\/___(//_\\)/_\\//__\\\\(/_|_)_______",
    "",
];

pub const ROYAL_ASCII: [&str; 19] = [
    "                                     #",
    "                                   #####",
    "                                     #",
    "                               ,,,  $$$  ,,,",
    "                           ,,=$   \"$$$$$\"   $=,,",
    "                         ,$$        $$$        $$,",
    "                         *>         <*>         <*",
    "                         $$         $$$         $$",
    "                         \"$$        $$$        $$\"",
    "                          \"$$       $$$       $$\"",
    "                           *#########*#########*",
    "                           *#########*#########*",
    "",
    "                             Veni, Vidi, Vici!",
    "                        I came, I saw, I conquered!",
    "                         All Hail the Mighty",
    "",
    "",
    "                   [ Press any key to continue ]",
];

pub fn chrono_or_simple_date() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = now / 86400;
    let mut year = 1970;
    let mut d = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d >= days_in_year {
            d -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &month_days {
        if d >= md {
            d -= md;
            month += 1;
        } else {
            break;
        }
    }
    let day = d + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn get_tombstone_row_text(row: usize, player: &Player) -> String {
    let center_at = |base_line: &str, text: &str, target_center: usize| -> String {
        let text_len = text.len();
        let start = target_center.saturating_sub(text_len / 2);
        let mut chars: Vec<char> = base_line.chars().collect();
        while chars.len() < 66 {
            chars.push(' ');
        }
        for (i, c) in text.chars().enumerate() {
            if start + i < chars.len() {
                chars[start + i] = c;
            }
        }
        chars.into_iter().collect()
    };

    let title = if player.total_winner || player.balrog_killed {
        "Magnificent"
    } else {
        player.rank_title()
    };

    let class_title = if player.total_winner || player.balrog_killed {
        match player.gender {
            crate::player::Gender::Male => "*King*",
            crate::player::Gender::Female => "*Queen*",
        }
    } else {
        match player.class {
            Class::Warrior => "Warrior",
            Class::Mage => "Mage",
            Class::Priest => "Priest",
            Class::Rogue => "Rogue",
            Class::Ranger => "Ranger",
            Class::Paladin => "Paladin",
        }
    };

    let died_str = if player.died_from.is_empty() {
        "Quitting"
    } else {
        &player.died_from
    };

    match row {
        0..=3 => TOMBSTONE_ASCII[row].to_string(),
        4 => center_at(TOMBSTONE_ASCII[4], &player.name, 26),
        5 => TOMBSTONE_ASCII[5].to_string(),
        6 => center_at(TOMBSTONE_ASCII[6], title, 26),
        7 => TOMBSTONE_ASCII[7].to_string(),
        8 => center_at(TOMBSTONE_ASCII[8], class_title, 26),
        9 => center_at(TOMBSTONE_ASCII[9], &player.level.to_string(), 32),
        10 => center_at(TOMBSTONE_ASCII[10], &format!("{} Exp", player.exp), 26),
        11 => center_at(TOMBSTONE_ASCII[11], &format!("{} Au", player.gold), 26),
        12 => {
            let depth_str = if player.max_depth_reached == 0 {
                "Town".to_string()
            } else {
                player.max_depth_reached.to_string()
            };
            center_at(TOMBSTONE_ASCII[12], &depth_str, 35)
        }
        13 => TOMBSTONE_ASCII[13].to_string(),
        14 => center_at(TOMBSTONE_ASCII[14], died_str, 26),
        15 => {
            let date = chrono_or_simple_date();
            center_at(TOMBSTONE_ASCII[15], &date, 26)
        }
        16..=18 => TOMBSTONE_ASCII[row].to_string(),
        19 => "".to_string(),
        20 => "Character record? [y: dump to rmoria_char.txt, ESC/other: continue]".to_string(),
        _ => "".to_string(),
    }
}

fn get_coronation_row_text(row: usize, player: &Player) -> String {
    if row == 16 {
        let title = match player.gender {
            crate::player::Gender::Male => "*King*",
            crate::player::Gender::Female => "*Queen*",
        };
        format!("{:^66}", format!("{} {}!", title, player.name))
    } else if row < ROYAL_ASCII.len() {
        format!("{:^66}", ROYAL_ASCII[row])
    } else {
        "".to_string()
    }
}

pub fn get_symbol_description(command: char) -> &'static str {
    match command {
        ' ' => "  - An open pit or unseen space.",
        '!' => "! - A potion.",
        '"' => "\" - An amulet, periapt, or necklace.",
        '#' => "# - A stone wall.",
        '$' => "$ - Treasure.",
        '%' => "% - A magma or quartz vein.",
        '&' => "& - Treasure chest.",
        '\'' => "' - An open door.",
        '(' => "( - Soft armor.",
        ')' => ") - A shield or weapon.",
        '*' => "* - Gems or rubble.",
        '+' => "+ - A closed door.",
        ',' => ", - Food or mushroom patch.",
        '-' => "- - A wand.",
        '.' => ". - Floor.",
        '/' => "/ - A pole weapon.",
        '1' => "1 - Entrance to General Store.",
        '2' => "2 - Entrance to Armory.",
        '3' => "3 - Entrance to Weaponsmith.",
        '4' => "4 - Entrance to Temple.",
        '5' => "5 - Entrance to Alchemy shop.",
        '6' => "6 - Entrance to Magic-Users store.",
        ':' => ": - Rubble.",
        ';' => "; - A loose rock or glyph of warding.",
        '<' => "< - An up staircase.",
        '=' => "= - A ring.",
        '>' => "> - A down staircase.",
        '?' => "? - A scroll or book.",
        '@' => "@ - The player.",
        'A' => "A - Giant Ant Lion.",
        'B' => "B - The Balrog.",
        'C' => "C - Gelatinous Cube.",
        'D' => "D - An Ancient Dragon (Beware).",
        'E' => "E - Elemental.",
        'F' => "F - Giant Fly.",
        'G' => "G - Ghost.",
        'H' => "H - Hobgoblin.",
        'I' => "I - Invisible Stalker.",
        'J' => "J - Jelly.",
        'K' => "K - Killer Beetle.",
        'L' => "L - Lich.",
        'M' => "M - Mummy.",
        'N' => "N - Naga.",
        'O' => "O - Ogre.",
        'P' => "P - Giant.",
        'Q' => "Q - Quylthulg.",
        'R' => "R - Reptile.",
        'S' => "S - Scorpion.",
        'T' => "T - Troll.",
        'U' => "U - Umber Hulk.",
        'V' => "V - Vampire.",
        'W' => "W - Wight or Wraith.",
        'X' => "X - Xorn.",
        'Y' => "Y - Yeti.",
        'Z' => "Z - Zephyr Hound.",
        'a' => "a - Giant Ant.",
        'b' => "b - Giant Bat.",
        'c' => "c - Giant Centipede.",
        'd' => "d - Dragon.",
        'e' => "e - Floating Eye.",
        'f' => "f - Giant Frog.",
        'g' => "g - Golem.",
        'h' => "h - Harpy.",
        'i' => "i - Icky Thing.",
        'j' => "j - Jackal.",
        'k' => "k - Kobold.",
        'l' => "l - Giant Louse.",
        'm' => "m - Mold.",
        'n' => "n - Naga.",
        'o' => "o - Orc.",
        'p' => "p - Person.",
        'q' => "q - Quadruped.",
        'r' => "r - Giant Rat.",
        's' => "s - Skeleton.",
        't' => "t - Giant Tick.",
        'u' => "u - Minor demon.",
        'v' => "v - Vortex.",
        'w' => "w - Worm or Worm Mass.",
        'y' => "y - Yeek.",
        'z' => "z - Zombie.",
        _ => "Unknown symbol.",
    }
}

fn get_reduced_map_row_text(row: usize, level: &DungeonLevel, player: &Player) -> String {
    if row == 0 || row == 21 {
        let mut s = String::with_capacity(66);
        s.push('+');
        for _ in 0..64 {
            s.push('-');
        }
        s.push('+');
        return s;
    }

    if row > 21 {
        return "".to_string();
    }

    let mut line = String::with_capacity(66);
    line.push('|');

    let map_y_start = (row - 1) * level.height / 20;
    let map_y_end = row * level.height / 20;

    for col in 0..64 {
        let map_x_start = col * level.width / 64;
        let map_x_end = (col + 1) * level.width / 64;

        let mut best_ch = ' ';
        let mut best_priority = -20;

        for y in map_y_start..map_y_end.min(level.height) {
            for x in map_x_start..map_x_end.min(level.width) {
                if x == player.x && y == player.y {
                    if 10 > best_priority {
                        best_priority = 10;
                        best_ch = '@';
                    }
                } else if let Some(tile) = level.get_tile(x, y)
                    && (tile.visible || tile.remembered)
                {
                    let (ch, prio) = match tile.tile_type {
                        TileType::StairsUp => ('<', 5),
                        TileType::StairsDown => ('>', 5),
                        TileType::Wall => ('#', -5),
                        TileType::Floor => ('.', -10),
                        TileType::DoorClosed { .. } => ('+', 0),
                        TileType::DoorOpen => ('\'', 0),
                        TileType::SecretDoor => ('#', -5),
                        TileType::MagmaVein { .. } | TileType::QuartzVein { .. } => ('%', 0),
                        TileType::Rubble => ('*', 0),
                        TileType::Trap { detected, .. } => {
                            if detected {
                                ('^', 0)
                            } else {
                                ('.', -10)
                            }
                        }
                        TileType::Chest { .. } => ('&', 0),
                        TileType::GlyphOfWarding => (';', 0),
                        TileType::ShopDoor(id) => ((b'1' + id.min(5)) as char, 0),
                        TileType::Empty => (' ', -15),
                    };
                    if prio > best_priority {
                        best_priority = prio;
                        best_ch = ch;
                    }
                }
            }
        }
        line.push(best_ch);
    }
    line.push('|');
    line
}

fn get_overlay_row_text(
    row: usize,
    mode: ScreenMode,
    player: &Player,
    shop: ShopType,
    active_haggle: Option<&HaggleState>,
    active_inscribe: Option<&InscribeState>,
    level: &DungeonLevel,
) -> String {
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
                return format!(
                    "{}. {} x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    item.count
                );
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
                return format!(
                    "--- INVENTORY (Carrying {}.{} lbs / Capacity {}.{} lbs) ---",
                    w_quot, w_rem, cap_quot, cap_rem
                );
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
                    ItemType::Weapon { damage } => {
                        format!("Weapon, {}d{} dmg", damage.num, damage.sides)
                    }
                    ItemType::Bow { multiplier } => format!("Launcher, x{} dmg", multiplier),
                    ItemType::Missile { damage } => {
                        format!("Missile, {}d{} dmg", damage.num, damage.sides)
                    }
                    ItemType::Armor { ac } => format!("Armor, +{} AC", ac),
                    ItemType::Potion { heal_amount } => {
                        format!("Potion (heals {} HP)", heal_amount)
                    }
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
                return format!(
                    "{}. {} ({}) x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    details,
                    item.count
                );
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
                return format!(
                    "{}. {}: {}",
                    (b'a' + idx as u8) as char,
                    slot,
                    item.display_name()
                );
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
            let equippable: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    matches!(
                        item.item_type,
                        ItemType::Weapon { .. }
                            | ItemType::Bow { .. }
                            | ItemType::Armor { .. }
                            | ItemType::Light { .. }
                            | ItemType::Ring { .. }
                            | ItemType::Amulet { .. }
                    )
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
                    ItemType::Weapon { damage } => {
                        format!("Weapon, {}d{} dmg", damage.num, damage.sides)
                    }
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
                return format!(
                    "{}. {} ({})",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    details
                );
            }
            if equippable.is_empty() && row == 1 {
                return "(No equippable weapons, armors, rings, amulets or lights in inventory)"
                    .to_string();
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
            let potions: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Potion { .. }))
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
                        return format!(
                            "{}. {} (heals {} HP) x{}",
                            (b'a' + idx as u8) as char,
                            item.display_name(),
                            heal_amount,
                            item.count
                        );
                    } else {
                        return format!(
                            "{}. {} x{}",
                            (b'a' + idx as u8) as char,
                            item.display_name(),
                            item.count
                        );
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
            let foods: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Food { .. }))
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
                    return format!(
                        "{}. {} (nutrition {}) x{}",
                        (b'a' + idx as u8) as char,
                        item.display_name(),
                        nutrition,
                        item.count
                    );
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
            let scrolls: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    matches!(item.item_type, ItemType::Scroll { .. })
                        && !item.name.contains("Torch")
                        && !item.name.contains("Spellbook")
                        && !item.name.contains("Prayerbook")
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
                return format!(
                    "{}. {} x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    item.count
                );
            }
            if scrolls.is_empty() && row == 1 {
                return "(No scrolls in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::BrowseBookMenu => {
            let is_prayer = player.is_priest_caster();
            if row == 0 {
                return format!(
                    "--- BROWSE {} ---",
                    if is_prayer {
                        "HOLY PRAYERS"
                    } else {
                        "MAGIC SPELLS"
                    }
                );
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
                    let stat_val = if is_prayer {
                        player.stats.wisdom
                    } else {
                        player.stats.intelligence
                    };
                    let fail = crate::magic::spell_chance_of_success(
                        player.level,
                        player.class,
                        stat_val,
                        player.mana,
                        def,
                    );
                    let status = if !player.knows_spell(spell_idx) {
                        " unknown"
                    } else if (player.spells_worked & (1 << spell_idx)) == 0 {
                        " untried"
                    } else {
                        ""
                    };
                    let letter = (b'a' + idx as u8) as char;
                    return format!(
                        "  {}) {:<27} {:>2} {:>4} {:>3}%{}",
                        letter, def.name, def.level_required, def.mana_required, fail, status
                    );
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
                    let fail = crate::magic::spell_chance_of_success(
                        player.level,
                        player.class,
                        player.stats.intelligence,
                        player.mana,
                        def,
                    );
                    let letter = (b'a' + idx as u8) as char;
                    return format!(
                        "  {}) {:<27} {:>2} {:>4} {:>3}%",
                        letter, def.name, def.level_required, def.mana_required, fail
                    );
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
                    let fail = crate::magic::spell_chance_of_success(
                        player.level,
                        player.class,
                        player.stats.wisdom,
                        player.mana,
                        def,
                    );
                    let letter = (b'a' + idx as u8) as char;
                    return format!(
                        "  {}) {:<27} {:>2} {:>4} {:>3}%",
                        letter, def.name, def.level_required, def.mana_required, fail
                    );
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
                return format!(
                    "--- STUDY / GAIN {}S ({} to learn) ---",
                    if is_prayer { "PRAYER" } else { "SPELL" },
                    player.new_spells_to_learn
                );
            }
            if row == 1 {
                return format!("  {:<30} {:>2} {:>4} {:>4}", "Name", "Lv", "Mana", "Fail");
            }
            let learnable = player.learnable_spells();
            if row >= 2 && row < 2 + learnable.len() {
                let idx = row - 2;
                let spell_idx = learnable[idx];
                if let Some(def) = crate::magic::get_spell_def(player.class, spell_idx, is_prayer) {
                    let stat_val = if is_prayer {
                        player.stats.wisdom
                    } else {
                        player.stats.intelligence
                    };
                    let fail = crate::magic::spell_chance_of_success(
                        player.level,
                        player.class,
                        stat_val,
                        player.mana,
                        def,
                    );
                    let letter = (b'a' + idx as u8) as char;
                    return format!(
                        "  {}) {:<27} {:>2} {:>4} {:>3}%",
                        letter, def.name, def.level_required, def.mana_required, fail
                    );
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
                2 => format!(
                    "Race:      {:?}      Class:      {:?}",
                    player.race, player.class
                ),
                4 => "Attributes:".to_string(),
                5 => format!(
                    "  STR: {:>6}   INT: {:>6}   WIS: {:>6}",
                    format_stat(player.stats.strength),
                    format_stat(player.stats.intelligence),
                    format_stat(player.stats.wisdom)
                ),
                6 => format!(
                    "  DEX: {:>6}   CON: {:>6}   CHR: {:>6}",
                    format_stat(player.stats.dexterity),
                    format_stat(player.stats.constitution),
                    format_stat(player.stats.charisma)
                ),
                8 => "Combat Stats:".to_string(),
                9 => format!(
                    "  HP:   {}/{}       Mana: {}/{}",
                    player.hp, player.max_hp, player.mana, player.max_mana
                ),
                10 => format!(
                    "  AC:   {:<10} Gold: {} gp",
                    player.calculate_ac(),
                    player.gold
                ),
                11 => format!("  Level: {:<9} Exp:  {} exp", player.level, player.exp),
                12 => format!(
                    "  Weight: {}.{} lbs   Capacity: {}.{} lbs",
                    player.total_weight() / 10,
                    player.total_weight() % 10,
                    player.carrying_capacity() / 10,
                    player.carrying_capacity() % 10
                ),
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
                let owner = &crate::dungeon::STORE_OWNERS
                    [haggle.owner_index.min(crate::dungeon::MAX_OWNERS - 1)];
                let owner_title = owner.name.split('(').next().unwrap_or("Shopkeeper").trim();
                match row {
                    0 => format!("--- BARTERING (BUY): {} ---", haggle.item.name),
                    2 => format!("{}: \"{}\"", owner_title, haggle.comment),
                    3 => format!("Current Ask Price:  {} gp", haggle.current_asking),
                    5 => match haggle.last_bid {
                        Some(bid) => format!("Your last offer:    {} gp", bid),
                        None => "Your last offer:    None".to_string(),
                    },
                    6 => format!(
                        "Patience left:      {}",
                        owner.max_insults.saturating_sub(haggle.insults)
                    ),
                    8 => format!("Enter your offer:   {}", haggle.player_input),
                    10 => "--------------------------------------------".to_string(),
                    11 => "Type your bid (digits) and press ENTER to submit.".to_string(),
                    12 => {
                        "Press ENTER on blank input to accept the shopkeeper's price.".to_string()
                    }
                    13 => "Press ESC to cancel and exit.".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        }
        ScreenMode::BarterSellMenu => {
            if let Some(haggle) = active_haggle {
                let owner = &crate::dungeon::STORE_OWNERS
                    [haggle.owner_index.min(crate::dungeon::MAX_OWNERS - 1)];
                let owner_title = owner.name.split('(').next().unwrap_or("Shopkeeper").trim();
                match row {
                    0 => format!("--- BARTERING (SELL): {} ---", haggle.item.name),
                    2 => format!("{}: \"{}\"", owner_title, haggle.comment),
                    3 => format!("Current Offer:      {} gp", haggle.current_asking),
                    5 => match haggle.last_bid {
                        Some(ask) => format!("Your last ask:      {} gp", ask),
                        None => "Your last ask:      None".to_string(),
                    },
                    6 => format!(
                        "Patience left:      {}",
                        owner.max_insults.saturating_sub(haggle.insults)
                    ),
                    8 => format!("Enter your price:   {}", haggle.player_input),
                    10 => "--------------------------------------------".to_string(),
                    11 => "Type your price (digits) and press ENTER to submit.".to_string(),
                    12 => {
                        "Press ENTER on blank input to accept the shopkeeper's price.".to_string()
                    }
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
                return format!(
                    "{}. {} x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    item.count
                );
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
                return format!(
                    "{}. {} x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    item.count
                );
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
                return format!(
                    "{}. {} x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    item.count
                );
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
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c):"
                    .to_string();
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
                "  i/I : Inventory/Equipment h : Quaff potion        b : Browse book",
                "  W/t : Wear/Take off       r : Read scroll         m : Cast Mage spell",
                "  D   : Drop item           u : Use staff           p : Recite prayer",
                "  E   : Eat food            y : Aim wand            G : Gain spells",
                "  F   : Refill light        f/v : Fire/Throw item   n : Disarm trap",
                "  R   : Rest a while        B   : Bash door/foe     o/C : Open/Close door",
                "  #   : Toggle auto-search  T/g : Tunnel rock       j : Jam door",
                "  k   : Search 1 turn       l   : Look              { : Inscribe item",
                "  X   : Swap weapon         /   : Identify symbol   H : Character Sheet",
                "  < / > : Up/Down stairs    =   : Options Menu      V : Hall of Fame",
                "  Ctrl-P : Message History  Q   : Save and Quit     ? : Help Menu",
                "",
                "Press ESC or any key to return to the game.",
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
            let wands: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Wand { .. }))
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
                    return format!(
                        "{}. {} ({} charges)",
                        (b'a' + idx as u8) as char,
                        item.display_name(),
                        charges
                    );
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
                return "Choose target direction using movement keys (q/w/e/a/d/z/x/c):"
                    .to_string();
            }
            "".to_string()
        }
        ScreenMode::SelectJamDirection => {
            if row == 0 {
                return "--- JAM DOOR ---".to_string();
            }
            if row == 2 {
                return "Choose direction to jam using movement keys (q/w/e/a/d/z/x/c):"
                    .to_string();
            }
            "".to_string()
        }
        ScreenMode::UseStaffMenu => {
            if row == 0 {
                return "--- USE STAFF SELECTION ---".to_string();
            }
            let staves: Vec<(usize, &Item)> = player
                .inventory
                .iter()
                .enumerate()
                .filter(|(_, item)| matches!(item.item_type, ItemType::Staff { .. }))
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
                    return format!(
                        "{}. {} ({} charges)",
                        (b'a' + idx as u8) as char,
                        item.display_name(),
                        charges
                    );
                }
            }
            if staves.is_empty() && row == 1 {
                return "(No staves in inventory)".to_string();
            }
            "".to_string()
        }
        ScreenMode::Shop => {
            let owner_idx = if let Some(haggle) = active_haggle {
                haggle.owner_index.min(crate::dungeon::MAX_OWNERS - 1)
            } else {
                (shop as usize).min(crate::dungeon::MAX_OWNERS - 1)
            };
            let owner = &crate::dungeon::STORE_OWNERS[owner_idx];
            let items = get_shop_items(shop);
            let has_s_item = items.len() > 18;
            let last_letter = (b'a' + items.len().saturating_sub(1) as u8) as char;
            let prompt = if has_s_item {
                format!(
                    "Press item letter (a-{}) to buy/haggle, 'S' to Sell, ESC to exit.",
                    last_letter
                )
            } else {
                format!(
                    "Press item letter (a-{}) to buy/haggle, 's'/'S' to Sell, ESC to exit.",
                    last_letter
                )
            };

            if items.len() > 20 {
                if row == 0 {
                    return format!(
                        "--- {} (BUY) --- Gold: {} gp",
                        owner.name.trim(),
                        player.gold
                    );
                }
                if row >= 1 && row < items.len() + 1 && row < 23 {
                    let idx = row - 1;
                    let (name, base_price, _) = &items[idx];
                    let (_, ask_price) = crate::dungeon::calculate_buy_price(
                        *base_price,
                        owner,
                        player.race,
                        player.stats.charisma,
                    );
                    return format!(
                        "{}. {:<32}  -  {} gp",
                        (b'a' + idx as u8) as char,
                        name,
                        ask_price
                    );
                }
                if row == 23 {
                    return prompt;
                }
            } else {
                if row == 0 {
                    return format!("--- {} (BUY) ---", owner.name.trim());
                }
                if row == 1 {
                    return format!(
                        "Your Gold: {} gp    (Purse: {} gp)",
                        player.gold, owner.max_cost
                    );
                }
                if row >= 3 && row < items.len() + 3 && row < 23 {
                    let idx = row - 3;
                    let (name, base_price, _) = &items[idx];
                    let (_, ask_price) = crate::dungeon::calculate_buy_price(
                        *base_price,
                        owner,
                        player.race,
                        player.stats.charisma,
                    );
                    return format!(
                        "{}. {:<32}  -  {} gp",
                        (b'a' + idx as u8) as char,
                        name,
                        ask_price
                    );
                }
                if items.len() <= 18 {
                    if row == items.len() + 3 {
                        return "--------------------------------".to_string();
                    }
                    if row == items.len() + 4 {
                        return prompt;
                    }
                } else if row == 23 {
                    return prompt;
                }
            }
            "".to_string()
        }
        ScreenMode::ShopSellMenu => {
            let owner_idx = if let Some(haggle) = active_haggle {
                haggle.owner_index.min(crate::dungeon::MAX_OWNERS - 1)
            } else {
                (shop as usize).min(crate::dungeon::MAX_OWNERS - 1)
            };
            let owner = &crate::dungeon::STORE_OWNERS[owner_idx];
            if row == 0 {
                return format!("--- {} (SELL) ---", owner.name.trim());
            }
            if row == 1 {
                return format!(
                    "Your Gold: {} gp    (Max Offer: {} gp)",
                    player.gold, owner.max_cost
                );
            }
            let inv_len = player.inventory.len();
            if row > 1 && row <= inv_len + 1 && row < 22 {
                let idx = row - 2;
                let item = &player.inventory[idx];
                let base_val = crate::dungeon::get_item_base_value(item);
                let (_, offer) = crate::dungeon::calculate_sell_price(
                    base_val,
                    owner,
                    player.race,
                    player.stats.charisma,
                );
                return format!(
                    "{}. {} (offers ~{} gp) x{}",
                    (b'a' + idx as u8) as char,
                    item.display_name(),
                    offer,
                    item.count
                );
            }
            if player.inventory.is_empty() && row == 2 {
                return "(Your inventory is completely empty)".to_string();
            }
            if inv_len <= 18 {
                if row == inv_len + 3 {
                    return "---------------------------------".to_string();
                }
                if row == inv_len + 4 {
                    return "Select item letter to sell, or press ESC to return to Buy menu."
                        .to_string();
                }
            } else {
                if row == 22 {
                    return "---------------------------------".to_string();
                }
                if row == 23 {
                    return "Select item letter to sell, or press ESC to return to Buy menu."
                        .to_string();
                }
            }
            "".to_string()
        }
        ScreenMode::Dungeon => "".to_string(),
        ScreenMode::SelectRestCount => "".to_string(),
        ScreenMode::SelectBashDirection => "".to_string(),
        ScreenMode::SelectTunnelDirection => "".to_string(),
        ScreenMode::LocateMap => {
            if row == 22 {
                return "[Use movement keys to pan map, ESC to return]".to_string();
            }
            "".to_string()
        }
        ScreenMode::ReducedMap => get_reduced_map_row_text(row, level, player),
        ScreenMode::GameOver | ScreenMode::Tombstone => get_tombstone_row_text(row, player),
        ScreenMode::RoyalCoronation => get_coronation_row_text(row, player),
        ScreenMode::MessageHistory => {
            if row == 0 {
                return "--- MESSAGE HISTORY ---".to_string();
            }
            if row == 22 {
                return "[Press ESC to return]".to_string();
            }
            if (2..=21).contains(&row) {
                let start_idx = player.message_history.len().saturating_sub(20);
                let current_idx = start_idx + (row - 2);
                if current_idx < player.message_history.len() {
                    return player.message_history[current_idx].clone();
                }
            }
            "".to_string()
        }
        ScreenMode::IdentifySymbol => {
            if row == 0 {
                return "--- IDENTIFY A SYMBOL ---".to_string();
            }
            if row == 2 {
                return "Enter character to be identified:".to_string();
            }
            if let Some(ch) = player.identified_symbol {
                if row == 4 {
                    return format!("Symbol: '{}'", ch);
                }
                if row == 5 {
                    return format!("Description: {}", get_symbol_description(ch));
                }
                if row == 7 {
                    return "Matching monsters:".to_string();
                }
                let matching: Vec<&str> = crate::entity::monster_data::CREATURES_LIST
                    .iter()
                    .filter(|c| c.symbol == ch)
                    .map(|c| c.name)
                    .collect();
                if matching.is_empty() {
                    if row == 8 {
                        return "  (No matching monsters)".to_string();
                    }
                } else {
                    let m_idx = row.saturating_sub(8);
                    if m_idx < matching.len() && row < 21 {
                        return format!("  - {}", matching[m_idx]);
                    }
                }
            }
            if row == 22 {
                return "[Press any key to return]".to_string();
            }
            "".to_string()
        }
        ScreenMode::OptionsMenu => {
            if row == 0 {
                return "--- OPTIONS MENU ---".to_string();
            }
            if row == 2 {
                return format!(
                    "1) Keybinding Profile: {:?} (Press [1] to cycle)",
                    player.keybinding_mode
                );
            }
            if row == 3 {
                return format!(
                    "2) Auto-Pickup: {} (Press [2] to toggle)",
                    player.auto_pickup
                );
            }
            if row == 5 {
                return "Available profiles: Original (1-9), Roguelike (hjklyubn), ModernWasd (wasd)".to_string();
            }
            if row == 22 {
                return "[Press 1 to change profile, 2 to toggle pickup, ESC to return]"
                    .to_string();
            }
            "".to_string()
        }
        ScreenMode::HighScores => {
            let scores = crate::scores::load_high_scores(crate::scores::DEFAULT_SCORES_FILE);
            if row == 0 {
                return "Rank  Points Name              Sex Race       Class  Lvl Killed By"
                    .to_string();
            }
            if row == 1 {
                return "------------------------------------------------------------------"
                    .to_string();
            }
            let total_pages = scores.len().max(1).div_ceil(20);
            let current_page = player.high_score_page.min(total_pages.saturating_sub(1));
            let start_idx = current_page * 20;

            if (2..=21).contains(&row) {
                let entry_idx = start_idx + (row - 2);
                if entry_idx < scores.len() {
                    let entry = &scores[entry_idx];
                    let rank = entry_idx + 1;
                    let name_trunc: String = entry.name.chars().take(17).collect();
                    let race_trunc: String = entry.race.chars().take(10).collect();
                    let class_trunc: String = entry.character_class.chars().take(6).collect();
                    let killed_trunc: String = entry.died_from.chars().take(15).collect();
                    return format!(
                        "{:<4} {:>7} {:<17} {:^3} {:<10} {:<6} {:>3} {:<15}",
                        rank,
                        entry.points,
                        name_trunc,
                        entry.gender,
                        race_trunc,
                        class_trunc,
                        entry.level,
                        killed_trunc
                    );
                } else if scores.is_empty() && row == 3 {
                    return "                (No high scores recorded yet)".to_string();
                }
            }
            if row == 22 {
                return format!(
                    "[ Page {}/{} - Press Space/PgDn for next, ESC to return ]",
                    current_page + 1,
                    total_pages
                );
            }
            "".to_string()
        }
        ScreenMode::CombatLog => {
            if row == 0 {
                return "--- COMBAT LOG ---".to_string();
            }
            if row == 2 {
                return "Most recent messages:".to_string();
            }
            if row > 3 && row <= 3 + player.message_history.len().min(5) {
                let idx = player.message_history.len() - (row - 3);
                return player.message_history[idx].clone();
            }
            if row == 10 {
                return "Press ESC or any key to continue...".to_string();
            }
            "".to_string()
        }
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
        let screen = render_gui_screen(
            level,
            player,
            monsters,
            mode,
            shop,
            active_haggle,
            active_inscribe,
        );
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
            "ac": player.calculate_ac(),
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
                "hallucinating": player.flags.image > 0,
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

    let (center_x, center_y) = if mode == ScreenMode::LocateMap {
        let cx = (player.x as isize + player.locate_offset.0)
            .clamp(0, level.width.saturating_sub(1) as isize) as usize;
        let cy = (player.y as isize + player.locate_offset.1)
            .clamp(0, level.height.saturating_sub(1) as isize) as usize;
        (cx, cy)
    } else {
        (player.x, player.y)
    };
    let (view_top, view_bottom, view_left, view_right) =
        level.get_viewport_bounds(center_x, center_y);
    let view_h = view_bottom.saturating_sub(view_top);

    for screen_y in 0..view_h {
        let map_y = view_top + screen_y;
        let stats_part = get_lhs_stat_line(screen_y + 1, player, level);
        let mut map_part = String::new();

        if mode == ScreenMode::Dungeon
            || mode == ScreenMode::SelectRestCount
            || mode == ScreenMode::SelectBashDirection
            || mode == ScreenMode::SelectTunnelDirection
            || mode == ScreenMode::LocateMap
        {
            for map_x in view_left..view_right {
                if let Some(tile) = level.get_tile(map_x, map_y) {
                    if map_x == player.x && map_y == player.y {
                        map_part.push('@');
                    } else if tile.visible
                        && player.flags.blind == 0
                        && monsters.iter().any(|m| m.x == map_x && m.y == map_y)
                    {
                        let monster = monsters
                            .iter()
                            .find(|m| m.x == map_x && m.y == map_y)
                            .unwrap();
                        if player.flags.image > 0 {
                            let mut rng = rand::thread_rng();
                            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
                            map_part.push(
                                chars[rand::Rng::gen_range(&mut rng, 0..chars.len())] as char,
                            );
                        } else {
                            map_part.push(monster.symbol);
                        }
                    } else if (tile.visible || tile.remembered)
                        && level.items.iter().any(|i| i.x == map_x && i.y == map_y)
                        && !matches!(
                            tile.tile_type,
                            TileType::Wall
                                | TileType::SecretDoor
                                | TileType::MagmaVein { .. }
                                | TileType::QuartzVein { .. }
                                | TileType::Rubble
                                | TileType::StairsUp
                                | TileType::StairsDown
                        )
                    {
                        let floor_item = level
                            .items
                            .iter()
                            .find(|i| i.x == map_x && i.y == map_y)
                            .unwrap();
                        if player.flags.image > 0 {
                            let mut rng = rand::thread_rng();
                            let chars = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
                            map_part.push(
                                chars[rand::Rng::gen_range(&mut rng, 0..chars.len())] as char,
                            );
                        } else {
                            map_part.push(get_item_symbol(&floor_item.item));
                        }
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
            map_part = format!(
                "{:<66}",
                get_overlay_row_text(
                    screen_y,
                    mode,
                    player,
                    shop,
                    active_haggle,
                    active_inscribe,
                    level
                )
            );
        }

        let row_line = format!("{} {}\x1b[K", stats_part, map_part);
        let _ = execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(offset_x, offset_y + 1 + screen_y as u16)
        );
        print!("{}", row_line);
    }

    let status_bar = get_bottom_status_line(player, level.depth, false);
    let _ = execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(offset_x, offset_y + 23)
    );
    print!("{}\x1b[K", status_bar);

    io::stdout().flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Class, Gender, Player, Race};

    #[test]
    fn test_tombstone_rows_content() {
        let mut player =
            Player::new_with_gender("Gimli", Race::Dwarf, Class::Warrior, Gender::Male, 10, 10);
        player.level = 25;
        player.exp = 45000;
        player.gold = 12500;
        player.max_depth_reached = 30;
        player.died_from = "a Balrog".to_string();

        let row4 = get_tombstone_row_text(4, &player);
        assert!(
            row4.contains("Gimli"),
            "Row 4 should contain player name: {}",
            row4
        );

        let row8 = get_tombstone_row_text(8, &player);
        assert!(
            row8.contains("Warrior"),
            "Row 8 should contain class: {}",
            row8
        );

        let row9 = get_tombstone_row_text(9, &player);
        assert!(row9.contains("25"), "Row 9 should contain level: {}", row9);

        let row10 = get_tombstone_row_text(10, &player);
        assert!(
            row10.contains("45000 Exp"),
            "Row 10 should contain exp: {}",
            row10
        );

        let row11 = get_tombstone_row_text(11, &player);
        assert!(
            row11.contains("12500 Au"),
            "Row 11 should contain gold: {}",
            row11
        );

        let row14 = get_tombstone_row_text(14, &player);
        assert!(
            row14.contains("a Balrog"),
            "Row 14 should contain cause of death: {}",
            row14
        );
    }

    #[test]
    fn test_coronation_rows_content() {
        let mut king =
            Player::new_with_gender("Arthur", Race::Human, Class::Paladin, Gender::Male, 10, 10);
        king.balrog_killed = true;
        king.total_winner = true;
        let coronation_row = get_coronation_row_text(16, &king);
        assert!(
            coronation_row.contains("*King* Arthur!"),
            "Coronation row 16 should acclaim King: {}",
            coronation_row
        );

        let mut queen =
            Player::new_with_gender("Guinevere", Race::Elf, Class::Mage, Gender::Female, 10, 10);
        queen.balrog_killed = true;
        queen.total_winner = true;
        let queen_row = get_coronation_row_text(16, &queen);
        assert!(
            queen_row.contains("*Queen* Guinevere!"),
            "Coronation row 16 should acclaim Queen: {}",
            queen_row
        );
    }

    #[test]
    fn test_symbol_descriptions() {
        assert!(
            get_symbol_description('@')
                .to_lowercase()
                .contains("player")
        );
        assert!(
            get_symbol_description('$')
                .to_lowercase()
                .contains("treasure")
        );
        assert!(
            get_symbol_description('!')
                .to_lowercase()
                .contains("potion")
        );
        assert!(
            get_symbol_description('?')
                .to_lowercase()
                .contains("scroll")
        );
        assert!(get_symbol_description('#').to_lowercase().contains("wall"));
        assert!(get_symbol_description('.').to_lowercase().contains("floor"));
    }

    #[test]
    fn test_reduced_map_row_length() {
        let player = Player::new("Scout", Race::Elf, Class::Ranger, 10, 10);
        let level = crate::dungeon::DungeonLevel::new(66, 22, 1, 50);
        let row_text = get_reduced_map_row_text(5, &level, &player);
        assert_eq!(row_text.len(), 66);
    }
}
