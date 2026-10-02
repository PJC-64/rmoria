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
                    _ => format!("{} {}", flv, self.name),
                }
            } else {
                self.name.clone()
            }
        } else {
            self.name.clone()
        };

        if let Some(ref ins) = self.inscription {
            format!("{} {{{}}}", base, ins)
        } else {
            base
        }
    }

    pub fn get_equipment_slot(&self) -> &'static str {
        match &self.item_type {
            ItemType::Weapon { .. } => "Weapon",
            ItemType::Bow { .. } => "Ranged Weapon",
            ItemType::Light { .. } => "Light Source",
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

fn default_town_threshold() -> u32 {
    use rand::Rng;
    rand::thread_rng().gen_range(1..=5)
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
    pub max_stats: Attributes,
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
            max_stats: base_stats,
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
                    starter.push(Item::new("Mage Spellbook [Beginner's Magick]", 1, 20, ItemType::Scroll { teleport: false }));
                } else if matches!(class, Class::Priest | Class::Paladin) {
                    starter.push(Item::new("Priest Prayerbook [Beginner's Handbook]", 1, 20, ItemType::Scroll { teleport: false }));
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
            base_hp_levels,
            exp_factor,
            history,
        };

        player.flavors.identify("Potion of Cure Light Wounds");
        player.flavors.identify("Scroll of Phase Door");
        
        player.apply_race_and_class_modifiers();
        player.max_stats = player.stats.clone();
        player.update_max_hp_and_mana();
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

    /// Drains a stat by 1 or percentile chunk according to canonical Umoria rules.
    /// If sustained, does nothing and returns false.
    pub fn drain_stat<R: rand::Rng>(&mut self, stat_idx: usize, rng: &mut R) -> bool {
        if self.is_stat_sustained(stat_idx) {
            return false;
        }

        let current = match stat_idx {
            0 => self.stats.strength,
            1 => self.stats.intelligence,
            2 => self.stats.wisdom,
            3 => self.stats.dexterity,
            4 => self.stats.constitution,
            5 => self.stats.charisma,
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
            0 => self.stats.strength = new_stat,
            1 => self.stats.intelligence = new_stat,
            2 => self.stats.wisdom = new_stat,
            3 => self.stats.dexterity = new_stat,
            4 => self.stats.constitution = new_stat,
            5 => self.stats.charisma = new_stat,
            _ => {}
        }

        self.update_max_hp_and_mana();
        true
    }

    /// Restores a single stat back to its natural maximum.
    pub fn restore_stat(&mut self, stat_idx: usize) -> bool {
        let (current, max) = match stat_idx {
            0 => (self.stats.strength, self.max_stats.strength),
            1 => (self.stats.intelligence, self.max_stats.intelligence),
            2 => (self.stats.wisdom, self.max_stats.wisdom),
            3 => (self.stats.dexterity, self.max_stats.dexterity),
            4 => (self.stats.constitution, self.max_stats.constitution),
            5 => (self.stats.charisma, self.max_stats.charisma),
            _ => return false,
        };

        if current >= max {
            return false;
        }

        match stat_idx {
            0 => self.stats.strength = max,
            1 => self.stats.intelligence = max,
            2 => self.stats.wisdom = max,
            3 => self.stats.dexterity = max,
            4 => self.stats.constitution = max,
            5 => self.stats.charisma = max,
            _ => {}
        }

        self.update_max_hp_and_mana();
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
        let current = match stat_idx {
            0 => self.stats.strength,
            1 => self.stats.intelligence,
            2 => self.stats.wisdom,
            3 => self.stats.dexterity,
            4 => self.stats.constitution,
            5 => self.stats.charisma,
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
                self.stats.strength = new_stat;
                if new_stat > self.max_stats.strength { self.max_stats.strength = new_stat; }
            }
            1 => {
                self.stats.intelligence = new_stat;
                if new_stat > self.max_stats.intelligence { self.max_stats.intelligence = new_stat; }
            }
            2 => {
                self.stats.wisdom = new_stat;
                if new_stat > self.max_stats.wisdom { self.max_stats.wisdom = new_stat; }
            }
            3 => {
                self.stats.dexterity = new_stat;
                if new_stat > self.max_stats.dexterity { self.max_stats.dexterity = new_stat; }
            }
            4 => {
                self.stats.constitution = new_stat;
                if new_stat > self.max_stats.constitution { self.max_stats.constitution = new_stat; }
            }
            5 => {
                self.stats.charisma = new_stat;
                if new_stat > self.max_stats.charisma { self.max_stats.charisma = new_stat; }
            }
            _ => {}
        }

        self.update_max_hp_and_mana();
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
            let min_level = match self.class {
                Class::Mage => 1,
                Class::Priest => 1,
                Class::Rogue => 5,
                Class::Ranger => 3,
                Class::Paladin => 1,
                Class::Warrior => 99,
            };
            if self.level >= min_level {
                let stat_val = match self.class {
                    Class::Mage | Class::Rogue | Class::Ranger => self.stats.intelligence,
                    _ => self.stats.wisdom,
                };
                let mana_per_level = if stat_val < 10 {
                    0
                } else if stat_val <= 14 {
                    1
                } else if stat_val <= 16 {
                    2
                } else if stat_val == 17 {
                    3
                } else if stat_val == 18 {
                    4
                } else {
                    5
                };
                self.max_mana = ((self.level - min_level + 1) as i32 * mana_per_level).max(0);
            } else {
                self.max_mana = 0;
            }
        } else {
            self.max_mana = 0;
        }
        self.mana = self.mana.min(self.max_mana);
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
            if let ItemType::Armor { ac: item_ac } = &item.item_type {
                ac += item_ac;
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
}
