use serde::{Serialize, Deserialize};
use crate::dice::Dice;
use rand::Rng;

pub mod creation;
pub use creation::run_character_creation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Human,
    HalfElf,
    Elf,
    Halfling,
    Gnome,
    Dwarf,
    HalfOrc,
    HalfTroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Warrior,
    Mage,
    Priest,
    Rogue,
    Ranger,
    Paladin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: i16,
    pub intelligence: i16,
    pub wisdom: i16,
    pub dexterity: i16,
    pub constitution: i16,
    pub charisma: i16,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            strength: 10,
            intelligence: 10,
            wisdom: 10,
            dexterity: 10,
            constitution: 10,
            charisma: 10,
        }
    }
}

impl Attributes {
    pub fn new(str_: i16, int_: i16, wis_: i16, dex_: i16, con_: i16, chr_: i16) -> Self {
        Self {
            strength: str_,
            intelligence: int_,
            wisdom: wis_,
            dexterity: dex_,
            constitution: con_,
            charisma: chr_,
        }
    }
}

/// Formats a stat value according to the canonical 18/xx percentile display:
/// - Values <= 18 display as right-aligned integer (e.g. `"    18"`).
/// - Values > 18 display as percentile (e.g. `" 18/01"`, `" 18/50"`, `"18/100"`).
pub fn format_stat(val: i16) -> String {
    if val <= 18 {
        format!("{:>6}", val)
    } else {
        let percentile = val - 18;
        if percentile >= 100 {
            "18/100".to_string()
        } else {
            format!(" 18/{:02}", percentile)
        }
    }
}

/// Adjusts a stat value by an integer modifier (such as from rings or amulets),
/// following canonical Umoria rules:
/// - Positive modifier: increases stat by 1 if < 18, or by 10 percentile points if >= 18 (capped at 118).
/// - Negative modifier: decreases stat by 10 percentile points if > 27, caps down to 18 if between 19 and 27,
///   or decreases by 1 if <= 18 (down to a minimum of 3).
pub fn modify_stat(current: i16, amount: i32) -> i16 {
    let mut new_stat = current;
    let loop_count = amount.unsigned_abs();
    for _ in 0..loop_count {
        if amount > 0 {
            if new_stat < 18 {
                new_stat += 1;
            } else if new_stat < 108 {
                new_stat += 10;
            } else {
                new_stat = 118;
            }
        } else if new_stat > 27 {
            new_stat -= 10;
        } else if new_stat > 18 {
            new_stat = 18;
        } else if new_stat > 3 {
            new_stat -= 1;
        }
    }
    new_stat
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PlayerFlags {
    pub rest: i16,
    pub blind: i16,
    pub paralysis: i16,
    pub confused: i16,
    pub afraid: i16,
    pub poisoned: i16,
    pub fast: i16,
    pub slow: i16,
    pub protect_evil: i16,
    pub invulnerability: i16,
    pub heroism: i16,
    pub super_heroism: i16,
    pub blessed: i16,
    pub heat_resistance: i16,
    pub cold_resistance: i16,
    pub detect_invisible: i16,
    pub word_of_recall: i16,
    pub see_infra: i16,
    pub timed_infra: i16,
    pub image: i16,

    pub see_invisible: bool,
    pub teleport: bool,
    pub free_action: bool,
    pub slow_digest: bool,
    pub aggravate: bool,
    pub resistant_to_fire: bool,
    pub resistant_to_cold: bool,
    pub resistant_to_acid: bool,
    pub regenerate_hp: bool,
    pub resistant_to_light: bool,
    pub free_fall: bool,
    pub sustain_str: bool,
    pub sustain_int: bool,
    pub sustain_wis: bool,
    pub sustain_con: bool,
    pub sustain_dex: bool,
    pub sustain_chr: bool,
    pub confuse_monster: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ConditionTickResult {
    pub messages: Vec<String>,
    pub died_from_poison: bool,
    pub recall_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    Weapon { damage: Dice },
    Armor { ac: i32 },
    Potion { heal_amount: i32 },
    Scroll { teleport: bool },
    Wand { charges: u32, spell_index: usize },
    Staff { charges: u32, prayer_index: usize },
    Food { nutrition: i32 },
    Light { fuel: i32 },
    Bow { multiplier: u32 },
    Missile { damage: Dice },
    Ring { bonus: i32 },
    Amulet { bonus: i32 },
    MagicBook { spell_flags: u32 },
    PrayerBook { spell_flags: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscribeState {
    pub item_index: usize,
    pub is_equipment: bool,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub count: u32,
    pub weight: u32,
    pub item_type: ItemType,
    #[serde(default)]
    pub inscription: Option<String>,
    #[serde(default = "default_identified")]
    pub identified: bool,
    #[serde(default)]
    pub flavor: Option<String>,
    #[serde(default)]
    pub equipped_slot: Option<String>,
    #[serde(default)]
    pub is_cursed: bool,
}

fn default_identified() -> bool {
    true
}

impl Item {
    pub fn new(name: &str, count: u32, weight: u32, item_type: ItemType) -> Self {
        Self {
            name: name.to_string(),
            count,
            weight,
            item_type,
            inscription: None,
            identified: true,
            flavor: None,
            equipped_slot: None,
            is_cursed: false,
        }
    }

    pub fn new_unidentified(name: &str, count: u32, weight: u32, item_type: ItemType) -> Self {
        Self {
            name: name.to_string(),
            count,
            weight,
            item_type,
            inscription: None,
            identified: false,
            flavor: None,
            equipped_slot: None,
            is_cursed: false,
        }
    }

    pub fn display_name(&self) -> String {
        let base = if !self.identified {
            if let Some(ref flv) = self.flavor {
                match &self.item_type {
                    ItemType::Potion { .. } => format!("{} Potion", flv),
                    ItemType::Scroll { .. } => format!("Scroll titled \"{}\"", flv),
                    ItemType::Wand { .. } => format!("{} Wand", flv),
                    ItemType::Staff { .. } => format!("{} Staff", flv),
                    ItemType::Ring { .. } => format!("{} Ring", flv),
                    ItemType::Amulet { .. } => format!("{} Amulet", flv),
                    _ => format!("{} {}", flv, self.name),
                }
            } else {
                self.name.clone()
            }
        } else {
            match &self.item_type {
                ItemType::Ring { bonus } if *bonus != 0 && !self.name.contains('(') => {
                    format!("{} ({:+})", self.name, bonus)
                }
                ItemType::Amulet { bonus } if *bonus != 0 && !self.name.contains('(') => {
                    format!("{} ({:+})", self.name, bonus)
                }
                _ => self.name.clone(),
            }
        };

        if let Some(ref ins) = self.inscription {
            format!("{} {{{}}}", base, ins)
        } else {
            base
        }
    }

    pub fn get_equipment_slot(&self) -> &str {
        if let Some(ref slot) = self.equipped_slot {
            return slot.as_str();
        }
        match &self.item_type {
            ItemType::Weapon { .. } => "Weapon",
            ItemType::Bow { .. } => "Ranged Weapon",
            ItemType::Light { .. } => "Light Source",
            ItemType::Ring { .. } => "On right hand",
            ItemType::Amulet { .. } => "Around neck",
            ItemType::Armor { .. } => {
                let name = self.name.to_lowercase();
                if name.contains("shield") {
                    "Shield"
                } else if name.contains("helm") || name.contains("hat") || name.contains("crown") {
                    "Headwear"
                } else if name.contains("gloves") || name.contains("gauntlets") {
                    "Gloves"
                } else if name.contains("boots") || name.contains("shoes") {
                    "Boots"
                } else if name.contains("cloak") {
                    "Cloak"
                } else {
                    "Body Armor"
                }
            }
            _ => "Accessory",
        }
    }
}

pub const BASE_EXP_LEVELS: &[u32] = &[
    10,   25,   45,    70,    100,   140,   200,    280,    380,    500,
    650,     850,     1100,    1400,    1800,    2300,    2900,     3600,     4400,     5400,
    6800, 8400, 10200, 12500, 17500, 25000, 35000, 50000, 75000, 100000,
    150000, 200000, 300000, 400000, 500000, 750000, 1500000, 2500000, 5000000, 10000000
];

fn default_food() -> i32 {
    7500
}

fn default_body_weight() -> u32 {
    150
}

fn default_town_threshold() -> u32 {
    use rand::Rng;
    rand::thread_rng().gen_range(1..=5)
}

fn default_spells_learned_order() -> [u8; 32] {
    [99; 32]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub level: u32,
    pub exp: u32,
    pub gold: u32,
    
    pub x: usize,
    pub y: usize,
    
    pub max_hp: i32,
    pub hp: i32,
    pub max_mana: i32,
    pub mana: i32,
    #[serde(default = "default_food")]
    pub food: i32,
    
    pub stats: Attributes,
    #[serde(default)]
    pub base_stats: Attributes,
    #[serde(default)]
    pub max_stats: Attributes,
    #[serde(default = "default_body_weight")]
    pub body_weight: u32,
    #[serde(default)]
    pub flags: PlayerFlags,
    #[serde(default)]
    pub max_depth_reached: u32,
    #[serde(default)]
    pub flavors: crate::flavor::FlavorRegistry,
    pub inventory: Vec<Item>,
    pub equipment: Vec<Item>,
    pub balrog_killed: bool,
    #[serde(default)]
    pub searching: bool,
    #[serde(default)]
    pub is_wizard: bool,
    #[serde(default)]
    pub killed_town_npcs: u32,
    #[serde(default = "default_town_threshold")]
    pub town_npc_attack_threshold: u32,
    #[serde(default)]
    pub spells_learnt: u32,
    #[serde(default)]
    pub spells_worked: u32,
    #[serde(default)]
    pub spells_forgotten: u32,
    #[serde(default = "default_spells_learned_order")]
    pub spells_learned_order: [u8; 32],
    #[serde(default)]
    pub new_spells_to_learn: u32,
    
    pub base_hp_levels: Vec<i32>,
    pub exp_factor: u32,
    pub history: String,
}

pub fn generate_history(race: Race) -> String {
    let mut rng = rand::thread_rng();
    let mut history = String::new();
    let mut chart = match race {
        Race::Human => 1,
        Race::HalfElf => 4,
        Race::Elf => 7,
        Race::Halfling => 10,
        Race::Gnome => 13,
        Race::Dwarf => 16,
        Race::HalfOrc => 19,
        Race::HalfTroll => 22,
    };

    loop {
        let roll = rng.gen_range(1..=100);
        let (text, next_chart) = match chart {
            1 => {
                if roll <= 10 { ("You are the illegitimate and unacknowledged child ", 2) }
                else if roll <= 20 { ("You are the illegitimate but acknowledged child ", 2) }
                else if roll <= 95 { ("You are one of several children ", 2) }
                else { ("You are the first child ", 2) }
            }
            2 => {
                if roll <= 40 { ("of a Serf. ", 3) }
                else if roll <= 65 { ("of a Yeoman. ", 3) }
                else if roll <= 80 { ("of a Townsman. ", 3) }
                else if roll <= 90 { ("of a Guildsman. ", 3) }
                else if roll <= 96 { ("of a Landed Knight. ", 3) }
                else if roll <= 99 { ("of a Titled Noble. ", 3) }
                else { ("of a Royal Blood Line. ", 3) }
            }
            3 => {
                if roll <= 20 { ("You are the black sheep of the family. ", 50) }
                else if roll <= 80 { ("You are a credit to the family. ", 50) }
                else { ("You are a well liked child. ", 50) }
            }
            4 => {
                if roll <= 40 { ("Your mother was a Green-Elf. ", 1) }
                else if roll <= 75 { ("Your father was a Green-Elf. ", 1) }
                else if roll <= 90 { ("Your mother was a Grey-Elf. ", 1) }
                else if roll <= 95 { ("Your father was a Grey-Elf. ", 1) }
                else if roll <= 98 { ("Your mother was a High-Elf. ", 1) }
                else { ("Your father was a High-Elf. ", 1) }
            }
            7 => {
                if roll <= 60 { ("You are one of several children ", 8) }
                else { ("You are the only child ", 8) }
            }
            8 => {
                if roll <= 75 { ("of a Green-Elf ", 9) }
                else if roll <= 95 { ("of a Grey-Elf ", 9) }
                else { ("of a High-Elf ", 9) }
            }
            9 => {
                if roll <= 40 { ("Ranger. ", 50) }
                else if roll <= 70 { ("Archer. ", 50) }
                else if roll <= 87 { ("Warrior. ", 50) }
                else if roll <= 95 { ("Mage. ", 50) }
                else if roll <= 99 { ("Prince. ", 50) }
                else { ("King. ", 50) }
            }
            10 => {
                if roll <= 85 { ("You are one of several children of a Halfling ", 11) }
                else { ("You are the only child of a Halfling ", 11) }
            }
            11 => {
                if roll <= 20 { ("Bum. ", 3) }
                else if roll <= 30 { ("Tavern Owner. ", 3) }
                else if roll <= 40 { ("Miller. ", 3) }
                else if roll <= 50 { ("Home Owner. ", 3) }
                else if roll <= 80 { ("Burglar. ", 3) }
                else if roll <= 95 { ("Warrior. ", 3) }
                else if roll <= 99 { ("Mage. ", 3) }
                else { ("Clan Elder. ", 3) }
            }
            13 => {
                if roll <= 85 { ("You are one of several children of a Gnome ", 14) }
                else { ("You are the only child of a Gnome ", 14) }
            }
            14 => {
                if roll <= 20 { ("Beggar. ", 3) }
                else if roll <= 50 { ("Braggart. ", 3) }
                else if roll <= 75 { ("Prankster. ", 3) }
                else if roll <= 95 { ("Warrior. ", 3) }
                else { ("Mage. ", 3) }
            }
            16 => {
                if roll <= 25 { ("You are one of two children of a Dwarven ", 17) }
                else { ("You are the only child of a Dwarven ", 17) }
            }
            17 => {
                if roll <= 10 { ("Thief. ", 18) }
                else if roll <= 25 { ("Prison Guard. ", 18) }
                else if roll <= 75 { ("Miner. ", 18) }
                else if roll <= 90 { ("Warrior. ", 18) }
                else if roll <= 99 { ("Priest. ", 18) }
                else { ("King. ", 18) }
            }
            18 => {
                if roll <= 15 { ("You are the black sheep of the family. ", 50) }
                else if roll <= 85 { ("You are a credit to the family. ", 50) }
                else { ("You are a well liked child. ", 50) }
            }
            19 => {
                if roll <= 25 { ("Your mother was an Orc, but it is unacknowledged. ", 20) }
                else { ("Your father was an Orc, but it is unacknowledged. ", 20) }
            }
            20 => {
                ("You are the adopted child ", 2)
            }
            22 => {
                if roll <= 30 { ("Your mother was a Cave-Troll ", 23) }
                else if roll <= 60 { ("Your father was a Cave-Troll ", 23) }
                else if roll <= 75 { ("Your mother was a Hill-Troll ", 23) }
                else if roll <= 90 { ("Your father was a Hill-Troll ", 23) }
                else if roll <= 95 { ("Your mother was a Water-Troll ", 23) }
                else { ("Your father was a Water-Troll ", 23) }
            }
            23 => {
                if roll <= 5 { ("Cook. ", 50) }
                else if roll <= 95 { ("Warrior. ", 50) }
                else if roll <= 99 { ("Shaman. ", 50) }
                else { ("Clan Chief. ", 50) }
            }
            50 => {
                if roll <= 20 { ("You have dark brown eyes, ", 51) }
                else if roll <= 60 { ("You have brown eyes, ", 51) }
                else if roll <= 70 { ("You have hazel eyes, ", 51) }
                else if roll <= 80 { ("You have green eyes, ", 51) }
                else if roll <= 90 { ("You have blue eyes, ", 51) }
                else { ("You have blue-gray eyes, ", 51) }
            }
            51 => {
                if roll <= 70 { ("straight ", 52) }
                else if roll <= 90 { ("wavy ", 52) }
                else { ("curly ", 52) }
            }
            52 => {
                if roll <= 30 { ("black hair, ", 53) }
                else if roll <= 70 { ("brown hair, ", 53) }
                else if roll <= 80 { ("auburn hair, ", 53) }
                else if roll <= 90 { ("red hair, ", 53) }
                else { ("blond hair, ", 53) }
            }
            53 => {
                if roll <= 10 { ("and a very dark complexion.", 0) }
                else if roll <= 30 { ("and a dark complexion.", 0) }
                else if roll <= 80 { ("and an average complexion.", 0) }
                else if roll <= 90 { ("and a fair complexion.", 0) }
                else { ("and a very fair complexion.", 0) }
            }
            _ => ("", 0),
        };

        history.push_str(text);
        if next_chart == 0 {
            break;
        }
        chart = next_chart;
    }

    history
}

impl Player {
    pub fn new(name: &str, race: Race, class: Class, x: usize, y: usize) -> Self {
        let base_stats = Attributes::new(10, 10, 10, 10, 10, 10);
        
        let race_hp = match race {
            Race::Human => 10,
            Race::HalfElf => 9,
            Race::Elf => 8,
            Race::Halfling => 6,
            Race::Gnome => 7,
            Race::Dwarf => 9,
            Race::HalfOrc => 10,
            Race::HalfTroll => 12,
        };
        let class_hp = match class {
            Class::Warrior => 9,
            Class::Mage => 0,
            Class::Priest => 2,
            Class::Rogue => 6,
            Class::Ranger => 4,
            Class::Paladin => 6,
        };
        let hit_die = race_hp + class_hp;

        let mut base_hp_levels = vec![0; 40];
        base_hp_levels[0] = hit_die;
        let mut temp_rng = rand::thread_rng();
        for i in 1..40 {
            base_hp_levels[i] = base_hp_levels[i - 1] + temp_rng.gen_range(1..=hit_die);
        }

        let race_exp = match race {
            Race::Human => 100,
            Race::HalfElf => 110,
            Race::Elf => 120,
            Race::Halfling => 110,
            Race::Gnome => 125,
            Race::Dwarf => 120,
            Race::HalfOrc => 110,
            Race::HalfTroll => 120,
        };
        let class_exp = match class {
            Class::Warrior => 0,
            Class::Mage => 30,
            Class::Priest => 20,
            Class::Rogue => 0,
            Class::Ranger => 40,
            Class::Paladin => 35,
        };
        let exp_factor = race_exp + class_exp;
        let history = generate_history(race);
        let body_weight = match race {
            Race::Human => 180,
            Race::HalfElf => 130,
            Race::Elf => 100,
            Race::Halfling => 60,
            Race::Gnome => 90,
            Race::Dwarf => 150,
            Race::HalfOrc => 150,
            Race::HalfTroll => 220,
        };

        let mut player = Self {
            name: name.to_string(),
            race,
            class,
            level: 1,
            exp: 0,
            gold: 150,
            x,
            y,
            max_hp: hit_die,
            hp: hit_die,
            max_mana: 0,
            mana: 0,
            food: 7500,
            stats: base_stats.clone(),
            base_stats: base_stats.clone(),
            max_stats: base_stats,
            body_weight,
            flags: PlayerFlags::default(),
            max_depth_reached: 0,
            flavors: crate::flavor::FlavorRegistry::new(&mut temp_rng),
            inventory: {
                let mut starter = vec![
                    Item::new("Dagger", 1, 10, ItemType::Weapon { damage: Dice::new(1, 4) }),
                    Item::new("Leather Armor", 1, 80, ItemType::Armor { ac: 4 }),
                    Item::new("Wooden Torch", 1, 15, ItemType::Light { fuel: 4000 }),
                    Item::new("Potion of Cure Light Wounds", 2, 5, ItemType::Potion { heal_amount: 10 }),
                    Item::new("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true }),
                    Item::new("Iron Spike", 5, 2, ItemType::Scroll { teleport: false }),
                    Item::new("Ration of Food", 2, 10, ItemType::Food { nutrition: 5000 }),
                ];
                if matches!(class, Class::Mage | Class::Rogue | Class::Ranger) {
                    starter.push(crate::magic::create_book(0, false));
                } else if matches!(class, Class::Priest | Class::Paladin) {
                    starter.push(crate::magic::create_book(0, true));
                }
                if matches!(class, Class::Ranger) {
                    starter.push(Item::new("Short Bow", 1, 30, ItemType::Bow { multiplier: 2 }));
                    starter.push(Item::new("Arrow", 25, 2, ItemType::Missile { damage: Dice::new(1, 4) }));
                }
                starter
            },
            equipment: Vec::new(),
            balrog_killed: false,
            searching: false,
            is_wizard: false,
            killed_town_npcs: 0,
            town_npc_attack_threshold: rand::thread_rng().gen_range(1..=5),
            spells_learnt: 0,
            spells_worked: 0,
            spells_forgotten: 0,
            spells_learned_order: [99; 32],
            new_spells_to_learn: 0,
            base_hp_levels,
            exp_factor,
            history,
        };

        player.flavors.identify("Potion of Cure Light Wounds");
        player.flavors.identify("Scroll of Phase Door");
        
        player.apply_race_and_class_modifiers();
        player.base_stats = player.stats.clone();
        player.max_stats = player.stats.clone();
        player.update_equipment_bonuses();
        player.update_max_hp_and_mana();
        player.update_spells_to_learn();
        player.hp = player.max_hp;
        player.mana = player.max_mana;
        player
    }

    pub fn identify_item_kind(&mut self, item_name: &str) -> bool {
        let newly_identified = self.flavors.identify(item_name);
        for item in self.inventory.iter_mut() {
            if item.name == item_name {
                item.identified = true;
            }
        }
        for item in self.equipment.iter_mut() {
            if item.name == item_name {
                item.identified = true;
            }
        }
        newly_identified
    }

    pub fn add_item_to_inventory(&mut self, item: Item) {
        if let Some(existing) = self.inventory.iter_mut().find(|i| i.name == item.name && i.item_type == item.item_type) {
            existing.count += item.count;
        } else {
            self.inventory.push(item);
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn get_light_radius(&self) -> usize {
        if let Some(light_item) = self.equipment.iter().find(|i| matches!(i.item_type, ItemType::Light { .. })) {
            match &light_item.item_type {
                ItemType::Light { fuel }
                    if *fuel > 0 => {
                        if light_item.name.contains("Lantern") {
                            3
                        } else {
                            2
                        }
                    }
                _ => 1,
            }
        } else {
            1
        }
    }

    pub fn tick_digestion_and_regen(&mut self, is_resting: bool) -> (Option<&'static str>, bool) {
        // Natural HP/Mana regeneration when active (not resting, since resting has accelerated 1 HP/turn)
        // Note: Poison prevents natural HP regeneration in Moria
        let can_regen_hp = self.flags.poisoned < 1 && self.hp < self.max_hp;
        let hp_interval = if self.flags.regenerate_hp { 10 } else { 20 };

        if !is_resting && self.food > 0 {
            if can_regen_hp && self.food % hp_interval == 0 {
                self.hp = (self.hp + 1).min(self.max_hp);
            }
            if self.food % 15 == 0 {
                self.mana = (self.mana + 1).min(self.max_mana);
            }
        }

        // Food consumption: 1 unit per turn, or 1 every 2 turns if slow_digest
        let consumes_food = if self.flags.slow_digest {
            self.food % 2 == 0
        } else {
            true
        };

        if consumes_food {
            self.food -= 1;
        }

        // Canonical Umoria: accelerated food burn when sped up (speed < 0)
        let speed_mod = self.speed_modifier();
        if speed_mod < 0 {
            self.food -= (speed_mod as i32) * (speed_mod as i32);
        }

        if self.food < 0 {
            // Starvation: damage ticks every 15 turns instead of every single turn
            let starvation_tick = self.food.abs() % 15 == 0;
            if starvation_tick {
                let dmg = if self.food < -500 { 2 } else { 1 };
                self.hp -= dmg;
                if self.hp <= 0 {
                    self.hp = 0;
                    return (Some(" You have starved to death!"), true);
                }
                return (Some(" You are starving!"), false);
            } else if self.food == -1 {
                return (Some(" You are starving!"), false);
            }
        } else if self.food < 300 {
            if self.food == 299 {
                return (Some(" You are getting faint from hunger."), false);
            } else if rand::thread_rng().gen_bool(0.02) {
                return (Some(" You faint from the lack of food!"), false);
            }
        } else if self.food == 999 {
            return (Some(" You are getting weak from hunger."), false);
        } else if self.food == 1999 {
            return (Some(" You are getting hungry."), false);
        }

        (None, false)
    }

    pub fn get_con_hp_bonus(&self) -> i32 {
        let con = self.stats.constitution;
        if con < 7 {
            con as i32 - 7
        } else if con < 17 {
            0
        } else if con == 17 {
            1
        } else if con < 94 {
            2
        } else if con < 117 {
            3
        } else {
            4
        }
    }

    pub fn is_stat_sustained(&self, stat_idx: usize) -> bool {
        match stat_idx {
            0 => self.flags.sustain_str,
            1 => self.flags.sustain_int,
            2 => self.flags.sustain_wis,
            3 => self.flags.sustain_dex,
            4 => self.flags.sustain_con,
            5 => self.flags.sustain_chr,
            _ => false,
        }
    }

    /// Recalculates all intrinsic flags and stat bonuses granted by equipped items.
    pub fn update_equipment_bonuses(&mut self) {
        if self.base_stats.strength == 0 && self.base_stats.intelligence == 0 {
            self.base_stats = self.stats.clone();
        }

        // Reset intrinsic flags provided by equipment
        self.flags.see_invisible = self.flags.detect_invisible > 0;
        self.flags.teleport = false;
        self.flags.free_action = false;
        self.flags.slow_digest = false;
        self.flags.aggravate = false;
        self.flags.resistant_to_fire = false;
        self.flags.resistant_to_cold = false;
        self.flags.resistant_to_acid = false;
        self.flags.free_fall = false;
        self.flags.sustain_str = false;
        self.flags.sustain_int = false;
        self.flags.sustain_wis = false;
        self.flags.sustain_con = false;
        self.flags.sustain_dex = false;
        self.flags.sustain_chr = false;

        let mut str_mod = 0i32;
        let mut int_mod = 0i32;
        let mut wis_mod = 0i32;
        let mut dex_mod = 0i32;
        let mut con_mod = 0i32;
        let mut chr_mod = 0i32;

        for item in &self.equipment {
            let name_lower = item.name.to_lowercase();
            match &item.item_type {
                ItemType::Ring { bonus } => {
                    if name_lower.contains("strength") && !name_lower.contains("sustain") {
                        str_mod += bonus;
                    } else if name_lower.contains("dexterity") && !name_lower.contains("sustain") {
                        dex_mod += bonus;
                    } else if name_lower.contains("constitution") && !name_lower.contains("sustain") {
                        con_mod += bonus;
                    } else if name_lower.contains("intelligence") && !name_lower.contains("sustain") {
                        int_mod += bonus;
                    } else if name_lower.contains("weakness") {
                        str_mod -= bonus.abs().max(1);
                    } else if name_lower.contains("stupidity") {
                        int_mod -= bonus.abs().max(1);
                    } else if name_lower.contains("woe") {
                        str_mod -= 5;
                        int_mod -= 5;
                        wis_mod -= 5;
                    }

                    if name_lower.contains("see invisible") {
                        self.flags.see_invisible = true;
                    }
                    if name_lower.contains("free action") {
                        self.flags.free_action = true;
                    }
                    if name_lower.contains("slow digestion") {
                        self.flags.slow_digest = true;
                    }
                    if name_lower.contains("feather falling") {
                        self.flags.free_fall = true;
                    }
                    if name_lower.contains("resist fire") || name_lower.contains("lordly protection (fire)") {
                        self.flags.resistant_to_fire = true;
                    }
                    if name_lower.contains("resist cold") || name_lower.contains("lordly protection (cold)") {
                        self.flags.resistant_to_cold = true;
                    }
                    if name_lower.contains("lordly protection (acid)") {
                        self.flags.resistant_to_acid = true;
                    }
                    if name_lower.contains("teleportation") {
                        self.flags.teleport = true;
                    }
                    if name_lower.contains("aggravate") {
                        self.flags.aggravate = true;
                    }
                    if name_lower.contains("sustain strength") {
                        self.flags.sustain_str = true;
                    }
                    if name_lower.contains("sustain intelligence") {
                        self.flags.sustain_int = true;
                    }
                    if name_lower.contains("sustain wisdom") {
                        self.flags.sustain_wis = true;
                    }
                    if name_lower.contains("sustain constitution") {
                        self.flags.sustain_con = true;
                    }
                    if name_lower.contains("sustain dexterity") {
                        self.flags.sustain_dex = true;
                    }
                    if name_lower.contains("sustain charisma") {
                        self.flags.sustain_chr = true;
                    }
                }
                ItemType::Amulet { bonus } => {
                    if name_lower.contains("wisdom") {
                        wis_mod += bonus;
                    } else if name_lower.contains("charisma") {
                        chr_mod += bonus;
                    } else if name_lower.contains("the magi") {
                        int_mod += bonus;
                        wis_mod += bonus;
                        chr_mod += bonus;
                        self.flags.free_action = true;
                        self.flags.see_invisible = true;
                    } else if name_lower.contains("doom") {
                        str_mod -= 5;
                        int_mod -= 5;
                        wis_mod -= 5;
                        dex_mod -= 5;
                        con_mod -= 5;
                        chr_mod -= 5;
                    }

                    if name_lower.contains("resist acid") {
                        self.flags.resistant_to_acid = true;
                    }
                    if name_lower.contains("slow digestion") {
                        self.flags.slow_digest = true;
                    }
                    if name_lower.contains("teleportation") {
                        self.flags.teleport = true;
                    }
                }
                _ => {
                    if name_lower.contains("see invisible") {
                        self.flags.see_invisible = true;
                    }
                    if name_lower.contains("free action") {
                        self.flags.free_action = true;
                    }
                }
            }
        }

        self.stats.strength = modify_stat(self.base_stats.strength, str_mod);
        self.stats.intelligence = modify_stat(self.base_stats.intelligence, int_mod);
        self.stats.wisdom = modify_stat(self.base_stats.wisdom, wis_mod);
        self.stats.dexterity = modify_stat(self.base_stats.dexterity, dex_mod);
        self.stats.constitution = modify_stat(self.base_stats.constitution, con_mod);
        self.stats.charisma = modify_stat(self.base_stats.charisma, chr_mod);

        self.update_max_hp_and_mana();
    }

    /// Drains a stat by 1 or percentile chunk according to canonical Umoria rules.
    /// If sustained, does nothing and returns false.
    pub fn drain_stat<R: rand::Rng>(&mut self, stat_idx: usize, rng: &mut R) -> bool {
        if self.is_stat_sustained(stat_idx) {
            return false;
        }

        if self.equipment.is_empty() || (self.base_stats.strength == 0 && self.base_stats.intelligence == 0) {
            self.base_stats = self.stats.clone();
        }

        let current = match stat_idx {
            0 => self.base_stats.strength,
            1 => self.base_stats.intelligence,
            2 => self.base_stats.wisdom,
            3 => self.base_stats.dexterity,
            4 => self.base_stats.constitution,
            5 => self.base_stats.charisma,
            _ => return false,
        };

        if current <= 3 {
            return false;
        }

        let new_stat = if (19..117).contains(&current) {
            let loss = (((118 - current as i32) >> 1) + 1) >> 1;
            let roll = rng.gen_range(1..=loss);
            (current as i32 - roll - loss).max(18) as i16
        } else {
            current - 1
        };

        match stat_idx {
            0 => self.base_stats.strength = new_stat,
            1 => self.base_stats.intelligence = new_stat,
            2 => self.base_stats.wisdom = new_stat,
            3 => self.base_stats.dexterity = new_stat,
            4 => self.base_stats.constitution = new_stat,
            5 => self.base_stats.charisma = new_stat,
            _ => {}
        }

        self.update_equipment_bonuses();
        true
    }

    /// Restores a single stat back to its natural maximum.
    pub fn restore_stat(&mut self, stat_idx: usize) -> bool {
        if self.equipment.is_empty() || (self.base_stats.strength == 0 && self.base_stats.intelligence == 0) {
            self.base_stats = self.stats.clone();
        }

        let (current, max) = match stat_idx {
            0 => (self.base_stats.strength, self.max_stats.strength),
            1 => (self.base_stats.intelligence, self.max_stats.intelligence),
            2 => (self.base_stats.wisdom, self.max_stats.wisdom),
            3 => (self.base_stats.dexterity, self.max_stats.dexterity),
            4 => (self.base_stats.constitution, self.max_stats.constitution),
            5 => (self.base_stats.charisma, self.max_stats.charisma),
            _ => return false,
        };

        if current >= max {
            return false;
        }

        match stat_idx {
            0 => self.base_stats.strength = max,
            1 => self.base_stats.intelligence = max,
            2 => self.base_stats.wisdom = max,
            3 => self.base_stats.dexterity = max,
            4 => self.base_stats.constitution = max,
            5 => self.base_stats.charisma = max,
            _ => {}
        }

        self.update_equipment_bonuses();
        true
    }

    /// Restores all 6 stats to natural maximums.
    pub fn restore_all_stats(&mut self) -> bool {
        let mut any = false;
        for i in 0..6 {
            if self.restore_stat(i) {
                any = true;
            }
        }
        any
    }

    /// Increases a stat (e.g. via stat gain potion) up to 118 (18/100).
    pub fn increase_stat<R: rand::Rng>(&mut self, stat_idx: usize, rng: &mut R) -> bool {
        if self.equipment.is_empty() || (self.base_stats.strength == 0 && self.base_stats.intelligence == 0) {
            self.base_stats = self.stats.clone();
        }

        let current = match stat_idx {
            0 => self.base_stats.strength,
            1 => self.base_stats.intelligence,
            2 => self.base_stats.wisdom,
            3 => self.base_stats.dexterity,
            4 => self.base_stats.constitution,
            5 => self.base_stats.charisma,
            _ => return false,
        };

        if current >= 118 {
            return false;
        }

        let new_stat = if (18..116).contains(&current) {
            let gain = ((118 - current as i32) / 3 + 1) >> 1;
            let roll = rng.gen_range(1..=gain);
            (current as i32 + roll + gain).min(118) as i16
        } else {
            current + 1
        };

        match stat_idx {
            0 => {
                self.base_stats.strength = new_stat;
                if new_stat > self.max_stats.strength { self.max_stats.strength = new_stat; }
            }
            1 => {
                self.base_stats.intelligence = new_stat;
                if new_stat > self.max_stats.intelligence { self.max_stats.intelligence = new_stat; }
            }
            2 => {
                self.base_stats.wisdom = new_stat;
                if new_stat > self.max_stats.wisdom { self.max_stats.wisdom = new_stat; }
            }
            3 => {
                self.base_stats.dexterity = new_stat;
                if new_stat > self.max_stats.dexterity { self.max_stats.dexterity = new_stat; }
            }
            4 => {
                self.base_stats.constitution = new_stat;
                if new_stat > self.max_stats.constitution { self.max_stats.constitution = new_stat; }
            }
            5 => {
                self.base_stats.charisma = new_stat;
                if new_stat > self.max_stats.charisma { self.max_stats.charisma = new_stat; }
            }
            _ => {}
        }

        self.update_equipment_bonuses();
        true
    }

    /// Ticks active conditions, counters, and status effects for one game turn.
    pub fn tick_conditions(&mut self, turn: u64) -> ConditionTickResult {
        let mut result = ConditionTickResult::default();

        // 1. Blindness
        if self.flags.blind > 0 {
            self.flags.blind -= 1;
            if self.flags.blind == 0 {
                result.messages.push("The veil of darkness lifts.".to_string());
            }
        }

        // 2. Confusion
        if self.flags.confused > 0 {
            self.flags.confused -= 1;
            if self.flags.confused == 0 {
                result.messages.push("You feel less confused now.".to_string());
            }
        }

        // 3. Fear
        if self.flags.afraid > 0 {
            if self.flags.heroism > 0 || self.flags.super_heroism > 0 {
                self.flags.afraid = 0;
                result.messages.push("You feel bolder now.".to_string());
            } else {
                self.flags.afraid -= 1;
                if self.flags.afraid == 0 {
                    result.messages.push("You feel bolder now.".to_string());
                }
            }
        }

        // 4. Poison
        if self.flags.poisoned > 0 {
            self.flags.poisoned -= 1;
            if self.flags.poisoned == 0 {
                result.messages.push("You feel better.".to_string());
            } else {
                let con_adj = self.get_con_hp_bonus();
                let damage = match con_adj {
                    a if a <= -4 => 4,
                    -3 | -2 => 3,
                    -1 => 2,
                    0 => 1,
                    1..=3 => if turn.is_multiple_of(2) { 1 } else { 0 },
                    4..=5 => if turn.is_multiple_of(3) { 1 } else { 0 },
                    _ => if turn.is_multiple_of(4) { 1 } else { 0 },
                };
                if damage > 0 {
                    self.hp -= damage;
                    if self.hp <= 0 {
                        self.hp = 0;
                        result.died_from_poison = true;
                    }
                }
            }
        }

        // 5. Speed (Fast / Slow)
        if self.flags.fast > 0 {
            self.flags.fast -= 1;
            if self.flags.fast == 0 {
                result.messages.push("You feel yourself slow down.".to_string());
            }
        }
        if self.flags.slow > 0 {
            self.flags.slow -= 1;
            if self.flags.slow == 0 {
                result.messages.push("You feel yourself speed up.".to_string());
            }
        }

        // 6. Evil protection
        if self.flags.protect_evil > 0 {
            self.flags.protect_evil -= 1;
            if self.flags.protect_evil == 0 {
                result.messages.push("You no longer feel safe from evil.".to_string());
            }
        }

        // 7. Invulnerability
        if self.flags.invulnerability > 0 {
            self.flags.invulnerability -= 1;
            if self.flags.invulnerability == 0 {
                result.messages.push("Your skin returns to normal.".to_string());
            }
        }

        // 8. Heroism
        if self.flags.heroism > 0 {
            self.flags.heroism -= 1;
            if self.flags.heroism == 0 {
                self.max_hp = (self.max_hp - 10).max(1);
                self.hp = self.hp.min(self.max_hp);
                result.messages.push("The heroism wears off.".to_string());
            }
        }

        // 9. Super Heroism
        if self.flags.super_heroism > 0 {
            self.flags.super_heroism -= 1;
            if self.flags.super_heroism == 0 {
                self.max_hp = (self.max_hp - 20).max(1);
                self.hp = self.hp.min(self.max_hp);
                result.messages.push("The super heroism wears off.".to_string());
            }
        }

        // 10. Blessed
        if self.flags.blessed > 0 {
            self.flags.blessed -= 1;
            if self.flags.blessed == 0 {
                result.messages.push("The prayer has expired.".to_string());
            }
        }

        // 11. Heat & Cold Resistance
        if self.flags.heat_resistance > 0 {
            self.flags.heat_resistance -= 1;
            if self.flags.heat_resistance == 0 {
                result.messages.push("You no longer feel safe from flame.".to_string());
            }
        }
        if self.flags.cold_resistance > 0 {
            self.flags.cold_resistance -= 1;
            if self.flags.cold_resistance == 0 {
                result.messages.push("You no longer feel safe from cold.".to_string());
            }
        }

        // 12. Detect invisible & infra
        if self.flags.detect_invisible > 0 {
            self.flags.detect_invisible -= 1;
        }
        if self.flags.timed_infra > 0 {
            self.flags.timed_infra -= 1;
        }

        // 13. Hallucination
        if self.flags.image > 0 {
            self.flags.image -= 1;
        }

        // 14. Paralysis
        if self.flags.paralysis > 0 {
            self.flags.paralysis -= 1;
        }

        // 15. Word of recall
        if self.flags.word_of_recall > 0 {
            if self.flags.word_of_recall == 1 {
                self.flags.word_of_recall = 0;
                result.recall_triggered = true;
            } else {
                self.flags.word_of_recall -= 1;
            }
        }

        result
    }

    pub fn update_max_hp_and_mana(&mut self) {
        let base_hp = self.base_hp_levels[(self.level as usize - 1).min(39)];
        let con_bonus = self.get_con_hp_bonus();
        let calculated = base_hp + (con_bonus * self.level as i32);
        self.max_hp = calculated.max(self.level as i32 + 1);
        self.hp = self.hp.min(self.max_hp);

        let is_caster = !matches!(self.class, Class::Warrior);
        if is_caster {
            let stat_val = match self.class {
                Class::Mage | Class::Rogue | Class::Ranger => self.stats.intelligence,
                _ => self.stats.wisdom,
            };
            let effective_learnt = if self.is_wizard {
                self.spells_learnt.max(1)
            } else {
                self.spells_learnt
            };
            self.max_mana = crate::magic::calculate_max_mana(self.level, self.class, stat_val, effective_learnt);
        } else {
            self.max_mana = 0;
        }
        self.mana = self.mana.min(self.max_mana);
        self.update_spells_to_learn();
    }

    pub fn is_mage_caster(&self) -> bool {
        self.is_wizard || matches!(self.class, Class::Mage | Class::Rogue | Class::Ranger)
    }

    pub fn is_priest_caster(&self) -> bool {
        self.is_wizard || matches!(self.class, Class::Priest | Class::Paladin)
    }

    pub fn knows_spell(&self, spell_idx: usize) -> bool {
        self.is_wizard || (self.spells_learnt & (1 << spell_idx)) != 0
    }

    pub fn learn_spell(&mut self, spell_idx: usize) {
        self.spells_learnt |= 1 << spell_idx;
        for slot in self.spells_learned_order.iter_mut() {
            if *slot == 99 {
                *slot = spell_idx as u8;
                break;
            }
        }
        if self.new_spells_to_learn > 0 {
            self.new_spells_to_learn -= 1;
        }
        if self.max_mana == 0 {
            self.update_max_hp_and_mana();
            self.mana = self.max_mana;
        }
    }

    pub fn update_spells_to_learn(&mut self) {
        let stat_val = if self.is_mage_caster() {
            self.stats.intelligence
        } else if self.is_priest_caster() {
            self.stats.wisdom
        } else {
            self.new_spells_to_learn = 0;
            return;
        };
        let allowed = crate::magic::number_of_spells_allowed(self.level, self.class, stat_val);
        let known = self.spells_learnt.count_ones();
        if allowed > known {
            self.new_spells_to_learn = allowed - known;
        } else {
            self.new_spells_to_learn = 0;
        }
    }

    pub fn carried_books(&self, is_prayer: bool) -> Vec<&Item> {
        self.inventory
            .iter()
            .filter(|item| {
                if is_prayer {
                    matches!(item.item_type, ItemType::PrayerBook { .. }) || item.name.contains("Prayerbook")
                } else {
                    matches!(item.item_type, ItemType::MagicBook { .. }) || item.name.contains("Spellbook")
                }
            })
            .collect()
    }

    pub fn carried_spell_flags(&self, is_prayer: bool) -> u32 {
        let mut flags = 0u32;
        for item in self.carried_books(is_prayer) {
            match item.item_type {
                ItemType::MagicBook { spell_flags } => flags |= spell_flags,
                ItemType::PrayerBook { spell_flags } => flags |= spell_flags,
                _ => {
                    if item.name.contains("Beginner") {
                        flags |= if is_prayer { 0x000000FF } else { 0x0000007F };
                    } else if item.name.contains("Incantation") || item.name.contains("Words") || item.name.contains("Magick I") {
                        flags |= if is_prayer { 0x0000FF00 } else { 0x0000FF80 };
                    } else if item.name.contains("Sorcery") || item.name.contains("Chants") || item.name.contains("Magick II") {
                        flags |= if is_prayer { 0x01FF0000 } else { 0x00FF0000 };
                    } else if item.name.contains("Arcana") || item.name.contains("Power") || item.name.contains("Holy") || item.name.contains("Exorcism") {
                        flags |= if is_prayer { 0x7E000000 } else { 0x7F000000 };
                    }
                }
            }
        }
        flags
    }

    pub fn learnable_spells(&self) -> Vec<usize> {
        let is_prayer = self.is_priest_caster();
        let book_flags = if is_prayer {
            0x7FFFFFFF // Priests receive prayers directly from their god
        } else {
            self.carried_spell_flags(false)
        };
        let mut list = Vec::new();
        for i in 0..31 {
            if (self.spells_learnt & (1 << i)) == 0
                && (book_flags & (1 << i)) != 0
                && let Some(def) = crate::magic::get_spell_def(self.class, i, is_prayer)
                && def.level_required <= self.level
            {
                list.push(i);
            }
        }
        list
    }

    pub fn known_spells(&self, is_prayer: bool) -> Vec<usize> {
        let carried_flags = self.carried_spell_flags(is_prayer);
        let mut list = Vec::new();
        for i in 0..31 {
            if self.knows_spell(i)
                && (self.is_wizard || (carried_flags & (1 << i)) != 0)
                && let Some(def) = crate::magic::get_spell_def(self.class, i, is_prayer)
                && (def.level_required <= self.level || self.is_wizard)
            {
                list.push(i);
            }
        }
        list
    }

    pub fn get_exp_to_next_level(&self) -> u32 {
        let base_idx = (self.level as usize - 1).min(39);
        let base_req = BASE_EXP_LEVELS[base_idx];
        base_req * self.exp_factor / 100
    }

    pub fn add_experience(&mut self, amount: u32) -> bool {
        if amount == 0 {
            return false;
        }
        self.exp += amount;
        
        let mut leveled_up = false;
        while self.level < 40 {
            let req = self.get_exp_to_next_level();
            if self.exp >= req {
                self.level += 1;
                leveled_up = true;
            } else {
                break;
            }
        }

        if leveled_up {
            self.update_max_hp_and_mana();
            self.hp = self.max_hp;
        }
        leveled_up
    }

    pub fn roll_melee_damage<R: rand::Rng>(&self, rng: &mut R) -> i32 {
        let base_dice = if let Some(weapon_item) = self.equipment.iter().find(|i| matches!(i.item_type, ItemType::Weapon { .. })) {
            match &weapon_item.item_type {
                ItemType::Weapon { damage } => *damage,
                _ => Dice::new(1, 4),
            }
        } else {
            match self.class {
                Class::Warrior => Dice::new(2, 6),
                Class::Rogue => Dice::new(1, 8),
                _ => Dice::new(1, 6),
            }
        };

        let mut damage = base_dice.roll(rng) as i32 + (self.stats.strength as i32 - 10) / 2;
        for item in &self.equipment {
            if let ItemType::Ring { bonus } = &item.item_type {
                let lname = item.name.to_lowercase();
                if lname.contains("increase damage") || lname.contains("slaying") {
                    damage += bonus;
                }
            }
        }
        if self.flags.heroism > 0 {
            damage += 2;
        }
        if self.flags.super_heroism > 0 {
            damage += 4;
        }
        if damage < 1 {
            1
        } else {
            damage
        }
    }

    pub fn digging_ability(&self) -> i32 {
        let wielded = self.equipment.iter().find(|i| matches!(i.item_type, ItemType::Weapon { .. }));
        match wielded {
            Some(weapon) => {
                let is_digging_tool = weapon.name.contains("Shovel") || weapon.name.contains("Pick") || weapon.name.contains("Mattock");
                let base_ability = self.stats.strength as i32;
                if is_digging_tool {
                    let tool_bonus = if weapon.name.contains("Pick") { 50 } else { 25 };
                    base_ability + tool_bonus
                } else {
                    let weapon_max_dmg = match &weapon.item_type {
                        ItemType::Weapon { damage } => damage.num * damage.sides,
                        _ => 4,
                    } as i32;
                    (base_ability + weapon_max_dmg) / 2
                }
            }
            None => 0,
        }
    }

    pub fn calculate_ac(&self) -> i32 {
        let mut ac = 10;
        ac += (self.stats.dexterity as i32 - 10) / 2;
        
        for item in &self.equipment {
            match &item.item_type {
                ItemType::Armor { ac: item_ac } => ac += item_ac,
                ItemType::Ring { bonus } if item.name.to_lowercase().contains("protection") => ac += bonus,
                ItemType::Amulet { bonus } if item.name.to_lowercase().contains("the magi") => ac += bonus,
                _ => {}
            }
        }

        if self.flags.invulnerability > 0 {
            ac += 100;
        }
        if self.flags.blessed > 0 {
            ac += 2;
        }
        
        ac
    }

    /// Saving throw check against magical attacks and spells according to canonical Umoria rules.
    pub fn saving_throw<R: rand::Rng>(&self, rng: &mut R) -> bool {
        let base_save = match self.class {
            Class::Warrior => 18,
            Class::Mage => 36,
            Class::Priest => 30,
            Class::Rogue => 30,
            Class::Ranger => 30,
            Class::Paladin => 24,
        };
        let race_save = match self.race {
            Race::Human => 0,
            Race::HalfElf => 3,
            Race::Elf => 6,
            Race::Halfling => 18,
            Race::Gnome => 12,
            Race::Dwarf => 9,
            Race::HalfOrc => -3,
            Race::HalfTroll => -8,
        };
        let wis_adj = crate::magic::stat_adjustment(self.stats.wisdom);
        let save_target = (base_save + race_save + wis_adj + self.level as i32).clamp(5, 95);
        rng.gen_range(1..=100) <= save_target
    }

    pub fn damage_fire(&mut self, damage: i32, status_msg: &mut String) {
        let mut dmg = damage;
        if self.flags.resistant_to_fire || self.flags.heat_resistance > 0 {
            dmg /= 3;
        }
        let final_dmg = dmg.max(1);
        self.hp -= final_dmg;
        status_msg.push_str(&format!(" You take {} fire damage!", final_dmg));
    }

    pub fn damage_cold(&mut self, damage: i32, status_msg: &mut String) {
        let mut dmg = damage;
        if self.flags.resistant_to_cold || self.flags.cold_resistance > 0 {
            dmg /= 3;
        }
        let final_dmg = dmg.max(1);
        self.hp -= final_dmg;
        status_msg.push_str(&format!(" You take {} cold damage!", final_dmg));
    }

    pub fn damage_lightning(&mut self, damage: i32, status_msg: &mut String) {
        let mut dmg = damage;
        if self.flags.resistant_to_light {
            dmg /= 3;
        }
        let final_dmg = dmg.max(1);
        self.hp -= final_dmg;
        status_msg.push_str(&format!(" You take {} lightning damage!", final_dmg));
    }

    pub fn damage_acid(&mut self, damage: i32, status_msg: &mut String) {
        let mut dmg = damage;
        if self.flags.resistant_to_acid {
            dmg /= 3;
        }
        let final_dmg = dmg.max(1);
        self.hp -= final_dmg;
        status_msg.push_str(&format!(" You take {} acid damage!", final_dmg));
    }

    pub fn damage_poison_gas(&mut self, damage: i32, status_msg: &mut String) {
        let dmg = damage;
        let final_dmg = dmg.max(1);
        self.hp -= final_dmg;
        self.flags.poisoned += 5;
        status_msg.push_str(&format!(" You take {} poison gas damage!", final_dmg));
    }

    pub fn take_damage(&mut self, damage: i32) {
        let actual_damage = if self.flags.invulnerability > 0 { 0 } else { damage.max(0) };
        self.hp -= actual_damage;
    }

    /// Total weight of items carried in inventory (in 1/10th lbs).
    pub fn total_weight(&self) -> u32 {
        self.inventory.iter().map(|item| item.weight * item.count).sum()
    }

    /// Maximum carrying capacity (in 1/10th lbs) according to canonical Umoria formula:
    /// `capacity = (STR * 130 + body_weight).min(3000)`.
    pub fn carrying_capacity(&self) -> u32 {
        let str_val = self.stats.strength.max(3) as u32;
        let cap = str_val * 130 + self.body_weight;
        cap.min(3000)
    }

    /// Returns true if the player's pack weight exceeds their carrying load limit.
    pub fn is_encumbered(&self) -> bool {
        self.total_weight() > self.carrying_capacity()
    }

    /// Returns the encumbrance speed penalty level (0 if unencumbered).
    pub fn encumbrance_penalty(&self) -> u32 {
        let cap = self.carrying_capacity();
        let total = self.total_weight();
        if total > cap {
            total / (cap + 1)
        } else {
            0
        }
    }

    /// Returns the relative speed modifier of the player (0 = normal, <0 = faster, >0 = slower).
    /// Matches canonical Umoria:
    /// - Haste (flags.fast > 0): -1
    /// - Slow (flags.slow > 0): +1
    /// - Searching: +1
    /// - Encumbrance: +encumbrance_penalty()
    /// - Speed equipment (e.g. Ring of Speed, Boots of Speed): -bonus
    pub fn speed_modifier(&self) -> i16 {
        let mut speed: i16 = 0;

        if self.flags.fast > 0 {
            speed -= 1;
        }
        if self.flags.slow > 0 {
            speed += 1;
        }
        if self.searching {
            speed += 1;
        }
        speed += self.encumbrance_penalty() as i16;

        for item in &self.equipment {
            let name_lower = item.name.to_lowercase();
            if name_lower.contains("speed") {
                match &item.item_type {
                    ItemType::Ring { bonus } => {
                        speed -= *bonus as i16;
                    }
                    ItemType::Armor { .. } => {
                        speed -= 1;
                    }
                    _ => {}
                }
            }
        }

        speed
    }

    /// Returns the canonical status string for speed (Very Fast, Fast, Slow, Very Slow, or empty).
    pub fn speed_status_string(&self) -> &'static str {
        let speed = self.speed_modifier();
        if speed > 1 {
            "Very Slow"
        } else if speed == 1 {
            "Slow     "
        } else if speed == -1 {
            "Fast     "
        } else if speed < -1 {
            "Very Fast"
        } else {
            "         "
        }
    }

    pub fn apply_race_and_class_modifiers(&mut self) {
        let (r_str, r_int, r_wis, r_dex, r_con, r_chr) = match self.race {
            Race::Human => (0, 0, 0, 0, 0, 0),
            Race::HalfElf => (-1, 1, 0, 1, -1, 1),
            Race::Elf => (-1, 2, 1, 1, -2, 1),
            Race::Halfling => (-2, 2, 1, 3, 1, 1),
            Race::Gnome => (-1, 2, 0, 2, 1, -2),
            Race::Dwarf => (2, -3, 1, -2, 2, -3),
            Race::HalfOrc => (2, -1, 0, 0, 1, -4),
            Race::HalfTroll => (4, -4, -2, -4, 3, -6),
        };

        let (c_str, c_int, c_wis, c_dex, c_con, c_chr) = match self.class {
            Class::Warrior => (5, -2, -2, 2, 2, -1),
            Class::Mage => (-5, 3, 0, 1, -2, 1),
            Class::Priest => (-3, -3, 3, -1, 0, 2),
            Class::Rogue => (2, 1, -2, 3, 1, -1),
            Class::Ranger => (2, 2, 0, 1, 1, 1),
            Class::Paladin => (3, -3, 1, 0, 2, 2),
        };

        self.stats.strength = (self.stats.strength + r_str + c_str).max(2);
        self.stats.intelligence = (self.stats.intelligence + r_int + c_int).max(2);
        self.stats.wisdom = (self.stats.wisdom + r_wis + c_wis).max(2);
        self.stats.dexterity = (self.stats.dexterity + r_dex + c_dex).max(2);
        self.stats.constitution = (self.stats.constitution + r_con + c_con).max(2);
        self.stats.charisma = (self.stats.charisma + r_chr + c_chr).max(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_radius_and_fuel() {
        let mut player = Player::new("TestPlayer", Race::Human, Class::Warrior, 10, 10);
        // Default player with no light equipped has radius 1
        assert_eq!(player.get_light_radius(), 1);

        // Equip a torch with 4000 fuel -> radius should be 2
        player.equipment.push(Item::new("Wooden Torch", 1, 15, ItemType::Light { fuel: 4000 }));
        assert_eq!(player.get_light_radius(), 2);

        // Discard the torch and equip a lantern with 7500 fuel -> radius should be 3
        player.equipment.clear();
        player.equipment.push(Item::new("Brass Lantern", 1, 35, ItemType::Light { fuel: 7500 }));
        assert_eq!(player.get_light_radius(), 3);

        // Drain the fuel to 0 -> radius should be 1
        if let ItemType::Light { ref mut fuel } = player.equipment[0].item_type {
            *fuel = 0;
        }
        assert_eq!(player.get_light_radius(), 1);
    }

    #[test]
    fn test_equipment_slots() {
        let dagger = Item::new("Iron Dagger", 1, 10, ItemType::Weapon { damage: Dice::new(1, 4) });
        let shield = Item::new("Iron Shield", 1, 150, ItemType::Armor { ac: 3 });
        let mail = Item::new("Chain Mail", 1, 250, ItemType::Armor { ac: 7 });
        let boots = Item::new("Leather Boots", 1, 30, ItemType::Armor { ac: 1 });
        let helm = Item::new("Iron Helm", 1, 50, ItemType::Armor { ac: 2 });
        let torch = Item::new("Wooden Torch", 1, 15, ItemType::Light { fuel: 4000 });
        let bow = Item::new("Short Bow", 1, 30, ItemType::Bow { multiplier: 2 });

        assert_eq!(dagger.get_equipment_slot(), "Weapon");
        assert_eq!(shield.get_equipment_slot(), "Shield");
        assert_eq!(mail.get_equipment_slot(), "Body Armor");
        assert_eq!(boots.get_equipment_slot(), "Boots");
        assert_eq!(helm.get_equipment_slot(), "Headwear");
        assert_eq!(torch.get_equipment_slot(), "Light Source");
        assert_eq!(bow.get_equipment_slot(), "Ranged Weapon");
    }

    #[test]
    fn test_town_npc_pacifism() {
        let player = Player::new("TestPlayer", Race::Human, Class::Warrior, 10, 10);
        let mut monster = crate::entity::monster::Monster::new("Filthy Street Urchin", 'p', 11, 11, 6, crate::dice::Dice::new(1, 2), 0);

        // Initially on town level (depth 0), the monster should not be hostile to player
        let depth = 0;
        let is_hostile = depth > 0 || monster.was_attacked || player.killed_town_npcs > player.town_npc_attack_threshold;
        assert!(!is_hostile, "Monster should be peaceful initially on town level");

        // If the depth > 0, the monster is always hostile
        let deep_hostile = 1 > 0 || monster.was_attacked || player.killed_town_npcs > player.town_npc_attack_threshold;
        assert!(deep_hostile, "Monster should be hostile in the dungeon");

        // If the monster is attacked, it becomes hostile
        monster.was_attacked = true;
        let attacked_hostile = depth > 0 || monster.was_attacked || player.killed_town_npcs > player.town_npc_attack_threshold;
        assert!(attacked_hostile, "Monster should be hostile after being attacked");
        
        // Reset monster attacked flag
        monster.was_attacked = false;

        // If player has killed more than the threshold NPCs, the monster becomes hostile
        let mut player_aggro = player.clone();
        player_aggro.killed_town_npcs = player_aggro.town_npc_attack_threshold + 1;
        let threshold_hostile = depth > 0 || monster.was_attacked || player_aggro.killed_town_npcs > player_aggro.town_npc_attack_threshold;
        assert!(threshold_hostile, "Monster should be hostile if player exceeded kill threshold");
    }

    #[test]
    fn test_minimum_stat_clamping() {
        let stats = Attributes::new(1, 1, 1, 1, 1, 1);
        let mut player = Player::new("TestClamping", Race::Halfling, Class::Mage, 10, 10);
        player.stats = stats;
        player.apply_race_and_class_modifiers();

        assert!(player.stats.strength >= 2, "Strength was not clamped to minimum 2");
        assert!(player.stats.intelligence >= 2, "Intelligence was not clamped to minimum 2");
        assert!(player.stats.wisdom >= 2, "Wisdom was not clamped to minimum 2");
        assert!(player.stats.dexterity >= 2, "Dexterity was not clamped to minimum 2");
        assert!(player.stats.constitution >= 2, "Constitution was not clamped to minimum 2");
        assert!(player.stats.charisma >= 2, "Charisma was not clamped to minimum 2");
    }

    #[test]
    fn test_look_command_los() {
        use crate::dungeon::{DungeonLevel, tile::TileType};
        let mut level = DungeonLevel::new(10, 10, 1, 1);
        for x in 1..=8 {
            if let Some(tile) = level.get_tile_mut(x, 1) {
                tile.tile_type = TileType::Floor;
                tile.visible = true;
            }
        }
        let mut player = Player::new("TestLook", Race::Human, Class::Warrior, 10, 10);
        player.move_to(1, 1);

        let monster = crate::entity::monster::Monster::new("TestMonster", 'm', 5, 1, 10, crate::dice::Dice::new(1, 1), 0);
        let monsters = [monster];

        let dx = 1;
        let dy = 0;
        let mut sx = player.x as isize;
        let mut sy = player.y as isize;
        let mut npc_found = None;
        loop {
            sx += dx;
            sy += dy;
            if sx < 0 || sx >= level.width as isize || sy < 0 || sy >= level.height as isize {
                break;
            }
            let x_u = sx as usize;
            let y_u = sy as usize;
            if let Some(tile) = level.get_tile(x_u, y_u) {
                if !tile.is_passable() || tile.tile_type == TileType::SecretDoor {
                    break;
                }
                if tile.visible
                    && let Some(m) = monsters.iter().find(|m| m.x == x_u && m.y == y_u) {
                        npc_found = Some(m.name.clone());
                        break;
                    }
            } else {
                break;
            }
        }
        assert_eq!(npc_found, Some("TestMonster".to_string()));

        if let Some(tile) = level.get_tile_mut(3, 1) {
            tile.tile_type = TileType::Wall;
        }

        let mut sx = player.x as isize;
        let mut sy = player.y as isize;
        let mut npc_found = None;
        loop {
            sx += dx;
            sy += dy;
            if sx < 0 || sx >= level.width as isize || sy < 0 || sy >= level.height as isize {
                break;
            }
            let x_u = sx as usize;
            let y_u = sy as usize;
            if let Some(tile) = level.get_tile(x_u, y_u) {
                if !tile.is_passable() || tile.tile_type == TileType::SecretDoor {
                    break;
                }
                if tile.visible
                    && let Some(m) = monsters.iter().find(|m| m.x == x_u && m.y == y_u) {
                        npc_found = Some(m.name.clone());
                        break;
                    }
            } else {
                break;
            }
        }
        assert_eq!(npc_found, None);
    }

    #[test]
    fn test_item_inscription() {
        let mut item = Item::new("Short Sword", 1, 10, ItemType::Weapon { damage: crate::dice::Dice::new(1, 6) });
        assert_eq!(item.display_name(), "Short Sword");

        item.inscription = Some("sharp".to_string());
        assert_eq!(item.display_name(), "Short Sword {sharp}");

        item.inscription = None;
        assert_eq!(item.display_name(), "Short Sword");
    }

    #[test]
    fn test_digestion_rate_and_natural_regeneration() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.food = 7500;
        player.hp = 10;
        player.max_hp = 30;
        player.mana = 5;
        player.max_mana = 20;

        // Turn 1: 7500 is divisible by 20 and 15 -> natural regen ticks
        let (_msg, starved) = player.tick_digestion_and_regen(false);
        assert!(!starved);
        assert_eq!(player.food, 7499, "Digestion must consume exactly 1 food per turn");
        assert_eq!(player.hp, 11, "HP regenerates +1 on div 20 turn");
        assert_eq!(player.mana, 6, "Mana regenerates +1 on div 15 turn");

        // Next 10 turns: food decrements by 1 each turn, no HP regen until turn % 20 == 0
        for _ in 0..10 {
            player.tick_digestion_and_regen(false);
        }
        assert_eq!(player.food, 7489);
        assert_eq!(player.hp, 11);
    }

    #[test]
    fn test_starvation_damage_and_survival() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.food = 0;
        player.hp = 10;
        player.max_hp = 25;

        // Turn 1 into starvation (food becomes -1)
        let (msg, starved) = player.tick_digestion_and_regen(false);
        assert!(!starved);
        assert_eq!(player.food, -1);
        assert_eq!(player.hp, 10, "Starvation must not deal damage immediately on step 1");
        assert_eq!(msg, Some(" You are starving!"));

        // Steps -2 to -14: NO damage taken
        for _ in 2..=14 {
            let (msg, starved) = player.tick_digestion_and_regen(false);
            assert!(!starved);
            assert_eq!(player.hp, 10, "No damage on non-tick turns");
            assert_eq!(msg, None, "No message spam on non-tick turns");
        }
        assert_eq!(player.food, -14);

        // Step -15: First starvation damage tick!
        let (msg, starved) = player.tick_digestion_and_regen(false);
        assert!(!starved);
        assert_eq!(player.food, -15);
        assert_eq!(player.hp, 9, "Starvation deals 1 damage on tick 15");
        assert_eq!(msg, Some(" You are starving!"));

        // Player with 9 HP survives next 15 * 8 = 120 turns
        for _ in 0..120 {
            let (_msg, starved) = player.tick_digestion_and_regen(false);
            assert!(!starved, "Player should survive gradual starvation");
        }
        assert_eq!(player.hp, 1, "Player should have 1 HP remaining after 8 more damage ticks");

        // Run until lethal starvation
        let mut died = false;
        for _ in 0..30 {
            let (_msg, starved) = player.tick_digestion_and_regen(false);
            if starved {
                died = true;
                break;
            }
        }
        assert!(died, "Player must eventually starve to death at HP 0");
        assert_eq!(player.hp, 0);
    }

    #[test]
    fn test_hunger_state_transitions() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);

        // Transition into hungry
        player.food = 2000;
        let (msg, _) = player.tick_digestion_and_regen(false);
        assert_eq!(msg, Some(" You are getting hungry."));
        let (msg2, _) = player.tick_digestion_and_regen(false);
        assert_eq!(msg2, None, "No message spam on subsequent turns");

        // Transition into weak
        player.food = 1000;
        let (msg, _) = player.tick_digestion_and_regen(false);
        assert_eq!(msg, Some(" You are getting weak from hunger."));

        // Transition into faint
        player.food = 300;
        let (msg, _) = player.tick_digestion_and_regen(false);
        assert_eq!(msg, Some(" You are getting faint from hunger."));
    }

    #[test]
    fn test_format_stat_18_xx() {
        assert_eq!(format_stat(3), "     3");
        assert_eq!(format_stat(10), "    10");
        assert_eq!(format_stat(18), "    18");
        assert_eq!(format_stat(19), " 18/01");
        assert_eq!(format_stat(28), " 18/10");
        assert_eq!(format_stat(117), " 18/99");
        assert_eq!(format_stat(118), "18/100");
        assert_eq!(format_stat(120), "18/100");
    }

    #[test]
    fn test_stat_drain_restore_and_sustain() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.stats.strength = 18;
        player.max_stats.strength = 18;

        let mut rng = rand::thread_rng();

        // 1. Unprotected drain reduces STR from 18 to 17
        let drained = player.drain_stat(0, &mut rng);
        assert!(drained);
        assert_eq!(player.stats.strength, 17);
        assert_eq!(player.max_stats.strength, 18);

        // 2. Sustain prevents drain
        player.flags.sustain_str = true;
        let drained_again = player.drain_stat(0, &mut rng);
        assert!(!drained_again, "Sustained stat must not be drained");
        assert_eq!(player.stats.strength, 17);

        // 3. Restore restores back to max_stats
        let restored = player.restore_stat(0);
        assert!(restored);
        assert_eq!(player.stats.strength, 18);

        // 4. Restoring an undrained stat returns false
        let restored_again = player.restore_stat(0);
        assert!(!restored_again);
    }

    #[test]
    fn test_condition_ticks_and_messages() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.flags.blind = 2;
        player.flags.confused = 1;
        player.flags.heroism = 1;

        // Turn 1
        let res1 = player.tick_conditions(1);
        assert_eq!(player.flags.blind, 1);
        assert_eq!(player.flags.confused, 0);
        assert_eq!(player.flags.heroism, 0);
        assert!(res1.messages.contains(&"You feel less confused now.".to_string()));
        assert!(res1.messages.contains(&"The heroism wears off.".to_string()));

        // Turn 2: blindness expires
        let res2 = player.tick_conditions(2);
        assert_eq!(player.flags.blind, 0);
        assert!(res2.messages.contains(&"The veil of darkness lifts.".to_string()));
    }

    #[test]
    fn test_poison_damage_and_lethality() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.hp = 2;
        player.flags.poisoned = 5;
        player.stats.constitution = 10; // con adj = 0 -> 1 damage per turn

        let res1 = player.tick_conditions(1);
        assert_eq!(player.hp, 1);
        assert!(!res1.died_from_poison);

        let res2 = player.tick_conditions(2);
        assert_eq!(player.hp, 0);
        assert!(res2.died_from_poison);
    }

    #[test]
    fn test_word_of_recall_countdown() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.flags.word_of_recall = 3;

        let res1 = player.tick_conditions(1);
        assert!(!res1.recall_triggered);
        assert_eq!(player.flags.word_of_recall, 2);

        let res2 = player.tick_conditions(2);
        assert!(!res2.recall_triggered);
        assert_eq!(player.flags.word_of_recall, 1);

        let res3 = player.tick_conditions(3);
        assert!(res3.recall_triggered);
        assert_eq!(player.flags.word_of_recall, 0);
    }

    #[test]
    fn test_poison_suppresses_natural_regen() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.food = 7500;
        player.hp = 10;
        player.max_hp = 30;
        player.flags.poisoned = 10;

        // When poisoned, 7500 divisible by 20 does NOT heal HP
        let (_msg, starved) = player.tick_digestion_and_regen(false);
        assert!(!starved);
        assert_eq!(player.hp, 10, "Poison must prevent natural HP regeneration");
    }

    #[test]
    fn test_stat_increase_into_percentile() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.stats.strength = 18;
        player.max_stats.strength = 18;

        let mut rng = rand::thread_rng();
        let increased = player.increase_stat(0, &mut rng);
        assert!(increased);
        assert!(player.stats.strength > 18);
        assert!(player.stats.strength <= 118);
        assert_eq!(player.max_stats.strength, player.stats.strength);
    }

    #[test]
    fn test_free_action_intrinsic() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.flags.free_action = true;
        assert!(player.flags.free_action);
    }

    #[test]
    fn test_identify_item_kind_updates_inventory_and_equipment() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        let mut unidentified_potion = Item::new_unidentified(
            "Potion of Healing",
            1,
            5,
            ItemType::Potion { heal_amount: 25 },
        );
        let mut rng = rand::thread_rng();
        player.flavors.assign_flavor(&mut unidentified_potion, &mut rng);
        assert!(!unidentified_potion.identified);
        assert!(unidentified_potion.flavor.is_some());
        assert!(!unidentified_potion.display_name().contains("Healing"));

        player.add_item_to_inventory(unidentified_potion);
        assert!(!player.inventory.last().unwrap().identified);

        // Identifying the item kind identifies it in inventory and registry
        let res = player.identify_item_kind("Potion of Healing");
        assert!(res);
        assert!(player.inventory.last().unwrap().identified);
        assert_eq!(player.inventory.last().unwrap().display_name(), "Potion of Healing");
        assert!(player.flavors.is_identified("Potion of Healing"));
    }

    #[test]
    fn test_unidentified_scroll_title_display() {
        let mut scroll = Item::new_unidentified(
            "Scroll of Identify",
            1,
            2,
            ItemType::Scroll { teleport: false },
        );
        scroll.flavor = Some("foo bar baz".to_string());
        assert_eq!(scroll.display_name(), "Scroll titled \"foo bar baz\"");
        scroll.identified = true;
        assert_eq!(scroll.display_name(), "Scroll of Identify");
    }

    #[test]
    fn test_ranger_starting_equipment() {
        let ranger = Player::new("Strider", Race::Elf, Class::Ranger, 10, 10);
        let has_bow = ranger.inventory.iter().any(|i| i.name == "Short Bow" && matches!(i.item_type, ItemType::Bow { multiplier: 2 }));
        let has_arrows = ranger.inventory.iter().any(|i| i.name == "Arrow" && matches!(i.item_type, ItemType::Missile { .. }));
        assert!(has_bow, "Ranger must start with a Short Bow");
        assert!(has_arrows, "Ranger must start with Arrows");
    }

    #[test]
    fn test_bow_and_missile_types() {
        let bow = Item::new("Heavy Crossbow", 1, 200, ItemType::Bow { multiplier: 4 });
        assert_eq!(bow.get_equipment_slot(), "Ranged Weapon");

        let bolt = Item::new("Bolt", 20, 3, ItemType::Missile { damage: Dice::new(1, 5) });
        assert_eq!(bolt.count, 20);
        assert_eq!(bolt.get_equipment_slot(), "Accessory");
    }

    #[test]
    fn test_rings_and_amulets_equipment_slots() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        
        let mut ring1 = Item::new("Ring of Protection", 1, 2, ItemType::Ring { bonus: 2 });
        assert_eq!(ring1.get_equipment_slot(), "On right hand");
        ring1.equipped_slot = Some("On right hand".to_string());
        player.equipment.push(ring1);

        let mut ring2 = Item::new("Ring of Resist Fire", 1, 2, ItemType::Ring { bonus: 0 });
        ring2.equipped_slot = Some("On left hand".to_string());
        player.equipment.push(ring2);

        let mut amulet = Item::new("Amulet of the Magi", 1, 3, ItemType::Amulet { bonus: 3 });
        assert_eq!(amulet.get_equipment_slot(), "Around neck");
        amulet.equipped_slot = Some("Around neck".to_string());
        player.equipment.push(amulet);

        assert_eq!(player.equipment[0].get_equipment_slot(), "On right hand");
        assert_eq!(player.equipment[1].get_equipment_slot(), "On left hand");
        assert_eq!(player.equipment[2].get_equipment_slot(), "Around neck");
    }

    #[test]
    fn test_equipment_bonuses_and_intrinsics() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.equipment.clear();
        player.update_equipment_bonuses();

        assert!(!player.flags.see_invisible);
        assert!(!player.flags.free_action);
        assert!(!player.flags.slow_digest);
        assert!(!player.flags.free_fall);
        assert!(!player.flags.resistant_to_fire);
        assert!(!player.flags.resistant_to_cold);
        assert!(!player.flags.resistant_to_acid);
        assert!(!player.flags.sustain_str);

        // Equip Ring of Free Action
        player.equipment.push(Item::new("Ring of Free Action", 1, 2, ItemType::Ring { bonus: 0 }));
        // Equip Ring of Lordly Protection (FIRE)
        player.equipment.push(Item::new("Ring of Lordly Protection (FIRE)", 1, 2, ItemType::Ring { bonus: 5 }));
        // Equip Ring of Sustain Strength
        player.equipment.push(Item::new("Ring of Sustain Strength", 1, 2, ItemType::Ring { bonus: 0 }));
        // Equip Amulet of the Magi
        player.equipment.push(Item::new("Amulet of the Magi", 1, 3, ItemType::Amulet { bonus: 3 }));

        player.update_equipment_bonuses();

        assert!(player.flags.free_action, "Must have free action from ring/amulet");
        assert!(player.flags.resistant_to_fire, "Must have fire resistance from lordly ring");
        assert!(player.flags.sustain_str, "Must have sustain strength from ring");
        assert!(player.flags.see_invisible, "Must have see invisible from Amulet of the Magi");

        // Calculate AC with bonuses: 10 base + DEX mod + 5 (ring) + 3 (amulet)
        let ac = player.calculate_ac();
        let expected_ac = 10 + (player.stats.dexterity as i32 - 10) / 2 + 5 + 3;
        assert_eq!(ac, expected_ac);

        // Unequip all items
        player.equipment.clear();
        player.update_equipment_bonuses();

        assert!(!player.flags.free_action);
        assert!(!player.flags.resistant_to_fire);
        assert!(!player.flags.sustain_str);
        assert!(!player.flags.see_invisible);
    }

    #[test]
    fn test_stat_modifiers_from_equipment() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.equipment.clear();
        player.stats.strength = 18;
        player.base_stats.strength = 18;
        player.max_stats.strength = 18;
        player.update_equipment_bonuses();
        assert_eq!(player.stats.strength, 18);

        // Equip Ring of Strength (+2)
        player.equipment.push(Item::new("Ring of Strength", 1, 2, ItemType::Ring { bonus: 2 }));
        player.update_equipment_bonuses();

        // 18 boosted by +2 becomes 18/20 (stat value 38)
        assert_eq!(player.stats.strength, 38);
        assert_eq!(format_stat(player.stats.strength), " 18/20");

        // Unequip
        player.equipment.clear();
        player.update_equipment_bonuses();
        assert_eq!(player.stats.strength, 18);
        assert_eq!(format_stat(player.stats.strength), "    18");
    }

    #[test]
    fn test_weight_capacity_and_encumbrance() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.inventory.clear();
        player.equipment.clear();
        player.body_weight = 180;
        player.stats.strength = 10;
        player.base_stats.strength = 10;

        // Capacity = STR * 130 + body_weight = 10 * 130 + 180 = 1480 (148.0 lbs)
        assert_eq!(player.carrying_capacity(), 1480);
        assert_eq!(player.total_weight(), 0);
        assert!(!player.is_encumbered());
        assert_eq!(player.encumbrance_penalty(), 0);

        // Add 100 lbs (1000 tenths) of iron spikes
        player.inventory.push(Item::new("Iron Spike", 500, 2, ItemType::Scroll { teleport: false }));
        assert_eq!(player.total_weight(), 1000);
        assert!(!player.is_encumbered());

        // Add another 50 lbs (500 tenths) -> total 1500 > 1480 -> encumbered!
        player.inventory.push(Item::new("Iron Spike", 250, 2, ItemType::Scroll { teleport: false }));
        assert_eq!(player.total_weight(), 1500);
        assert!(player.is_encumbered());
        assert_eq!(player.encumbrance_penalty(), 1500 / 1481); // 1
    }

    #[test]
    fn test_player_speed_modifier_and_conditions() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.equipment.clear();
        player.inventory.clear();

        // 1. Base speed is 0
        assert_eq!(player.speed_modifier(), 0);
        assert_eq!(player.speed_status_string(), "         ");

        // 2. Fast (Haste)
        player.flags.fast = 10;
        assert_eq!(player.speed_modifier(), -1);
        assert_eq!(player.speed_status_string(), "Fast     ");

        // 3. Fast + Slow cancel out
        player.flags.slow = 10;
        assert_eq!(player.speed_modifier(), 0);
        assert_eq!(player.speed_status_string(), "         ");

        // 4. Slow only
        player.flags.fast = 0;
        assert_eq!(player.speed_modifier(), 1);
        assert_eq!(player.speed_status_string(), "Slow     ");

        // 5. Searching mode adds +1
        player.searching = true;
        assert_eq!(player.speed_modifier(), 2);
        assert_eq!(player.speed_status_string(), "Very Slow");

        // Reset
        player.flags.slow = 0;
        player.searching = false;
        assert_eq!(player.speed_modifier(), 0);

        // 6. Ring of Speed (+2)
        player.equipment.push(Item::new("Ring of Speed", 1, 2, ItemType::Ring { bonus: 2 }));
        assert_eq!(player.speed_modifier(), -2);
        assert_eq!(player.speed_status_string(), "Very Fast");
    }

    #[test]
    fn test_speed_accelerated_food_consumption() {
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 10, 10);
        player.food = 1000;
        player.equipment.clear();

        // Normal speed: 1 food consumed
        player.tick_digestion_and_regen(false);
        assert_eq!(player.food, 999);

        // Sped up (-1 speed): 1 base + (-1)^2 = 2 food consumed
        player.flags.fast = 10;
        player.tick_digestion_and_regen(false);
        assert_eq!(player.food, 997);

        // Sped up (-2 speed): 1 base + (-2)^2 = 5 food consumed
        player.equipment.push(Item::new("Ring of Speed", 1, 2, ItemType::Ring { bonus: 1 }));
        assert_eq!(player.speed_modifier(), -2);
        player.tick_digestion_and_regen(false);
        assert_eq!(player.food, 992);
    }
}
