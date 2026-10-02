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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: i16,
    pub intelligence: i16,
    pub wisdom: i16,
    pub dexterity: i16,
    pub constitution: i16,
    pub charisma: i16,
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
}

impl Item {
    pub fn new(name: &str, count: u32, weight: u32, item_type: ItemType) -> Self {
        Self {
            name: name.to_string(),
            count,
            weight,
            item_type,
            inscription: None,
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(ref ins) = self.inscription {
            format!("{} {{{}}}", self.name, ins)
        } else {
            self.name.clone()
        }
    }

    pub fn get_equipment_slot(&self) -> &'static str {
        match &self.item_type {
            ItemType::Weapon { .. } => "Weapon",
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
            stats: base_stats,
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
        
        player.apply_race_and_class_modifiers();
        player.update_max_hp_and_mana();
        player.hp = player.max_hp;
        player.mana = player.max_mana;
        player
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
        if !is_resting && self.food > 0 {
            if self.food % 20 == 0 {
                self.hp = (self.hp + 1).min(self.max_hp);
            }
            if self.food % 15 == 0 {
                self.mana = (self.mana + 1).min(self.max_mana);
            }
        }

        // Food consumption: 1 unit per turn
        self.food -= 1;

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
        } else if con <= 16 {
            0
        } else if con == 17 {
            1
        } else if con == 18 {
            2
        } else {
            3
        }
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

        let damage = base_dice.roll(rng) as i32 + (self.stats.strength as i32 - 10) / 2;
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

        assert_eq!(dagger.get_equipment_slot(), "Weapon");
        assert_eq!(shield.get_equipment_slot(), "Shield");
        assert_eq!(mail.get_equipment_slot(), "Body Armor");
        assert_eq!(boots.get_equipment_slot(), "Boots");
        assert_eq!(helm.get_equipment_slot(), "Headwear");
        assert_eq!(torch.get_equipment_slot(), "Light Source");
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
}
