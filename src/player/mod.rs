use serde::{Serialize, Deserialize};
use crate::dice::Dice;
use rand::Rng;

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub count: u32,
    pub weight: u32,
    pub item_type: ItemType,
}

impl Item {
    pub fn new(name: &str, count: u32, weight: u32, item_type: ItemType) -> Self {
        Self {
            name: name.to_string(),
            count,
            weight,
            item_type,
        }
    }
}

// C++ Original Experience Levels Table
pub const BASE_EXP_LEVELS: &[u32] = &[
    10,   25,   45,    70,    100,   140,   200,    280,    380,    500,
    650,     850,     1100,    1400,    1800,    2300,    2900,     3600,     4400,     5400,
    6800, 8400, 10200, 12500, 17500, 25000, 35000, 50000, 75000, 100000,
    150000, 200000, 300000, 400000, 500000, 750000, 1500000, 2500000, 5000000, 10000000
];

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
    
    pub stats: Attributes,
    pub inventory: Vec<Item>,
    pub equipment: Vec<Item>,
    pub balrog_killed: bool,
    
    // Character progression details
    pub base_hp_levels: Vec<i32>,
    pub exp_factor: u32,
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
            stats: base_stats,
            inventory: {
                let mut starter = vec![
                    Item::new("Dagger", 1, 10, ItemType::Weapon { damage: Dice::new(1, 4) }),
                    Item::new("Leather Armor", 1, 80, ItemType::Armor { ac: 4 }),
                    Item::new("Wooden Torch", 1, 15, ItemType::Scroll { teleport: false }),
                    Item::new("Potion of Cure Light Wounds", 2, 5, ItemType::Potion { heal_amount: 10 }),
                    Item::new("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true }),
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
            base_hp_levels,
            exp_factor,
        };
        
        player.apply_race_and_class_modifiers();
        player.update_max_hp_and_mana();
        player.hp = player.max_hp;
        player.mana = player.max_mana;
        player
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn get_light_radius(&self) -> usize {
        if self.equipment.iter().any(|item| item.name.contains("Torch") || item.name.contains("Lantern")) {
            2
        } else {
            1
        }
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

        // Caster Max Mana scaling
        let is_caster = match self.class {
            Class::Warrior => false,
            _ => true,
        };
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
            self.hp = self.max_hp; // Heal to full on level up
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

        self.stats.strength += r_str + c_str;
        self.stats.intelligence += r_int + c_int;
        self.stats.wisdom += r_wis + c_wis;
        self.stats.dexterity += r_dex + c_dex;
        self.stats.constitution += r_con + c_con;
        self.stats.charisma += r_chr + c_chr;
    }
}
