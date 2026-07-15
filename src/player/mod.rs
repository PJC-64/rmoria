use serde::{Serialize, Deserialize};
use crate::dice::Dice;

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
}

impl Player {
    pub fn new(name: &str, race: Race, class: Class, x: usize, y: usize) -> Self {
        let base_stats = Attributes::new(10, 10, 10, 10, 10, 10);
        let mut player = Self {
            name: name.to_string(),
            race,
            class,
            level: 1,
            exp: 0,
            gold: 150,
            x,
            y,
            max_hp: 15,
            hp: 15,
            max_mana: 0,
            mana: 0,
            stats: base_stats,
            inventory: vec![
                Item::new("Dagger", 1, 10, ItemType::Weapon { damage: Dice::new(1, 4) }),
                Item::new("Leather Armor", 1, 80, ItemType::Armor { ac: 4 }),
                Item::new("Wooden Torch", 1, 15, ItemType::Scroll { teleport: false }), // Treated as accessory/light
                Item::new("Potion of Cure Light Wounds", 2, 5, ItemType::Potion { heal_amount: 10 }),
                Item::new("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true }),
            ],
            equipment: Vec::new(),
            balrog_killed: false,
        };
        player.apply_race_and_class_modifiers();
        player
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn get_light_radius(&self) -> usize {
        // Return radius 2 if wielding a Torch, otherwise radius 1 in pitch dark
        if self.equipment.iter().any(|item| item.name.contains("Torch") || item.name.contains("Lantern")) {
            2
        } else {
            1
        }
    }

    pub fn add_experience(&mut self, amount: u32) -> bool {
        if amount == 0 {
            return false;
        }
        self.exp += amount;
        let next_level_threshold = match self.level {
            1 => 100,
            2 => 300,
            3 => 600,
            4 => 1000,
            5 => 1500,
            l => l * 1000,
        };

        if self.exp >= next_level_threshold {
            self.level += 1;
            self.max_hp += 8;
            self.hp = self.max_hp;
            true
        } else {
            false
        }
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

        let (c_str, c_int, c_wis, c_dex, c_con, c_chr, base_hp) = match self.class {
            Class::Warrior => (5, -2, -2, 2, 2, -1, 18),
            Class::Mage => (-5, 3, 0, 1, -2, 1, 15),
            Class::Priest => (-3, -3, 3, -1, 0, 2, 16),
            Class::Rogue => (2, 1, -2, 3, 1, -1, 16),
            Class::Ranger => (2, 2, 0, 1, 1, 1, 17),
            Class::Paladin => (3, -3, 1, 0, 2, 2, 18),
        };

        self.stats.strength += r_str + c_str;
        self.stats.intelligence += r_int + c_int;
        self.stats.wisdom += r_wis + c_wis;
        self.stats.dexterity += r_dex + c_dex;
        self.stats.constitution += r_con + c_con;
        self.stats.charisma += r_chr + c_chr;

        self.max_hp = base_hp as i32 + (self.stats.constitution as i32 - 10) / 2;
        if self.max_hp < 1 {
            self.max_hp = 1;
        }
        self.hp = self.max_hp;
    }
}
