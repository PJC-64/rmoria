//! Directly ported from Umoria's `data_treasure.cpp`, `data_stores.cpp`, and `treasure.cpp`.

#![allow(dead_code)]

use crate::dice::Dice;
use crate::dungeon::ShopType;
use crate::player::{Item, ItemType};

// Category IDs (TV_*)
pub const TV_NOTHING: u8 = 0;
pub const TV_MISC: u8 = 1;
pub const TV_CHEST: u8 = 2;
pub const TV_SLING_AMMO: u8 = 10;
pub const TV_BOLT: u8 = 11;
pub const TV_ARROW: u8 = 12;
pub const TV_SPIKE: u8 = 13;
pub const TV_LIGHT: u8 = 15;
pub const TV_BOW: u8 = 20;
pub const TV_HAFTED: u8 = 21;
pub const TV_POLEARM: u8 = 22;
pub const TV_SWORD: u8 = 23;
pub const TV_DIGGING: u8 = 25;
pub const TV_BOOTS: u8 = 30;
pub const TV_GLOVES: u8 = 31;
pub const TV_CLOAK: u8 = 32;
pub const TV_HELM: u8 = 33;
pub const TV_SHIELD: u8 = 34;
pub const TV_HARD_ARMOR: u8 = 35;
pub const TV_SOFT_ARMOR: u8 = 36;
pub const TV_AMULET: u8 = 40;
pub const TV_RING: u8 = 45;
pub const TV_STAFF: u8 = 55;
pub const TV_WAND: u8 = 65;
pub const TV_SCROLL1: u8 = 70;
pub const TV_SCROLL2: u8 = 71;
pub const TV_POTION1: u8 = 75;
pub const TV_POTION2: u8 = 76;
pub const TV_FLASK: u8 = 77;
pub const TV_FOOD: u8 = 80;
pub const TV_MAGIC_BOOK: u8 = 90;
pub const TV_PRAYER_BOOK: u8 = 91;
pub const TV_GOLD: u8 = 100;
pub const TV_INVIS_TRAP: u8 = 101;
pub const TV_VIS_TRAP: u8 = 102;
pub const TV_RUBBLE: u8 = 103;
pub const TV_OPEN_DOOR: u8 = 104;
pub const TV_CLOSED_DOOR: u8 = 105;
pub const TV_UP_STAIR: u8 = 107;
pub const TV_DOWN_STAIR: u8 = 108;
pub const TV_SECRET_DOOR: u8 = 109;
pub const TV_STORE_DOOR: u8 = 110;

// Wearable item capability and magic flags (TR_*)
pub const TR_STR: u32 = 0x0000_0001;
pub const TR_INT: u32 = 0x0000_0002;
pub const TR_WIS: u32 = 0x0000_0004;
pub const TR_DEX: u32 = 0x0000_0008;
pub const TR_CON: u32 = 0x0000_0010;
pub const TR_CHR: u32 = 0x0000_0020;
pub const TR_SEARCH: u32 = 0x0000_0040;
pub const TR_SLOW_DIGEST: u32 = 0x0000_0080;
pub const TR_STEALTH: u32 = 0x0000_0100;
pub const TR_AGGRAVATE: u32 = 0x0000_0200;
pub const TR_TELEPORT: u32 = 0x0000_0400;
pub const TR_REGEN: u32 = 0x0000_0800;
pub const TR_SPEED: u32 = 0x0000_1000;
pub const TR_SLAY_DRAGON: u32 = 0x0000_2000;
pub const TR_SLAY_ANIMAL: u32 = 0x0000_4000;
pub const TR_SLAY_EVIL: u32 = 0x0000_8000;
pub const TR_SLAY_UNDEAD: u32 = 0x0001_0000;
pub const TR_FROST_BRAND: u32 = 0x0002_0000;
pub const TR_FLAME_TONGUE: u32 = 0x0004_0000;
pub const TR_RES_FIRE: u32 = 0x0008_0000;
pub const TR_RES_ACID: u32 = 0x0010_0000;
pub const TR_RES_COLD: u32 = 0x0020_0000;
pub const TR_SUST_STAT: u32 = 0x0040_0000;
pub const TR_FREE_ACT: u32 = 0x0080_0000;
pub const TR_SEE_INVIS: u32 = 0x0100_0000;
pub const TR_RES_LIGHT: u32 = 0x0200_0000;
pub const TR_FFALL: u32 = 0x0400_0000;
pub const TR_BLIND: u32 = 0x0800_0000;
pub const TR_TIMID: u32 = 0x1000_0000;
pub const TR_TUNNEL: u32 = 0x2000_0000;
pub const TR_INFRA: u32 = 0x4000_0000;
pub const TR_CURSED: u32 = 0x8000_0000;
pub const TR_STATS: u32 = 0x0000_003F;
pub const TR_EGO_WEAPON: u32 = 0x0007_E000;

// Chest trap bitflags from Umoria
pub const CH_LOCKED: u32 = 0x0000_0001;
pub const CH_TRAPPED: u32 = 0x0000_01F0;
pub const CH_LOSE_STR: u32 = 0x0000_0010;
pub const CH_POISON: u32 = 0x0000_0020;
pub const CH_PARALYSED: u32 = 0x0000_0040;
pub const CH_EXPLODE: u32 = 0x0000_0080;
pub const CH_SUMMON: u32 = 0x0000_0100;

// Special name IDs from Umoria SpecialNameIds
pub const SN_NULL: usize = 0;
pub const SN_R: usize = 1;
pub const SN_RA: usize = 2;
pub const SN_RF: usize = 3;
pub const SN_RC: usize = 4;
pub const SN_RL: usize = 5;
pub const SN_HA: usize = 6;
pub const SN_DF: usize = 7;
pub const SN_SA: usize = 8;
pub const SN_SD: usize = 9;
pub const SN_SE: usize = 10;
pub const SN_SU: usize = 11;
pub const SN_FT: usize = 12;
pub const SN_FB: usize = 13;
pub const SN_FREE_ACTION: usize = 14;
pub const SN_SLAYING: usize = 15;
pub const SN_CLUMSINESS: usize = 16;
pub const SN_WEAKNESS: usize = 17;
pub const SN_SLOW_DESCENT: usize = 18;
pub const SN_SPEED: usize = 19;
pub const SN_STEALTH: usize = 20;
pub const SN_SLOWNESS: usize = 21;
pub const SN_NOISE: usize = 22;
pub const SN_GREAT_MASS: usize = 23;
pub const SN_INTELLIGENCE: usize = 24;
pub const SN_WISDOM: usize = 25;
pub const SN_INFRAVISION: usize = 26;
pub const SN_MIGHT: usize = 27;
pub const SN_LORDLINESS: usize = 28;
pub const SN_MAGI: usize = 29;
pub const SN_BEAUTY: usize = 30;
pub const SN_SEEING: usize = 31;
pub const SN_REGENERATION: usize = 32;
pub const SN_STUPIDITY: usize = 33;
pub const SN_DULLNESS: usize = 34;
pub const SN_BLINDNESS: usize = 35;
pub const SN_TIMIDNESS: usize = 36;
pub const SN_TELEPORTATION: usize = 37;
pub const SN_UGLINESS: usize = 38;
pub const SN_PROTECTION: usize = 39;
pub const SN_IRRITATION: usize = 40;
pub const SN_VULNERABILITY: usize = 41;
pub const SN_ENVELOPING: usize = 42;
pub const SN_FIRE: usize = 43;
pub const SN_SLAY_EVIL: usize = 44;
pub const SN_DRAGON_SLAYING: usize = 45;
pub const SN_EMPTY: usize = 46;
pub const SN_LOCKED_DESC: usize = 47;
pub const SN_POISON_NEEDLE: usize = 48;
pub const SN_GAS_TRAP: usize = 49;
pub const SN_EXPLOSION_DEVICE: usize = 50;
pub const SN_SUMMONING_RUNES: usize = 51;
pub const SN_MULTIPLE_TRAPS: usize = 52;
pub const SN_DISARMED: usize = 53;
pub const SN_UNLOCKED: usize = 54;
pub const SN_SLAY_ANIMAL: usize = 55;

pub fn set_special_name(item: &mut Item, sn_id: usize) {
    if sn_id < SPECIAL_ITEM_NAMES.len() && !SPECIAL_ITEM_NAMES[sn_id].is_empty() {
        item.ego_name = Some(SPECIAL_ITEM_NAMES[sn_id].to_string());
    } else {
        item.ego_name = None;
    }
}

// Ego and special item names from Umoria SpecialNameIds
pub const SPECIAL_ITEM_NAMES: [&str; 56] = [
    "",
    "(R)",
    "(RA)",
    "(RF)",
    "(RC)",
    "(RL)",
    "(HA)",
    "(DF)",
    "(SA)",
    "(SD)",
    "(SE)",
    "(SU)",
    "(FT)",
    "(FB)",
    "of Free Action",
    "of Slaying",
    "of Clumsiness",
    "of Weakness",
    "of Slow Descent",
    "of Speed",
    "of Stealth",
    "of Slowness",
    "of Noise",
    "of Great Mass",
    "of Intelligence",
    "of Wisdom",
    "of Infra-Vision",
    "of Might",
    "of Lordliness",
    "of the Magi",
    "of Beauty",
    "of Seeing",
    "of Regeneration",
    "of Stupidity",
    "of Dullness",
    "of Blindness",
    "of Timidness",
    "of Teleportation",
    "of Ugliness",
    "of Protection",
    "of Irritation",
    "of Vulnerability",
    "of Enveloping",
    "of Fire",
    "of Slay Evil",
    "of Dragon Slaying",
    "(Empty)",
    "(Locked)",
    "(Poison Needle)",
    "(Gas Trap)",
    "(Explosion Device)",
    "(Summoning Runes)",
    "(Multiple Traps)",
    "(Disarmed)",
    "(Unlocked)",
    "of Slay Animal",
];

// Authentic store choices matrix from
pub const STORE_CHOICES: [[u16; 26]; 6] = [
    // General Store
    [
        366, 365, 364, 84, 84, 365, 123, 366, 365, 350, 349, 348, 347, 346, 346, 345, 345, 345,
        344, 344, 344, 344, 344, 344, 344, 344,
    ],
    // Armory
    [
        94, 95, 96, 109, 103, 104, 105, 106, 110, 111, 112, 114, 116, 124, 125, 126, 127, 129, 103,
        104, 124, 125, 91, 92, 95, 96,
    ],
    // Weaponsmith
    [
        29, 30, 34, 37, 45, 49, 57, 58, 59, 65, 67, 68, 73, 74, 75, 77, 79, 80, 81, 83, 29, 30, 80,
        83, 80, 83,
    ],
    // Temple
    [
        322, 323, 324, 325, 180, 180, 233, 237, 240, 241, 361, 362, 57, 58, 59, 260, 358, 359, 265,
        237, 237, 240, 240, 241, 323, 359,
    ],
    // Alchemy shop
    [
        173, 174, 175, 351, 351, 352, 353, 354, 355, 356, 357, 206, 227, 230, 236, 252, 253, 352,
        353, 354, 355, 356, 359, 363, 359, 359,
    ],
    // Magic-User store
    [
        318, 141, 142, 153, 164, 167, 168, 140, 319, 320, 320, 321, 269, 270, 282, 286, 287, 292,
        293, 294, 295, 308, 269, 290, 319, 282,
    ],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreasureDefinition {
    pub name: &'static str,
    pub flags: u32,
    pub category_id: u8,
    pub sprite: char,
    pub misc_use: i16,
    pub cost: i32,
    pub sub_category_id: u8,
    pub items_count: u8,
    pub weight: u16,
    pub to_hit: i16,
    pub to_damage: i16,
    pub ac: i16,
    pub to_ac: i16,
    pub damage: Dice,
    pub depth_first_found: u8,
}

pub static GAME_OBJECTS: [TreasureDefinition; 420] = [
    TreasureDefinition {
        name: "Poison",
        flags: 0x0000_0001,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 64,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Blindness",
        flags: 0x0000_0002,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 65,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 9,
    },
    TreasureDefinition {
        name: "Paranoia",
        flags: 0x0000_0004,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 66,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 9,
    },
    TreasureDefinition {
        name: "Confusion",
        flags: 0x0000_0008,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 67,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Hallucination",
        flags: 0x0000_0010,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 68,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 13,
    },
    TreasureDefinition {
        name: "Cure Poison",
        flags: 0x0000_0020,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 60,
        sub_category_id: 69,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 8,
    },
    TreasureDefinition {
        name: "Cure Blindness",
        flags: 0x0000_0040,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 50,
        sub_category_id: 70,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Cure Paranoia",
        flags: 0x0000_0080,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 25,
        sub_category_id: 71,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Cure Confusion",
        flags: 0x0000_0100,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 50,
        sub_category_id: 72,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 6,
    },
    TreasureDefinition {
        name: "Weakness",
        flags: 0x0400_0200,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 0,
        sub_category_id: 73,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Unhealth",
        flags: 0x0400_0400,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 50,
        sub_category_id: 74,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 10, sides: 10 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Restore Constitution",
        flags: 0x0001_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 350,
        sub_category_id: 75,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "First-Aid",
        flags: 0x0020_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 5,
        sub_category_id: 76,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 6,
    },
    TreasureDefinition {
        name: "Minor Cures",
        flags: 0x0040_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 20,
        sub_category_id: 77,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Light Cures",
        flags: 0x0080_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 30,
        sub_category_id: 78,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Restoration",
        flags: 0x001F_8000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 1000,
        sub_category_id: 79,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Poison",
        flags: 0x0000_0001,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1200,
        cost: 0,
        sub_category_id: 80,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Hallucination",
        flags: 0x0000_0010,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1200,
        cost: 0,
        sub_category_id: 81,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 18,
    },
    TreasureDefinition {
        name: "Cure Poison",
        flags: 0x0000_0020,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1200,
        cost: 75,
        sub_category_id: 82,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 19,
    },
    TreasureDefinition {
        name: "Unhealth",
        flags: 0x0400_0400,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1200,
        cost: 75,
        sub_category_id: 83,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 10, sides: 12 },
        depth_first_found: 28,
    },
    TreasureDefinition {
        name: "Major Cures",
        flags: 0x0200_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1200,
        cost: 75,
        sub_category_id: 84,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 16,
    },
    TreasureDefinition {
        name: "& Ration~ of Food",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 5000,
        cost: 3,
        sub_category_id: 90,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Ration~ of Food",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 5000,
        cost: 3,
        sub_category_id: 90,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Ration~ of Food",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 5000,
        cost: 3,
        sub_category_id: 90,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "& Slime Mold~",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 3000,
        cost: 2,
        sub_category_id: 91,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Piece~ of Elvish Waybread",
        flags: 0x0200_0020,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 7500,
        cost: 25,
        sub_category_id: 92,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 6,
    },
    TreasureDefinition {
        name: "& Piece~ of Elvish Waybread",
        flags: 0x0200_0020,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 7500,
        cost: 25,
        sub_category_id: 92,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "& Piece~ of Elvish Waybread",
        flags: 0x0200_0020,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 7500,
        cost: 25,
        sub_category_id: 92,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Dagger (Main Gauche)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 25,
        sub_category_id: 1,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 5 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "& Dagger (Misericorde)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 10,
        sub_category_id: 2,
        items_count: 1,
        weight: 15,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Dagger (Stiletto)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 10,
        sub_category_id: 3,
        items_count: 1,
        weight: 12,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Dagger (Bodkin)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 10,
        sub_category_id: 4,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Broken Dagger",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 0,
        sub_category_id: 5,
        items_count: 1,
        weight: 15,
        to_hit: -2,
        to_damage: -2,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Backsword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 150,
        sub_category_id: 6,
        items_count: 1,
        weight: 95,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 9 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "& Bastard Sword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 350,
        sub_category_id: 7,
        items_count: 1,
        weight: 140,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 4 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "& Thrusting Sword (Bilbo)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 60,
        sub_category_id: 8,
        items_count: 1,
        weight: 80,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 4,
    },
    TreasureDefinition {
        name: "& Thrusting Sword (Baselard)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 80,
        sub_category_id: 9,
        items_count: 1,
        weight: 100,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 7 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Broadsword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 255,
        sub_category_id: 10,
        items_count: 1,
        weight: 150,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 5 },
        depth_first_found: 9,
    },
    TreasureDefinition {
        name: "& Two-Handed Sword (Claymore)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 775,
        sub_category_id: 11,
        items_count: 1,
        weight: 200,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 6 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "& Cutlass",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 85,
        sub_category_id: 12,
        items_count: 1,
        weight: 110,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 7 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "& Two-Handed Sword (Espadon)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 655,
        sub_category_id: 13,
        items_count: 1,
        weight: 180,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 6 },
        depth_first_found: 35,
    },
    TreasureDefinition {
        name: "& Executioner's Sword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 850,
        sub_category_id: 14,
        items_count: 1,
        weight: 260,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 5 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "& Two-Handed Sword (Flamberge)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 15,
        items_count: 1,
        weight: 240,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 5 },
        depth_first_found: 45,
    },
    TreasureDefinition {
        name: "& Foil",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 35,
        sub_category_id: 16,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 5 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "& Katana",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 400,
        sub_category_id: 17,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 4 },
        depth_first_found: 18,
    },
    TreasureDefinition {
        name: "& Longsword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 200,
        sub_category_id: 18,
        items_count: 1,
        weight: 130,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 10 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "& Two-Handed Sword (No-Dachi)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 675,
        sub_category_id: 19,
        items_count: 1,
        weight: 200,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 4 },
        depth_first_found: 45,
    },
    TreasureDefinition {
        name: "& Rapier",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 42,
        sub_category_id: 20,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 4,
    },
    TreasureDefinition {
        name: "& Sabre",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 50,
        sub_category_id: 21,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 7 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Small Sword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 48,
        sub_category_id: 22,
        items_count: 1,
        weight: 75,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Two-Handed Sword (Zweihander)",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 1500,
        sub_category_id: 23,
        items_count: 1,
        weight: 280,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 6 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "& Broken Sword",
        flags: 0x0000_0000,
        category_id: TV_SWORD,
        sprite: '|',
        misc_use: 0,
        cost: 0,
        sub_category_id: 24,
        items_count: 1,
        weight: 75,
        to_hit: -2,
        to_damage: -2,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Ball and Chain",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 200,
        sub_category_id: 1,
        items_count: 1,
        weight: 150,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Cat-o'-Nine-Tails",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 14,
        sub_category_id: 2,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "& Wooden Club",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 10,
        sub_category_id: 3,
        items_count: 1,
        weight: 100,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Flail",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 353,
        sub_category_id: 4,
        items_count: 1,
        weight: 150,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "& Two-Handed Great Flail",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 590,
        sub_category_id: 5,
        items_count: 1,
        weight: 280,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 6 },
        depth_first_found: 45,
    },
    TreasureDefinition {
        name: "& Morningstar",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 396,
        sub_category_id: 6,
        items_count: 1,
        weight: 150,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "& Mace",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 130,
        sub_category_id: 7,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 6,
    },
    TreasureDefinition {
        name: "& War Hammer",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 225,
        sub_category_id: 8,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 3 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Lead-Filled Mace",
        flags: 0x0000_0000,
        category_id: TV_HAFTED,
        sprite: '\\',
        misc_use: 0,
        cost: 502,
        sub_category_id: 9,
        items_count: 1,
        weight: 180,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 4 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Awl-Pike",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 200,
        sub_category_id: 1,
        items_count: 1,
        weight: 160,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 8 },
        depth_first_found: 8,
    },
    TreasureDefinition {
        name: "& Beaked Axe",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 408,
        sub_category_id: 2,
        items_count: 1,
        weight: 180,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Fauchard",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 326,
        sub_category_id: 3,
        items_count: 1,
        weight: 170,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 10 },
        depth_first_found: 17,
    },
    TreasureDefinition {
        name: "& Glaive",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 363,
        sub_category_id: 4,
        items_count: 1,
        weight: 190,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Halberd",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 430,
        sub_category_id: 5,
        items_count: 1,
        weight: 190,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 4 },
        depth_first_found: 22,
    },
    TreasureDefinition {
        name: "& Lucerne Hammer",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 376,
        sub_category_id: 6,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 5 },
        depth_first_found: 11,
    },
    TreasureDefinition {
        name: "& Pike",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 358,
        sub_category_id: 7,
        items_count: 1,
        weight: 160,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 5 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Spear",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 36,
        sub_category_id: 8,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "& Lance",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 230,
        sub_category_id: 9,
        items_count: 1,
        weight: 300,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 8 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "& Javelin",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 18,
        sub_category_id: 10,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 4,
    },
    TreasureDefinition {
        name: "& Battle Axe (Balestarius)",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 500,
        sub_category_id: 11,
        items_count: 1,
        weight: 180,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 8 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "& Battle Axe (European)",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 334,
        sub_category_id: 12,
        items_count: 1,
        weight: 170,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 3, sides: 4 },
        depth_first_found: 13,
    },
    TreasureDefinition {
        name: "& Broad Axe",
        flags: 0x0000_0000,
        category_id: TV_POLEARM,
        sprite: '/',
        misc_use: 0,
        cost: 304,
        sub_category_id: 13,
        items_count: 1,
        weight: 160,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 17,
    },
    TreasureDefinition {
        name: "& Short Bow",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 2,
        cost: 50,
        sub_category_id: 1,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "& Long Bow",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 3,
        cost: 120,
        sub_category_id: 2,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "& Composite Bow",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 4,
        cost: 240,
        sub_category_id: 3,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "& Light Crossbow",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 5,
        cost: 140,
        sub_category_id: 10,
        items_count: 1,
        weight: 110,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Heavy Crossbow",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 6,
        cost: 300,
        sub_category_id: 11,
        items_count: 1,
        weight: 200,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "& Sling",
        flags: 0x0000_0000,
        category_id: TV_BOW,
        sprite: '}',
        misc_use: 1,
        cost: 5,
        sub_category_id: 20,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Arrow~",
        flags: 0x0000_0000,
        category_id: TV_ARROW,
        sprite: '{',
        misc_use: 0,
        cost: 1,
        sub_category_id: 193,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "& Bolt~",
        flags: 0x0000_0000,
        category_id: TV_BOLT,
        sprite: '{',
        misc_use: 0,
        cost: 2,
        sub_category_id: 193,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 5 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "& Rounded Pebble~",
        flags: 0x0000_0000,
        category_id: TV_SLING_AMMO,
        sprite: '{',
        misc_use: 0,
        cost: 1,
        sub_category_id: 193,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Iron Shot~",
        flags: 0x0000_0000,
        category_id: TV_SLING_AMMO,
        sprite: '{',
        misc_use: 0,
        cost: 2,
        sub_category_id: 194,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "& Iron Spike~",
        flags: 0x0000_0000,
        category_id: TV_SPIKE,
        sprite: '~',
        misc_use: 0,
        cost: 1,
        sub_category_id: 193,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Brass Lantern~",
        flags: 0x0000_0000,
        category_id: TV_LIGHT,
        sprite: '~',
        misc_use: 7500,
        cost: 35,
        sub_category_id: 1,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Wooden Torch~",
        flags: 0x0000_0000,
        category_id: TV_LIGHT,
        sprite: '~',
        misc_use: 4000,
        cost: 2,
        sub_category_id: 193,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Orcish Pick",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 2,
        cost: 500,
        sub_category_id: 2,
        items_count: 1,
        weight: 180,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Dwarven Pick",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 3,
        cost: 1200,
        sub_category_id: 3,
        items_count: 1,
        weight: 200,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "& Gnomish Shovel",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 1,
        cost: 100,
        sub_category_id: 5,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Dwarven Shovel",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 2,
        cost: 250,
        sub_category_id: 6,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "& Pair of Soft Leather Shoes",
        flags: 0x0000_0000,
        category_id: TV_BOOTS,
        sprite: ']',
        misc_use: 0,
        cost: 4,
        sub_category_id: 1,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 1,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Pair of Soft Leather Boots",
        flags: 0x0000_0000,
        category_id: TV_BOOTS,
        sprite: ']',
        misc_use: 0,
        cost: 7,
        sub_category_id: 2,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 2,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 4,
    },
    TreasureDefinition {
        name: "& Pair of Hard Leather Boots",
        flags: 0x0000_0000,
        category_id: TV_BOOTS,
        sprite: ']',
        misc_use: 0,
        cost: 12,
        sub_category_id: 3,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 3,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 6,
    },
    TreasureDefinition {
        name: "& Soft Leather Cap",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 4,
        sub_category_id: 1,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 1,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "& Hard Leather Cap",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 12,
        sub_category_id: 2,
        items_count: 1,
        weight: 15,
        to_hit: 0,
        to_damage: 0,
        ac: 2,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 4,
    },
    TreasureDefinition {
        name: "& Metal Cap",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 30,
        sub_category_id: 3,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 3,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "& Iron Helm",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 75,
        sub_category_id: 4,
        items_count: 1,
        weight: 75,
        to_hit: 0,
        to_damage: 0,
        ac: 5,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Steel Helm",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 200,
        sub_category_id: 5,
        items_count: 1,
        weight: 60,
        to_hit: 0,
        to_damage: 0,
        ac: 6,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "& Silver Crown",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 500,
        sub_category_id: 6,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "& Golden Crown",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 7,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 47,
    },
    TreasureDefinition {
        name: "& Jewel-Encrusted Crown",
        flags: 0x0000_0000,
        category_id: TV_HELM,
        sprite: ']',
        misc_use: 0,
        cost: 2000,
        sub_category_id: 8,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "& Robe",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 4,
        sub_category_id: 1,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 2,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Soft Leather Armor",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 18,
        sub_category_id: 2,
        items_count: 1,
        weight: 80,
        to_hit: 0,
        to_damage: 0,
        ac: 4,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Soft Studded Leather",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 35,
        sub_category_id: 3,
        items_count: 1,
        weight: 90,
        to_hit: 0,
        to_damage: 0,
        ac: 5,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Hard Leather Armor",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 55,
        sub_category_id: 4,
        items_count: 1,
        weight: 100,
        to_hit: -1,
        to_damage: 0,
        ac: 6,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Hard Studded Leather",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 100,
        sub_category_id: 5,
        items_count: 1,
        weight: 110,
        to_hit: -1,
        to_damage: 0,
        ac: 7,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Woven Cord Armor",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 45,
        sub_category_id: 6,
        items_count: 1,
        weight: 150,
        to_hit: -1,
        to_damage: 0,
        ac: 6,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Soft Leather Ring Mail",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 160,
        sub_category_id: 7,
        items_count: 1,
        weight: 130,
        to_hit: -1,
        to_damage: 0,
        ac: 6,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Hard Leather Ring Mail",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 230,
        sub_category_id: 8,
        items_count: 1,
        weight: 150,
        to_hit: -2,
        to_damage: 0,
        ac: 8,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Leather Scale Mail",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '(',
        misc_use: 0,
        cost: 330,
        sub_category_id: 9,
        items_count: 1,
        weight: 140,
        to_hit: -1,
        to_damage: 0,
        ac: 11,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Metal Scale Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 430,
        sub_category_id: 1,
        items_count: 1,
        weight: 250,
        to_hit: -2,
        to_damage: 0,
        ac: 13,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 24,
    },
    TreasureDefinition {
        name: "Chain Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 530,
        sub_category_id: 2,
        items_count: 1,
        weight: 220,
        to_hit: -2,
        to_damage: 0,
        ac: 14,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 26,
    },
    TreasureDefinition {
        name: "Rusty Chain Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 0,
        sub_category_id: 3,
        items_count: 1,
        weight: 220,
        to_hit: -5,
        to_damage: 0,
        ac: 14,
        to_ac: -8,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 26,
    },
    TreasureDefinition {
        name: "Double Chain Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 630,
        sub_category_id: 4,
        items_count: 1,
        weight: 260,
        to_hit: -2,
        to_damage: 0,
        ac: 15,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 28,
    },
    TreasureDefinition {
        name: "Augmented Chain Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 675,
        sub_category_id: 5,
        items_count: 1,
        weight: 270,
        to_hit: -2,
        to_damage: 0,
        ac: 16,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Bar Chain Mail",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 720,
        sub_category_id: 6,
        items_count: 1,
        weight: 280,
        to_hit: -2,
        to_damage: 0,
        ac: 18,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 34,
    },
    TreasureDefinition {
        name: "Metal Brigandine Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 775,
        sub_category_id: 7,
        items_count: 1,
        weight: 290,
        to_hit: -3,
        to_damage: 0,
        ac: 19,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 36,
    },
    TreasureDefinition {
        name: "Laminated Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 825,
        sub_category_id: 8,
        items_count: 1,
        weight: 300,
        to_hit: -3,
        to_damage: 0,
        ac: 20,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 38,
    },
    TreasureDefinition {
        name: "Partial Plate Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 900,
        sub_category_id: 9,
        items_count: 1,
        weight: 320,
        to_hit: -3,
        to_damage: 0,
        ac: 22,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 42,
    },
    TreasureDefinition {
        name: "Metal Lamellar Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 950,
        sub_category_id: 10,
        items_count: 1,
        weight: 340,
        to_hit: -3,
        to_damage: 0,
        ac: 23,
        to_ac: 0,
        damage: Dice { num: 1, sides: 6 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Full Plate Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 1050,
        sub_category_id: 11,
        items_count: 1,
        weight: 380,
        to_hit: -3,
        to_damage: 0,
        ac: 25,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 48,
    },
    TreasureDefinition {
        name: "Ribbed Plate Armor",
        flags: 0x0000_0000,
        category_id: TV_HARD_ARMOR,
        sprite: '[',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 12,
        items_count: 1,
        weight: 380,
        to_hit: -3,
        to_damage: 0,
        ac: 28,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "& Cloak",
        flags: 0x0000_0000,
        category_id: TV_CLOAK,
        sprite: '(',
        misc_use: 0,
        cost: 3,
        sub_category_id: 1,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 1,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Set of Leather Gloves",
        flags: 0x0000_0000,
        category_id: TV_GLOVES,
        sprite: ']',
        misc_use: 0,
        cost: 3,
        sub_category_id: 1,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 1,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Set of Gauntlets",
        flags: 0x0000_0000,
        category_id: TV_GLOVES,
        sprite: ']',
        misc_use: 0,
        cost: 35,
        sub_category_id: 2,
        items_count: 1,
        weight: 25,
        to_hit: 0,
        to_damage: 0,
        ac: 2,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "& Small Leather Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 30,
        sub_category_id: 1,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 2,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "& Medium Leather Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 60,
        sub_category_id: 2,
        items_count: 1,
        weight: 75,
        to_hit: 0,
        to_damage: 0,
        ac: 3,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 8,
    },
    TreasureDefinition {
        name: "& Large Leather Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 120,
        sub_category_id: 3,
        items_count: 1,
        weight: 100,
        to_hit: 0,
        to_damage: 0,
        ac: 4,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Small Metal Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 50,
        sub_category_id: 4,
        items_count: 1,
        weight: 65,
        to_hit: 0,
        to_damage: 0,
        ac: 3,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "& Medium Metal Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 125,
        sub_category_id: 5,
        items_count: 1,
        weight: 90,
        to_hit: 0,
        to_damage: 0,
        ac: 4,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "& Large Metal Shield",
        flags: 0x0000_0000,
        category_id: TV_SHIELD,
        sprite: ')',
        misc_use: 0,
        cost: 200,
        sub_category_id: 6,
        items_count: 1,
        weight: 120,
        to_hit: 0,
        to_damage: 0,
        ac: 5,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Strength",
        flags: 0x0000_0001,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 400,
        sub_category_id: 0,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Dexterity",
        flags: 0x0000_0008,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 400,
        sub_category_id: 1,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Constitution",
        flags: 0x0000_0010,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 400,
        sub_category_id: 2,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Intelligence",
        flags: 0x0000_0002,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 400,
        sub_category_id: 3,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Speed",
        flags: 0x0000_1000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 3000,
        sub_category_id: 4,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Searching",
        flags: 0x0000_0040,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 250,
        sub_category_id: 5,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Teleportation",
        flags: 0x8000_0400,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 0,
        sub_category_id: 6,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Slow Digestion",
        flags: 0x0000_0080,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 200,
        sub_category_id: 7,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Resist Fire",
        flags: 0x0008_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 250,
        sub_category_id: 8,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Resist Cold",
        flags: 0x0020_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 250,
        sub_category_id: 9,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Feather Falling",
        flags: 0x0400_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 200,
        sub_category_id: 10,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Adornment",
        flags: 0x0000_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 20,
        sub_category_id: 11,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "& Arrow~",
        flags: 0x0000_0000,
        category_id: TV_ARROW,
        sprite: '{',
        misc_use: 0,
        cost: 1,
        sub_category_id: 193,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Weakness",
        flags: 0x8000_0001,
        category_id: TV_RING,
        sprite: '=',
        misc_use: -5,
        cost: 0,
        sub_category_id: 13,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Lordly Protection (FIRE)",
        flags: 0x0008_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 14,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 5,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Lordly Protection (ACID)",
        flags: 0x0010_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 15,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 5,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Lordly Protection (COLD)",
        flags: 0x0020_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 16,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 5,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "WOE",
        flags: 0x8000_0644,
        category_id: TV_RING,
        sprite: '=',
        misc_use: -5,
        cost: 0,
        sub_category_id: 17,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: -3,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Stupidity",
        flags: 0x8000_0002,
        category_id: TV_RING,
        sprite: '=',
        misc_use: -5,
        cost: 0,
        sub_category_id: 18,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Increase Damage",
        flags: 0x0000_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 100,
        sub_category_id: 19,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Increase To-Hit",
        flags: 0x0000_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 100,
        sub_category_id: 20,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Protection",
        flags: 0x0000_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 100,
        sub_category_id: 21,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Aggravate Monster",
        flags: 0x8000_0200,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 0,
        sub_category_id: 22,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "See Invisible",
        flags: 0x0100_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 500,
        sub_category_id: 23,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Sustain Strength",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 1,
        cost: 750,
        sub_category_id: 24,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Sustain Intelligence",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 2,
        cost: 600,
        sub_category_id: 25,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Sustain Wisdom",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 3,
        cost: 600,
        sub_category_id: 26,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Sustain Constitution",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 4,
        cost: 750,
        sub_category_id: 27,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Sustain Dexterity",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 5,
        cost: 750,
        sub_category_id: 28,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Sustain Charisma",
        flags: 0x0040_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 6,
        cost: 500,
        sub_category_id: 29,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 44,
    },
    TreasureDefinition {
        name: "Slaying",
        flags: 0x0000_0000,
        category_id: TV_RING,
        sprite: '=',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 30,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Wisdom",
        flags: 0x0000_0004,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 300,
        sub_category_id: 0,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Charisma",
        flags: 0x0000_0020,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 250,
        sub_category_id: 1,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Searching",
        flags: 0x0000_0040,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 250,
        sub_category_id: 2,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Teleportation",
        flags: 0x8000_0400,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 0,
        sub_category_id: 3,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Slow Digestion",
        flags: 0x0000_0080,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 200,
        sub_category_id: 4,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 14,
    },
    TreasureDefinition {
        name: "Resist Acid",
        flags: 0x0010_0000,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 250,
        sub_category_id: 5,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 24,
    },
    TreasureDefinition {
        name: "Adornment",
        flags: 0x0000_0000,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 20,
        sub_category_id: 6,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 16,
    },
    TreasureDefinition {
        name: "& Bolt~",
        flags: 0x0000_0000,
        category_id: TV_BOLT,
        sprite: '{',
        misc_use: 0,
        cost: 2,
        sub_category_id: 193,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 5 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "the Magi",
        flags: 0x0180_0040,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: 0,
        cost: 5000,
        sub_category_id: 8,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 3,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "DOOM",
        flags: 0x8000_007F,
        category_id: TV_AMULET,
        sprite: '"',
        misc_use: -5,
        cost: 0,
        sub_category_id: 9,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Enchant Weapon To-Hit",
        flags: 0x0000_0001,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 125,
        sub_category_id: 64,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Enchant Weapon To-Dam",
        flags: 0x0000_0002,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 125,
        sub_category_id: 65,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Enchant Armor",
        flags: 0x0000_0004,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 125,
        sub_category_id: 66,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Identify",
        flags: 0x0000_0008,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 67,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Identify",
        flags: 0x0000_0008,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 67,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Identify",
        flags: 0x0000_0008,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 67,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Identify",
        flags: 0x0000_0008,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 67,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Remove Curse",
        flags: 0x0000_0010,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 100,
        sub_category_id: 68,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0020,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 69,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0020,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 69,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0020,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 69,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Summon Monster",
        flags: 0x0000_0040,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 70,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Phase Door",
        flags: 0x0000_0080,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 71,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Teleport",
        flags: 0x0000_0100,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 40,
        sub_category_id: 72,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Teleport Level",
        flags: 0x0000_0200,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 73,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Monster Confusion",
        flags: 0x0000_0400,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 30,
        sub_category_id: 74,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Magic Mapping",
        flags: 0x0000_0800,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 40,
        sub_category_id: 75,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Sleep Monster",
        flags: 0x0000_1000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 76,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Rune of Protection",
        flags: 0x0000_2000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 500,
        sub_category_id: 77,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Treasure Detection",
        flags: 0x0000_4000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 78,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Object Detection",
        flags: 0x0000_8000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 79,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Trap Detection",
        flags: 0x0001_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 80,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Trap Detection",
        flags: 0x0001_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 80,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 8,
    },
    TreasureDefinition {
        name: "Trap Detection",
        flags: 0x0001_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 80,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Door/Stair Location",
        flags: 0x0002_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 81,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Door/Stair Location",
        flags: 0x0002_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 81,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Door/Stair Location",
        flags: 0x0002_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 35,
        sub_category_id: 81,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Mass Genocide",
        flags: 0x0004_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 82,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Detect Invisible",
        flags: 0x0008_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 83,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Aggravate Monster",
        flags: 0x0010_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 84,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Trap Creation",
        flags: 0x0020_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 85,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Trap/Door Destruction",
        flags: 0x0040_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 86,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Door Creation",
        flags: 0x0080_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 100,
        sub_category_id: 87,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Recharging",
        flags: 0x0100_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 200,
        sub_category_id: 88,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Genocide",
        flags: 0x0200_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 750,
        sub_category_id: 89,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 35,
    },
    TreasureDefinition {
        name: "Darkness",
        flags: 0x0400_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 90,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Protection from Evil",
        flags: 0x0800_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 100,
        sub_category_id: 91,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Create Food",
        flags: 0x1000_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 10,
        sub_category_id: 92,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Dispel Undead",
        flags: 0x2000_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 200,
        sub_category_id: 93,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "*Enchant Weapon*",
        flags: 0x0000_0001,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 500,
        sub_category_id: 94,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Curse Weapon",
        flags: 0x0000_0002,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 95,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "*Enchant Armor*",
        flags: 0x0000_0004,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 500,
        sub_category_id: 96,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Curse Armor",
        flags: 0x0000_0008,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 97,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Summon Undead",
        flags: 0x0000_0010,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 0,
        sub_category_id: 98,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Blessing",
        flags: 0x0000_0020,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 99,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Holy Chant",
        flags: 0x0000_0040,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 40,
        sub_category_id: 100,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Holy Prayer",
        flags: 0x0000_0080,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 80,
        sub_category_id: 101,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 24,
    },
    TreasureDefinition {
        name: "Word-of-Recall",
        flags: 0x0000_0100,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 150,
        sub_category_id: 102,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "*Destruction*",
        flags: 0x0000_0200,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 750,
        sub_category_id: 103,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Slime Mold Juice",
        flags: 0x3000_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 400,
        cost: 2,
        sub_category_id: 64,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Apple Juice",
        flags: 0x0000_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 250,
        cost: 1,
        sub_category_id: 65,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Water",
        flags: 0x0000_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 200,
        cost: 0,
        sub_category_id: 66,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Strength",
        flags: 0x0000_0001,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 300,
        sub_category_id: 67,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Weakness",
        flags: 0x0000_0002,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 68,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Restore Strength",
        flags: 0x0000_0004,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 69,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Intelligence",
        flags: 0x0000_0008,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 70,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Lose Intelligence",
        flags: 0x0000_0010,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 71,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Restore Intelligence",
        flags: 0x0000_0020,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 72,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Wisdom",
        flags: 0x0000_0040,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 73,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Lose Wisdom",
        flags: 0x0000_0080,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 74,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Restore Wisdom",
        flags: 0x0000_0100,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 75,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Charisma",
        flags: 0x0000_0200,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 76,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Ugliness",
        flags: 0x0000_0400,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 77,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Restore Charisma",
        flags: 0x0000_0800,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 78,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Cure Light Wounds",
        flags: 0x1000_1000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 15,
        sub_category_id: 79,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Cure Light Wounds",
        flags: 0x1000_1000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 15,
        sub_category_id: 79,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Cure Light Wounds",
        flags: 0x1000_1000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 15,
        sub_category_id: 79,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Cure Serious Wounds",
        flags: 0x3000_2000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 100,
        cost: 40,
        sub_category_id: 80,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Cure Critical Wounds",
        flags: 0x7000_4000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 100,
        cost: 100,
        sub_category_id: 81,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Healing",
        flags: 0x7000_8000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 200,
        cost: 200,
        sub_category_id: 82,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Constitution",
        flags: 0x0001_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 300,
        sub_category_id: 83,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Gain Experience",
        flags: 0x0002_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 2500,
        sub_category_id: 84,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Sleep",
        flags: 0x0004_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 100,
        cost: 0,
        sub_category_id: 85,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Blindness",
        flags: 0x0008_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 86,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Confusion",
        flags: 0x0010_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 0,
        sub_category_id: 87,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Poison",
        flags: 0x0020_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 88,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Haste Self",
        flags: 0x0040_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 75,
        sub_category_id: 89,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Slowness",
        flags: 0x0080_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 0,
        sub_category_id: 90,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Dexterity",
        flags: 0x0200_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 91,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Restore Dexterity",
        flags: 0x0400_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 92,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Restore Constitution",
        flags: 0x6800_0000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 0,
        cost: 300,
        sub_category_id: 93,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Lose Experience",
        flags: 0x0000_0002,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 95,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Salt Water",
        flags: 0x0000_0004,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 96,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Invulnerability",
        flags: 0x0000_0008,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 97,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Heroism",
        flags: 0x0000_0010,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 35,
        sub_category_id: 98,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Super Heroism",
        flags: 0x0000_0020,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 100,
        sub_category_id: 99,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Boldness",
        flags: 0x0000_0040,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 10,
        sub_category_id: 100,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Restore Life Levels",
        flags: 0x0000_0080,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 400,
        sub_category_id: 101,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Resist Heat",
        flags: 0x0000_0100,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 30,
        sub_category_id: 102,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Resist Cold",
        flags: 0x0000_0200,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 30,
        sub_category_id: 103,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Detect Invisible",
        flags: 0x0000_0400,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 50,
        sub_category_id: 104,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "Slow Poison",
        flags: 0x0000_0800,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 25,
        sub_category_id: 105,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Neutralize Poison",
        flags: 0x0000_1000,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 75,
        sub_category_id: 106,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Restore Mana",
        flags: 0x0000_2000,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 350,
        sub_category_id: 107,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Infra-Vision",
        flags: 0x0000_4000,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 20,
        sub_category_id: 108,
        items_count: 1,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 3,
    },
    TreasureDefinition {
        name: "& Flask~ of Oil",
        flags: 0x0004_0000,
        category_id: TV_FLASK,
        sprite: '!',
        misc_use: 7500,
        cost: 3,
        sub_category_id: 64,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0001,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 200,
        sub_category_id: 0,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Lightning Bolts",
        flags: 0x0000_0002,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 600,
        sub_category_id: 1,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Frost Bolts",
        flags: 0x0000_0004,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 800,
        sub_category_id: 2,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Fire Bolts",
        flags: 0x0000_0008,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 3,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 30,
    },
    TreasureDefinition {
        name: "Stone-to-Mud",
        flags: 0x0000_0010,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 300,
        sub_category_id: 4,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Polymorph",
        flags: 0x0000_0020,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 400,
        sub_category_id: 5,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Heal Monster",
        flags: 0x0000_0040,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 0,
        sub_category_id: 6,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Haste Monster",
        flags: 0x0000_0080,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 0,
        sub_category_id: 7,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Slow Monster",
        flags: 0x0000_0100,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 500,
        sub_category_id: 8,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Confuse Monster",
        flags: 0x0000_0200,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 400,
        sub_category_id: 9,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Sleep Monster",
        flags: 0x0000_0400,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 500,
        sub_category_id: 10,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "Drain Life",
        flags: 0x0000_0800,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 11,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Trap/Door Destruction",
        flags: 0x0000_1000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 500,
        sub_category_id: 12,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 12,
    },
    TreasureDefinition {
        name: "Magic Missile",
        flags: 0x0000_2000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 200,
        sub_category_id: 13,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Wall Building",
        flags: 0x0000_4000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 400,
        sub_category_id: 14,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Clone Monster",
        flags: 0x0000_8000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 0,
        sub_category_id: 15,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "Teleport Away",
        flags: 0x0001_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 350,
        sub_category_id: 16,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Disarming",
        flags: 0x0002_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 500,
        sub_category_id: 17,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Lightning Balls",
        flags: 0x0004_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 18,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 35,
    },
    TreasureDefinition {
        name: "Cold Balls",
        flags: 0x0008_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1500,
        sub_category_id: 19,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Fire Balls",
        flags: 0x0010_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1800,
        sub_category_id: 20,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Stinking Cloud",
        flags: 0x0020_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 400,
        sub_category_id: 21,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Acid Balls",
        flags: 0x0040_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 1650,
        sub_category_id: 22,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 48,
    },
    TreasureDefinition {
        name: "Wonder",
        flags: 0x0080_0000,
        category_id: TV_WAND,
        sprite: '-',
        misc_use: 0,
        cost: 250,
        sub_category_id: 23,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 2,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0001,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 250,
        sub_category_id: 0,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Door/Stair Location",
        flags: 0x0000_0002,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 350,
        sub_category_id: 1,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Trap Location",
        flags: 0x0000_0004,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 350,
        sub_category_id: 2,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Treasure Location",
        flags: 0x0000_0008,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 200,
        sub_category_id: 3,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Object Location",
        flags: 0x0000_0010,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 200,
        sub_category_id: 4,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Teleportation",
        flags: 0x0000_0020,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 800,
        sub_category_id: 5,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Earthquakes",
        flags: 0x0000_0040,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 350,
        sub_category_id: 6,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Summoning",
        flags: 0x0000_0080,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 7,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Summoning",
        flags: 0x0000_0080,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 7,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "*Destruction*",
        flags: 0x0000_0200,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 2500,
        sub_category_id: 8,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Starlight",
        flags: 0x0000_0400,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 400,
        sub_category_id: 9,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Haste Monsters",
        flags: 0x0000_0800,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 10,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Slow Monsters",
        flags: 0x0000_1000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 800,
        sub_category_id: 11,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Sleep Monsters",
        flags: 0x0000_2000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 700,
        sub_category_id: 12,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "Cure Light Wounds",
        flags: 0x0000_4000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 200,
        sub_category_id: 13,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Detect Invisible",
        flags: 0x0000_8000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 200,
        sub_category_id: 14,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "Speed",
        flags: 0x0001_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 15,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Slowness",
        flags: 0x0002_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 16,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "Mass Polymorph",
        flags: 0x0004_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 750,
        sub_category_id: 17,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 46,
    },
    TreasureDefinition {
        name: "Remove Curse",
        flags: 0x0008_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 500,
        sub_category_id: 18,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 47,
    },
    TreasureDefinition {
        name: "Detect Evil",
        flags: 0x0010_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 350,
        sub_category_id: 19,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 20,
    },
    TreasureDefinition {
        name: "Curing",
        flags: 0x0020_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 1000,
        sub_category_id: 20,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "Dispel Evil",
        flags: 0x0040_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 1200,
        sub_category_id: 21,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 49,
    },
    TreasureDefinition {
        name: "Darkness",
        flags: 0x0100_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 22,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "Darkness",
        flags: 0x0100_0000,
        category_id: TV_STAFF,
        sprite: '_',
        misc_use: 0,
        cost: 0,
        sub_category_id: 22,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 5,
    },
    TreasureDefinition {
        name: "[Beginners-Magick]",
        flags: 0x0000_007F,
        category_id: TV_MAGIC_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 25,
        sub_category_id: 64,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Magick I]",
        flags: 0x0000_FF80,
        category_id: TV_MAGIC_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 100,
        sub_category_id: 65,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Magick II]",
        flags: 0x00FF_0000,
        category_id: TV_MAGIC_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 400,
        sub_category_id: 66,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[The Mages' Guide to Power]",
        flags: 0x7F00_0000,
        category_id: TV_MAGIC_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 800,
        sub_category_id: 67,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Beginners Handbook]",
        flags: 0x0000_00FF,
        category_id: TV_PRAYER_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 25,
        sub_category_id: 64,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Words of Wisdom]",
        flags: 0x0000_FF00,
        category_id: TV_PRAYER_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 100,
        sub_category_id: 65,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Chants and Blessings]",
        flags: 0x01FF_0000,
        category_id: TV_PRAYER_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 400,
        sub_category_id: 66,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "[Exorcisms and Dispellings]",
        flags: 0x7E00_0000,
        category_id: TV_PRAYER_BOOK,
        sprite: '?',
        misc_use: 0,
        cost: 800,
        sub_category_id: 67,
        items_count: 1,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 40,
    },
    TreasureDefinition {
        name: "& Small Wooden Chest",
        flags: 0x1380_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 20,
        sub_category_id: 1,
        items_count: 1,
        weight: 250,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 3 },
        depth_first_found: 7,
    },
    TreasureDefinition {
        name: "& Large Wooden Chest",
        flags: 0x1780_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 60,
        sub_category_id: 4,
        items_count: 1,
        weight: 500,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 5 },
        depth_first_found: 15,
    },
    TreasureDefinition {
        name: "& Small Iron Chest",
        flags: 0x1780_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 100,
        sub_category_id: 7,
        items_count: 1,
        weight: 500,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 25,
    },
    TreasureDefinition {
        name: "& Large Iron Chest",
        flags: 0x2380_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 150,
        sub_category_id: 10,
        items_count: 1,
        weight: 1000,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 35,
    },
    TreasureDefinition {
        name: "& Small Steel Chest",
        flags: 0x1B80_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 200,
        sub_category_id: 13,
        items_count: 1,
        weight: 500,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 4 },
        depth_first_found: 45,
    },
    TreasureDefinition {
        name: "& Large Steel Chest",
        flags: 0x3380_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 250,
        sub_category_id: 16,
        items_count: 1,
        weight: 1000,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "& Rat Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Giant Centipede Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 2,
        items_count: 1,
        weight: 25,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "some Filthy Rags",
        flags: 0x0000_0000,
        category_id: TV_SOFT_ARMOR,
        sprite: '~',
        misc_use: 0,
        cost: 0,
        sub_category_id: 63,
        items_count: 1,
        weight: 20,
        to_hit: 0,
        to_damage: 0,
        ac: 1,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& empty bottle",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: '!',
        misc_use: 0,
        cost: 0,
        sub_category_id: 4,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "some shards of pottery",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: '~',
        misc_use: 0,
        cost: 0,
        sub_category_id: 5,
        items_count: 1,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Human Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 7,
        items_count: 1,
        weight: 60,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Dwarf Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 8,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Elf Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 9,
        items_count: 1,
        weight: 40,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Gnome Skeleton",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 10,
        items_count: 1,
        weight: 25,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& broken set of teeth",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 11,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& large broken bone",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: 's',
        misc_use: 0,
        cost: 0,
        sub_category_id: 12,
        items_count: 1,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& broken stick",
        flags: 0x0000_0000,
        category_id: TV_MISC,
        sprite: '~',
        misc_use: 0,
        cost: 0,
        sub_category_id: 13,
        items_count: 1,
        weight: 3,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Ration~ of Food",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 5000,
        cost: 3,
        sub_category_id: 90,
        items_count: 5,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Hard Biscuit~",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 1,
        sub_category_id: 93,
        items_count: 5,
        weight: 2,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Strip~ of Beef Jerky",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1750,
        cost: 2,
        sub_category_id: 94,
        items_count: 5,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Pint~ of Fine Ale",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 500,
        cost: 1,
        sub_category_id: 95,
        items_count: 3,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Pint~ of Fine Wine",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 400,
        cost: 2,
        sub_category_id: 96,
        items_count: 1,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Pick",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 1,
        cost: 50,
        sub_category_id: 1,
        items_count: 1,
        weight: 150,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 3 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Shovel",
        flags: 0x2000_0000,
        category_id: TV_DIGGING,
        sprite: '\\',
        misc_use: 0,
        cost: 15,
        sub_category_id: 4,
        items_count: 1,
        weight: 60,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 2 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Identify",
        flags: 0x0000_0008,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 50,
        sub_category_id: 67,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Light",
        flags: 0x0000_0020,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 69,
        items_count: 3,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Phase Door",
        flags: 0x0000_0080,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 71,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Magic Mapping",
        flags: 0x0000_0800,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 40,
        sub_category_id: 75,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Treasure Detection",
        flags: 0x0000_4000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 78,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Object Detection",
        flags: 0x0000_8000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 79,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Detect Invisible",
        flags: 0x0008_0000,
        category_id: TV_SCROLL1,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 83,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Blessing",
        flags: 0x0000_0020,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 15,
        sub_category_id: 99,
        items_count: 2,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Word-of-Recall",
        flags: 0x0000_0100,
        category_id: TV_SCROLL2,
        sprite: '?',
        misc_use: 0,
        cost: 150,
        sub_category_id: 102,
        items_count: 3,
        weight: 5,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Cure Light Wounds",
        flags: 0x1000_1000,
        category_id: TV_POTION1,
        sprite: '!',
        misc_use: 50,
        cost: 15,
        sub_category_id: 79,
        items_count: 2,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Heroism",
        flags: 0x0000_0010,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 35,
        sub_category_id: 98,
        items_count: 2,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Boldness",
        flags: 0x0000_0040,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 10,
        sub_category_id: 100,
        items_count: 2,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Slow Poison",
        flags: 0x0000_0800,
        category_id: TV_POTION2,
        sprite: '!',
        misc_use: 0,
        cost: 25,
        sub_category_id: 105,
        items_count: 2,
        weight: 4,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Brass Lantern~",
        flags: 0x0000_0000,
        category_id: TV_LIGHT,
        sprite: '~',
        misc_use: 7500,
        cost: 35,
        sub_category_id: 0,
        items_count: 1,
        weight: 50,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Wooden Torch~",
        flags: 0x0000_0000,
        category_id: TV_LIGHT,
        sprite: '~',
        misc_use: 4000,
        cost: 2,
        sub_category_id: 192,
        items_count: 5,
        weight: 30,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& Flask~ of Oil",
        flags: 0x0004_0000,
        category_id: TV_FLASK,
        sprite: '!',
        misc_use: 7500,
        cost: 3,
        sub_category_id: 64,
        items_count: 5,
        weight: 10,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "& open door",
        flags: 0x0000_0000,
        category_id: TV_OPEN_DOOR,
        sprite: '\'',
        misc_use: 0,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& closed door",
        flags: 0x0000_0000,
        category_id: TV_CLOSED_DOOR,
        sprite: '+',
        misc_use: 0,
        cost: 0,
        sub_category_id: 19,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& secret door",
        flags: 0x0000_0000,
        category_id: TV_SECRET_DOOR,
        sprite: '#',
        misc_use: 0,
        cost: 0,
        sub_category_id: 19,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "an up staircase",
        flags: 0x0000_0000,
        category_id: TV_UP_STAIR,
        sprite: '<',
        misc_use: 0,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "a down staircase",
        flags: 0x0000_0000,
        category_id: TV_DOWN_STAIR,
        sprite: '>',
        misc_use: 0,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "General Store",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '1',
        misc_use: 0,
        cost: 0,
        sub_category_id: 101,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Armory",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '2',
        misc_use: 0,
        cost: 0,
        sub_category_id: 102,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Weapon Smiths",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '3',
        misc_use: 0,
        cost: 0,
        sub_category_id: 103,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Temple",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '4',
        misc_use: 0,
        cost: 0,
        sub_category_id: 104,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Alchemy Shop",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '5',
        misc_use: 0,
        cost: 0,
        sub_category_id: 105,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "Magic Shop",
        flags: 0x0000_0000,
        category_id: TV_STORE_DOOR,
        sprite: '6',
        misc_use: 0,
        cost: 0,
        sub_category_id: 106,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "an open pit",
        flags: 0x0000_0000,
        category_id: TV_VIS_TRAP,
        sprite: ' ',
        misc_use: 1,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 50,
    },
    TreasureDefinition {
        name: "an arrow trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 3,
        cost: 0,
        sub_category_id: 2,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 8 },
        depth_first_found: 90,
    },
    TreasureDefinition {
        name: "a covered pit",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 2,
        cost: 0,
        sub_category_id: 3,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 60,
    },
    TreasureDefinition {
        name: "a trap door",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 4,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 8 },
        depth_first_found: 75,
    },
    TreasureDefinition {
        name: "a gas trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 3,
        cost: 0,
        sub_category_id: 5,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 95,
    },
    TreasureDefinition {
        name: "a loose rock",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: ';',
        misc_use: 0,
        cost: 0,
        sub_category_id: 6,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "a dart trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 7,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "a strange rune",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 8,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 90,
    },
    TreasureDefinition {
        name: "some loose rock",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 9,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 90,
    },
    TreasureDefinition {
        name: "a gas trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 10,
        cost: 0,
        sub_category_id: 10,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 105,
    },
    TreasureDefinition {
        name: "a strange rune",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 11,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 90,
    },
    TreasureDefinition {
        name: "a blackened spot",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 10,
        cost: 0,
        sub_category_id: 12,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 6 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "some corroded rock",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 10,
        cost: 0,
        sub_category_id: 13,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 4, sides: 6 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "a gas trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 14,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 2, sides: 6 },
        depth_first_found: 105,
    },
    TreasureDefinition {
        name: "a gas trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 15,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 4 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "a gas trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 16,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 8 },
        depth_first_found: 105,
    },
    TreasureDefinition {
        name: "a dart trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 17,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 8 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "a dart trap",
        flags: 0x0000_0000,
        category_id: TV_INVIS_TRAP,
        sprite: '^',
        misc_use: 5,
        cost: 0,
        sub_category_id: 18,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 8 },
        depth_first_found: 110,
    },
    TreasureDefinition {
        name: "some rubble",
        flags: 0x0000_0000,
        category_id: TV_RUBBLE,
        sprite: ':',
        misc_use: 0,
        cost: 0,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& Pint~ of Fine Grade Mush",
        flags: 0x0000_0000,
        category_id: TV_FOOD,
        sprite: ',',
        misc_use: 1500,
        cost: 1,
        sub_category_id: 97,
        items_count: 1,
        weight: 1,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 1, sides: 1 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "a strange rune",
        flags: 0x0000_0000,
        category_id: TV_VIS_TRAP,
        sprite: '^',
        misc_use: 0,
        cost: 0,
        sub_category_id: 99,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 10,
    },
    TreasureDefinition {
        name: "copper",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 3,
        sub_category_id: 1,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "copper",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 4,
        sub_category_id: 2,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "copper",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 5,
        sub_category_id: 3,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "silver",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 6,
        sub_category_id: 4,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "silver",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 7,
        sub_category_id: 5,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "silver",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 8,
        sub_category_id: 6,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "garnets",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 9,
        sub_category_id: 7,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "garnets",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 10,
        sub_category_id: 8,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "gold",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 12,
        sub_category_id: 9,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "gold",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 14,
        sub_category_id: 10,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "gold",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 16,
        sub_category_id: 11,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "opals",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 18,
        sub_category_id: 12,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "sapphires",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 20,
        sub_category_id: 13,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "gold",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 24,
        sub_category_id: 14,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "rubies",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 28,
        sub_category_id: 15,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "diamonds",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 32,
        sub_category_id: 16,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "emeralds",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '*',
        misc_use: 0,
        cost: 40,
        sub_category_id: 17,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "mithril",
        flags: 0x0000_0000,
        category_id: TV_GOLD,
        sprite: '$',
        misc_use: 0,
        cost: 80,
        sub_category_id: 18,
        items_count: 1,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 1,
    },
    TreasureDefinition {
        name: "nothing",
        flags: 0x0000_0000,
        category_id: TV_NOTHING,
        sprite: ' ',
        misc_use: 0,
        cost: 0,
        sub_category_id: 64,
        items_count: 0,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "& ruined chest",
        flags: 0x0000_0000,
        category_id: TV_CHEST,
        sprite: '&',
        misc_use: 0,
        cost: 0,
        sub_category_id: 0,
        items_count: 1,
        weight: 250,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
    TreasureDefinition {
        name: "",
        flags: 0x0000_0000,
        category_id: TV_NOTHING,
        sprite: ' ',
        misc_use: 0,
        cost: 0,
        sub_category_id: 0,
        items_count: 0,
        weight: 0,
        to_hit: 0,
        to_damage: 0,
        ac: 0,
        to_ac: 0,
        damage: Dice { num: 0, sides: 0 },
        depth_first_found: 0,
    },
];

pub fn clean_item_name(raw: &str, cat: u8, count: u32) -> String {
    let mut name = raw.trim();
    if let Some(stripped) = name.strip_prefix("& ") {
        name = stripped;
    }
    let s = if name.contains('~') {
        name.replace('~', if count > 1 { "s" } else { "" })
    } else {
        name.to_string()
    };

    match cat {
        TV_RING if !s.to_lowercase().starts_with("ring") => format!("Ring of {}", s),
        TV_AMULET if !s.to_lowercase().starts_with("amulet") => format!("Amulet of {}", s),
        TV_SCROLL1 | TV_SCROLL2 if !s.to_lowercase().starts_with("scroll") => {
            format!("Scroll of {}", s)
        }
        TV_POTION1 | TV_POTION2 if !s.to_lowercase().starts_with("potion") => {
            format!("Potion of {}", s)
        }
        TV_MAGIC_BOOK if !s.to_lowercase().starts_with("mage") => format!("Mage Spellbook {}", s),
        TV_PRAYER_BOOK if !s.to_lowercase().starts_with("priest") => {
            format!("Priest Prayerbook {}", s)
        }
        TV_WAND if !s.to_lowercase().starts_with("wand") => format!("Wand of {}", s),
        TV_STAFF if !s.to_lowercase().starts_with("staff") => format!("Staff of {}", s),
        _ => s,
    }
}

pub fn create_item_from_index(index: usize, count: u32) -> Item {
    let def = &GAME_OBJECTS[index.min(GAME_OBJECTS.len() - 1)];
    let name = clean_item_name(def.name, def.category_id, count);

    let item_type = match def.category_id {
        TV_SWORD | TV_HAFTED | TV_POLEARM | TV_DIGGING => ItemType::Weapon { damage: def.damage },
        TV_BOW => ItemType::Bow {
            multiplier: (def.misc_use.max(1) as u32),
        },
        TV_ARROW | TV_BOLT | TV_SLING_AMMO => ItemType::Missile { damage: def.damage },
        TV_SPIKE => ItemType::Missile {
            damage: Dice::new(1, 1),
        },
        TV_BOOTS | TV_GLOVES | TV_CLOAK | TV_HELM | TV_SHIELD | TV_HARD_ARMOR | TV_SOFT_ARMOR => {
            ItemType::Armor { ac: def.ac as i32 }
        }
        TV_RING => ItemType::Ring {
            bonus: def.misc_use as i32,
        },
        TV_AMULET => ItemType::Amulet {
            bonus: def.misc_use as i32,
        },
        TV_LIGHT => ItemType::Light {
            fuel: def.misc_use as i32,
        },
        TV_FOOD => ItemType::Food {
            nutrition: def.misc_use as i32,
        },
        TV_FLASK => ItemType::Potion { heal_amount: 0 },
        TV_POTION1 | TV_POTION2 => {
            let n_lower = def.name.to_lowercase();
            let heal_amount = if n_lower.contains("critical") {
                40
            } else if n_lower.contains("serious") {
                20
            } else if n_lower.contains("light") {
                10
            } else if n_lower.contains("healing") {
                50
            } else {
                0
            };
            ItemType::Potion { heal_amount }
        }
        TV_SCROLL1 | TV_SCROLL2 => {
            let n_lower = def.name.to_lowercase();
            let teleport = n_lower.contains("phase door") || n_lower.contains("teleport");
            ItemType::Scroll { teleport }
        }
        TV_MAGIC_BOOK => ItemType::MagicBook {
            spell_flags: def.flags,
        },
        TV_PRAYER_BOOK => ItemType::PrayerBook {
            spell_flags: def.flags,
        },
        TV_WAND => ItemType::Wand {
            charges: def.misc_use.max(1) as u32,
            spell_index: def.sub_category_id as usize,
        },
        TV_STAFF => ItemType::Staff {
            charges: def.misc_use.max(1) as u32,
            prayer_index: def.sub_category_id as usize,
        },
        _ => ItemType::Food { nutrition: 0 },
    };

    Item {
        name,
        count,
        weight: def.weight as u32,
        item_type,
        inscription: None,
        identified: true,
        flavor: None,
        equipped_slot: None,
        is_cursed: (def.flags & TR_CURSED) != 0,
        to_hit: def.to_hit,
        to_damage: def.to_damage,
        to_ac: def.to_ac,
        flags: def.flags,
        cost: def.cost.max(0) as u32,
        ego_name: None,
        category_id: def.category_id,
        sub_category_id: def.sub_category_id as u16,
        misc_use: def.misc_use as i32,
    }
}

pub const MAX_DUNGEON_OBJECTS: usize = 344;
pub const TREASURE_MAX_LEVELS: usize = 50;

static MISSILES_COUNTER: std::sync::atomic::AtomicI16 = std::sync::atomic::AtomicI16::new(0);

pub fn item_bigger_than_chest(obj: &TreasureDefinition) -> bool {
    match obj.category_id {
        TV_CHEST | TV_BOW | TV_POLEARM | TV_HARD_ARMOR | TV_SOFT_ARMOR | TV_STAFF => true,
        TV_HAFTED | TV_SWORD | TV_DIGGING => obj.weight > 150,
        _ => false,
    }
}

pub struct TreasureTables {
    pub sorted_objects: [usize; MAX_DUNGEON_OBJECTS],
    pub treasure_levels: [usize; TREASURE_MAX_LEVELS + 1],
}

impl Default for TreasureTables {
    fn default() -> Self {
        Self::new()
    }
}

impl TreasureTables {
    pub fn new() -> Self {
        let mut treasure_levels = [0usize; TREASURE_MAX_LEVELS + 1];
        for obj in &GAME_OBJECTS[..MAX_DUNGEON_OBJECTS] {
            let lvl = (obj.depth_first_found as usize).min(TREASURE_MAX_LEVELS);
            treasure_levels[lvl] += 1;
        }
        for i in 1..=TREASURE_MAX_LEVELS {
            treasure_levels[i] += treasure_levels[i - 1];
        }

        let mut indexes = [1usize; TREASURE_MAX_LEVELS + 1];
        let mut sorted_objects = [0usize; MAX_DUNGEON_OBJECTS];

        for (i, obj) in GAME_OBJECTS[..MAX_DUNGEON_OBJECTS].iter().enumerate() {
            let level = (obj.depth_first_found as usize).min(TREASURE_MAX_LEVELS);
            let object_id = treasure_levels[level] - indexes[level];
            sorted_objects[object_id] = i;
            indexes[level] += 1;
        }

        Self {
            sorted_objects,
            treasure_levels,
        }
    }

    pub fn get_random_object_id<R: rand::Rng>(
        &self,
        mut level: usize,
        must_be_small: bool,
        rng: &mut R,
    ) -> usize {
        if level == 0 {
            let max_idx = self.treasure_levels[0].max(1);
            let sorted_idx = rng.gen_range(0..max_idx);
            return self.sorted_objects[sorted_idx];
        }

        if level >= TREASURE_MAX_LEVELS {
            level = TREASURE_MAX_LEVELS;
        } else if rng.gen_range(1..=12) == 1 {
            let denom = rng.gen_range(1..=TREASURE_MAX_LEVELS);
            level = (level * TREASURE_MAX_LEVELS / denom) + 1;
            if level > TREASURE_MAX_LEVELS {
                level = TREASURE_MAX_LEVELS;
            }
        }

        let max_idx = self.treasure_levels[level].max(1);
        let mut object_id;

        loop {
            if rng.gen_range(1..=2) == 1 {
                object_id = rng.gen_range(0..max_idx);
            } else {
                let mut choice = rng.gen_range(0..max_idx);
                let j = rng.gen_range(0..max_idx);
                if choice < j {
                    choice = j;
                }
                let k = rng.gen_range(0..max_idx);
                if choice < k {
                    choice = k;
                }

                let found_level = (GAME_OBJECTS[self.sorted_objects[choice]].depth_first_found
                    as usize)
                    .min(TREASURE_MAX_LEVELS);
                if found_level == 0 {
                    let zero_max = self.treasure_levels[0].max(1);
                    object_id = rng.gen_range(0..zero_max);
                } else {
                    let span =
                        self.treasure_levels[found_level] - self.treasure_levels[found_level - 1];
                    let offset = if span > 0 { rng.gen_range(0..span) } else { 0 };
                    object_id = self.treasure_levels[found_level - 1] + offset;
                }
            }

            let candidate_game_obj_id = self.sorted_objects[object_id.min(MAX_DUNGEON_OBJECTS - 1)];
            if !(must_be_small && item_bigger_than_chest(&GAME_OBJECTS[candidate_game_obj_id])) {
                return candidate_game_obj_id;
            }
        }
    }
}

pub static TREASURE_TABLES: std::sync::LazyLock<TreasureTables> =
    std::sync::LazyLock::new(TreasureTables::new);

pub fn magic_enchantment_bonus<R: rand::Rng>(
    base: i32,
    max_standard: i32,
    level: i32,
    rng: &mut R,
) -> i32 {
    let mut stand_deviation = (125 * level / 100) + 7;
    if stand_deviation > max_standard || level > max_standard {
        stand_deviation = max_standard;
    }
    let tmp = (0..8).map(|_| rng.gen_range(1..=99)).sum::<i32>();
    let diff = (tmp - 400) * stand_deviation / 81;
    let abs_dist = diff.abs();
    let bonus = (abs_dist / 10) + base;
    bonus.max(base)
}

fn magic_should_be_enchanted<R: rand::Rng>(chance: i32, rng: &mut R) -> bool {
    rng.gen_range(1..=100) <= chance
}

pub fn magical_armor<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_ac += magic_enchantment_bonus(1, 30, level, rng) as i16;
    if !magic_should_be_enchanted(special, rng) {
        return;
    }

    match rng.gen_range(1..=9) {
        1 => {
            item.flags |= TR_RES_LIGHT | TR_RES_COLD | TR_RES_ACID | TR_RES_FIRE;
            set_special_name(item, SN_R);
            item.to_ac += 5;
            item.cost += 2500;
        }
        2 => {
            item.flags |= TR_RES_ACID;
            set_special_name(item, SN_RA);
            item.cost += 1000;
        }
        3 | 4 => {
            item.flags |= TR_RES_FIRE;
            set_special_name(item, SN_RF);
            item.cost += 600;
        }
        5 | 6 => {
            item.flags |= TR_RES_COLD;
            set_special_name(item, SN_RC);
            item.cost += 600;
        }
        _ => {
            item.flags |= TR_RES_LIGHT;
            set_special_name(item, SN_RL);
            item.cost += 500;
        }
    }
}

pub fn cursed_armor<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.to_ac -= magic_enchantment_bonus(1, 40, level, rng) as i16;
    item.cost = 0;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
}

pub fn magical_sword<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_hit += magic_enchantment_bonus(0, 40, level, rng) as i16;
    let damage_bonus = match &item.item_type {
        ItemType::Weapon { damage } => (damage.num * damage.sides) as i32,
        _ => 10,
    };
    item.to_damage +=
        magic_enchantment_bonus(0, 4 * damage_bonus, damage_bonus * level / 10, rng) as i16;

    if magic_should_be_enchanted(3 * special / 2, rng) {
        match rng.gen_range(1..=16) {
            1 => {
                // Holy Avenger
                item.flags |= TR_SEE_INVIS | TR_SUST_STAT | TR_SLAY_UNDEAD | TR_SLAY_EVIL | TR_STR;
                item.to_hit += 5;
                item.to_damage += 5;
                item.to_ac += rng.gen_range(1..=4) as i16;
                item.misc_use = rng.gen_range(1..=4);
                set_special_name(item, SN_HA);
                item.cost += (item.misc_use as u32 * 500) + 10000;
            }
            2 => {
                // Defender
                item.flags |= TR_FFALL
                    | TR_RES_LIGHT
                    | TR_SEE_INVIS
                    | TR_FREE_ACT
                    | TR_RES_COLD
                    | TR_RES_ACID
                    | TR_RES_FIRE
                    | TR_REGEN
                    | TR_STEALTH;
                item.to_hit += 3;
                item.to_damage += 3;
                item.to_ac += 5 + rng.gen_range(1..=5) as i16;
                item.misc_use = rng.gen_range(1..=3);
                set_special_name(item, SN_DF);
                item.cost += (item.misc_use as u32 * 500) + 7500;
            }
            3 | 4 => {
                item.flags |= TR_SLAY_ANIMAL;
                item.to_hit += 2;
                item.to_damage += 2;
                set_special_name(item, SN_SA);
                item.cost += 3000;
            }
            5 | 6 => {
                item.flags |= TR_SLAY_DRAGON;
                item.to_hit += 3;
                item.to_damage += 3;
                set_special_name(item, SN_SD);
                item.cost += 4000;
            }
            7 | 8 => {
                item.flags |= TR_SLAY_EVIL;
                item.to_hit += 3;
                item.to_damage += 3;
                set_special_name(item, SN_SE);
                item.cost += 4000;
            }
            9 | 10 => {
                item.flags |= TR_SEE_INVIS | TR_SLAY_UNDEAD;
                item.to_hit += 3;
                item.to_damage += 3;
                set_special_name(item, SN_SU);
                item.cost += 5000;
            }
            11..=13 => {
                item.flags |= TR_FLAME_TONGUE;
                item.to_hit += 1;
                item.to_damage += 3;
                set_special_name(item, SN_FT);
                item.cost += 2000;
            }
            _ => {
                item.flags |= TR_FROST_BRAND;
                item.to_hit += 1;
                item.to_damage += 1;
                set_special_name(item, SN_FB);
                item.cost += 1200;
            }
        }
    }
}

pub fn cursed_sword<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.to_hit -= magic_enchantment_bonus(1, 55, level, rng) as i16;
    let damage_bonus = match &item.item_type {
        ItemType::Weapon { damage } => (damage.num * damage.sides) as i32,
        _ => 10,
    };
    item.to_damage -=
        magic_enchantment_bonus(1, 11 * damage_bonus / 2, damage_bonus * level / 10, rng) as i16;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
    item.cost = 0;
}

pub fn magical_bow<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.to_hit += magic_enchantment_bonus(1, 30, level, rng) as i16;
    item.to_damage += magic_enchantment_bonus(1, 20, level, rng) as i16;
}

pub fn cursed_bow<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.to_hit -= magic_enchantment_bonus(1, 50, level, rng) as i16;
    item.to_damage -= magic_enchantment_bonus(1, 30, level, rng) as i16;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
    item.cost = 0;
}

pub fn magical_digging_tool<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.misc_use += magic_enchantment_bonus(0, 25, level, rng);
}

pub fn cursed_digging_tool<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    item.misc_use = -magic_enchantment_bonus(1, 30, level, rng);
    item.cost = 0;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
}

pub fn magical_gloves<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
    if !magic_should_be_enchanted(special, rng) {
        return;
    }

    if rng.gen_range(1..=2) == 1 {
        item.flags |= TR_FREE_ACT;
        set_special_name(item, SN_FREE_ACTION);
        item.cost += 1000;
    } else {
        item.to_hit += 1 + rng.gen_range(1..=3) as i16;
        item.to_damage += 1 + rng.gen_range(1..=3) as i16;
        set_special_name(item, SN_SLAYING);
        item.cost += (item.to_hit.max(0) as u32 + item.to_damage.max(0) as u32) * 250;
    }
}

pub fn cursed_gloves<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    if magic_should_be_enchanted(special, rng) {
        if rng.gen_range(1..=2) == 1 {
            item.flags |= TR_DEX;
            set_special_name(item, SN_CLUMSINESS);
        } else {
            item.flags |= TR_STR;
            set_special_name(item, SN_WEAKNESS);
        }
        item.misc_use = -magic_enchantment_bonus(1, 10, level, rng);
    }
    item.to_ac -= magic_enchantment_bonus(1, 40, level, rng) as i16;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
    item.cost = 0;
}

pub fn magical_boots<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
    if !magic_should_be_enchanted(special, rng) {
        return;
    }

    match rng.gen_range(1..=12) {
        1 => {
            item.flags |= TR_SPEED;
            set_special_name(item, SN_SPEED);
            item.misc_use = 1;
            item.cost += 5000;
        }
        2..=5 => {
            item.flags |= TR_STEALTH;
            item.misc_use = rng.gen_range(1..=3);
            set_special_name(item, SN_STEALTH);
            item.cost += 500;
        }
        _ => {
            item.flags |= TR_FFALL;
            set_special_name(item, SN_SLOW_DESCENT);
            item.cost += 250;
        }
    }
}

pub fn cursed_boots<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    match rng.gen_range(1..=3) {
        1 => {
            item.flags |= TR_SPEED;
            set_special_name(item, SN_SLOWNESS);
            item.misc_use = -1;
        }
        2 => {
            item.flags |= TR_AGGRAVATE;
            set_special_name(item, SN_NOISE);
        }
        _ => {
            set_special_name(item, SN_GREAT_MASS);
            item.weight = item.weight.saturating_mul(5);
        }
    }
    item.cost = 0;
    item.to_ac -= magic_enchantment_bonus(2, 45, level, rng) as i16;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
}

pub fn magical_helms<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
    if !magic_should_be_enchanted(special, rng) {
        return;
    }

    if item.sub_category_id < 6 {
        match rng.gen_range(1..=3) {
            1 => {
                item.misc_use = rng.gen_range(1..=2);
                item.flags |= TR_INT;
                set_special_name(item, SN_INTELLIGENCE);
                item.cost += item.misc_use as u32 * 500;
            }
            2 => {
                item.misc_use = rng.gen_range(1..=2);
                item.flags |= TR_WIS;
                set_special_name(item, SN_WISDOM);
                item.cost += item.misc_use as u32 * 500;
            }
            _ => {
                item.misc_use = 1 + rng.gen_range(1..=4);
                item.flags |= TR_INFRA;
                set_special_name(item, SN_INFRAVISION);
                item.cost += item.misc_use as u32 * 250;
            }
        }
        return;
    }

    // Crowns (sub_category_id >= 6)
    match rng.gen_range(1..=6) {
        1 => {
            item.misc_use = rng.gen_range(1..=3);
            item.flags |= TR_FREE_ACT | TR_CON | TR_DEX | TR_STR;
            set_special_name(item, SN_MIGHT);
            item.cost += 1000 + item.misc_use as u32 * 500;
        }
        2 => {
            item.misc_use = rng.gen_range(1..=3);
            item.flags |= TR_CHR | TR_WIS;
            set_special_name(item, SN_LORDLINESS);
            item.cost += 1000 + item.misc_use as u32 * 500;
        }
        3 => {
            item.misc_use = rng.gen_range(1..=3);
            item.flags |= TR_RES_LIGHT | TR_RES_COLD | TR_RES_ACID | TR_RES_FIRE | TR_INT;
            set_special_name(item, SN_MAGI);
            item.cost += 3000 + item.misc_use as u32 * 500;
        }
        4 => {
            item.misc_use = rng.gen_range(1..=3);
            item.flags |= TR_CHR;
            set_special_name(item, SN_BEAUTY);
            item.cost += 750;
        }
        5 => {
            item.misc_use = 5 * (1 + rng.gen_range(1..=4));
            item.flags |= TR_SEE_INVIS | TR_SEARCH;
            set_special_name(item, SN_SEEING);
            item.cost += 1000 + item.misc_use as u32 * 100;
        }
        _ => {
            item.flags |= TR_REGEN;
            set_special_name(item, SN_REGENERATION);
            item.cost += 1500;
        }
    }
}

pub fn cursed_helms<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    item.to_ac -= magic_enchantment_bonus(1, 45, level, rng) as i16;
    item.flags |= TR_CURSED;
    item.is_cursed = true;
    item.cost = 0;

    if !magic_should_be_enchanted(special, rng) {
        return;
    }

    match rng.gen_range(1..=7) {
        1 => {
            item.misc_use = -rng.gen_range(1..=5);
            item.flags |= TR_INT;
            set_special_name(item, SN_STUPIDITY);
        }
        2 => {
            item.misc_use = -rng.gen_range(1..=5);
            item.flags |= TR_WIS;
            set_special_name(item, SN_DULLNESS);
        }
        3 => {
            item.flags |= TR_BLIND;
            set_special_name(item, SN_BLINDNESS);
        }
        4 => {
            item.flags |= TR_TIMID;
            set_special_name(item, SN_TIMIDNESS);
        }
        5 => {
            item.misc_use = -rng.gen_range(1..=5);
            item.flags |= TR_STR;
            set_special_name(item, SN_WEAKNESS);
        }
        6 => {
            item.flags |= TR_TELEPORT;
            set_special_name(item, SN_TELEPORTATION);
        }
        _ => {
            item.misc_use = -rng.gen_range(1..=5);
            item.flags |= TR_CHR;
            set_special_name(item, SN_UGLINESS);
        }
    }
}

pub fn process_rings<R: rand::Rng>(item: &mut Item, level: i32, cursed: i32, rng: &mut R) {
    match item.sub_category_id {
        0..=3 => {
            if magic_should_be_enchanted(cursed, rng) {
                item.misc_use = -magic_enchantment_bonus(1, 20, level, rng);
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            } else {
                item.misc_use = magic_enchantment_bonus(1, 10, level, rng);
                item.cost = item.cost.saturating_add(item.misc_use as u32 * 100);
            }
        }
        4 => {
            if magic_should_be_enchanted(cursed, rng) {
                item.misc_use = -rng.gen_range(1..=3);
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            } else {
                item.misc_use = 1;
            }
        }
        5 => {
            item.misc_use = 5 * magic_enchantment_bonus(1, 20, level, rng);
            item.cost = item.cost.saturating_add(item.misc_use as u32 * 50);
            if magic_should_be_enchanted(cursed, rng) {
                item.misc_use = -item.misc_use;
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            }
        }
        19 => {
            // Increase damage
            item.to_damage += magic_enchantment_bonus(1, 20, level, rng) as i16;
            item.cost = item.cost.saturating_add(item.to_damage.max(0) as u32 * 100);
            if magic_should_be_enchanted(cursed, rng) {
                item.to_damage = -item.to_damage;
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            }
        }
        20 => {
            // Increase To-Hit
            item.to_hit += magic_enchantment_bonus(1, 20, level, rng) as i16;
            item.cost = item.cost.saturating_add(item.to_hit.max(0) as u32 * 100);
            if magic_should_be_enchanted(cursed, rng) {
                item.to_hit = -item.to_hit;
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            }
        }
        21 => {
            // Protection
            item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
            item.cost = item.cost.saturating_add(item.to_ac.max(0) as u32 * 100);
            if magic_should_be_enchanted(cursed, rng) {
                item.to_ac = -item.to_ac;
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            }
        }
        30 => {
            // Slaying
            item.to_damage += magic_enchantment_bonus(1, 25, level, rng) as i16;
            item.to_hit += magic_enchantment_bonus(1, 25, level, rng) as i16;
            item.cost = item
                .cost
                .saturating_add((item.to_hit.max(0) as u32 + item.to_damage.max(0) as u32) * 100);
            if magic_should_be_enchanted(cursed, rng) {
                item.to_hit = -item.to_hit;
                item.to_damage = -item.to_damage;
                item.flags |= TR_CURSED;
                item.is_cursed = true;
            }
        }
        _ => {}
    }
}

pub fn process_amulets<R: rand::Rng>(item: &mut Item, level: i32, cursed: i32, rng: &mut R) {
    if item.sub_category_id < 2 {
        if magic_should_be_enchanted(cursed, rng) {
            item.misc_use = -magic_enchantment_bonus(1, 20, level, rng);
            item.flags |= TR_CURSED;
            item.is_cursed = true;
        } else {
            item.misc_use = magic_enchantment_bonus(1, 10, level, rng);
            item.cost = item.cost.saturating_add(item.misc_use as u32 * 100);
        }
    } else if item.sub_category_id == 2 {
        item.misc_use = 5 * magic_enchantment_bonus(1, 25, level, rng);
        if magic_should_be_enchanted(cursed, rng) {
            item.misc_use = -item.misc_use;
            item.flags |= TR_CURSED;
            item.is_cursed = true;
        } else {
            item.cost = item.cost.saturating_add(item.misc_use as u32 * 50);
        }
    } else if item.sub_category_id == 8 {
        // Amulet of the Magi is never cursed
        item.misc_use = 5 * magic_enchantment_bonus(1, 25, level, rng);
        item.cost = item.cost.saturating_add(item.misc_use as u32 * 20);
    }
}

pub fn wand_magic<R: rand::Rng>(sub_cat: u16, rng: &mut R) -> i32 {
    match sub_cat {
        0 => rng.gen_range(1..=10) + 6,
        1 => rng.gen_range(1..=8) + 6,
        2 => rng.gen_range(1..=5) + 6,
        3 => rng.gen_range(1..=8) + 6,
        4 => rng.gen_range(1..=4) + 3,
        5 => rng.gen_range(1..=8) + 6,
        6 | 7 => rng.gen_range(1..=20) + 12,
        8 => rng.gen_range(1..=10) + 6,
        9 => rng.gen_range(1..=12) + 6,
        10 => rng.gen_range(1..=10) + 12,
        11 => rng.gen_range(1..=3) + 3,
        12 => rng.gen_range(1..=8) + 6,
        13 => rng.gen_range(1..=10) + 6,
        14 | 15 => rng.gen_range(1..=5) + 3,
        16 => rng.gen_range(1..=5) + 6,
        17 => rng.gen_range(1..=5) + 4,
        18 => rng.gen_range(1..=8) + 4,
        19 => rng.gen_range(1..=6) + 2,
        20 => rng.gen_range(1..=4) + 2,
        21 => rng.gen_range(1..=8) + 6,
        22 => rng.gen_range(1..=5) + 2,
        23 => rng.gen_range(1..=12) + 12,
        _ => -1,
    }
}

pub fn staff_magic<R: rand::Rng>(sub_cat: u16, rng: &mut R) -> i32 {
    match sub_cat {
        0 => rng.gen_range(1..=20) + 12,
        1 => rng.gen_range(1..=8) + 6,
        2 => rng.gen_range(1..=5) + 6,
        3 => rng.gen_range(1..=20) + 12,
        4 => rng.gen_range(1..=15) + 6,
        5 => rng.gen_range(1..=4) + 5,
        6 => rng.gen_range(1..=5) + 3,
        7 | 8 => rng.gen_range(1..=3) + 1,
        9 => rng.gen_range(1..=5) + 6,
        10 => rng.gen_range(1..=10) + 12,
        11..=13 => rng.gen_range(1..=5) + 6,
        14 => rng.gen_range(1..=10) + 12,
        15 => rng.gen_range(1..=3) + 4,
        16 | 17 => rng.gen_range(1..=5) + 6,
        18 => rng.gen_range(1..=3) + 4,
        19 => rng.gen_range(1..=10) + 12,
        20 | 21 => rng.gen_range(1..=3) + 4,
        22 => rng.gen_range(1..=10) + 6,
        _ => -1,
    }
}

pub fn magical_cloak<R: rand::Rng>(item: &mut Item, special: i32, level: i32, rng: &mut R) {
    if !magic_should_be_enchanted(special, rng) {
        item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
        return;
    }

    if rng.gen_range(1..=2) == 1 {
        set_special_name(item, SN_PROTECTION);
        item.to_ac += magic_enchantment_bonus(2, 40, level, rng) as i16;
        item.cost += 250;
        return;
    }

    item.to_ac += magic_enchantment_bonus(1, 20, level, rng) as i16;
    item.misc_use = rng.gen_range(1..=3);
    item.flags |= TR_STEALTH;
    set_special_name(item, SN_STEALTH);
    item.cost += 500;
}

pub fn cursed_cloak<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    match rng.gen_range(1..=3) {
        1 => {
            item.flags |= TR_AGGRAVATE;
            set_special_name(item, SN_IRRITATION);
            item.to_ac -= magic_enchantment_bonus(1, 10, level, rng) as i16;
            item.to_hit -= magic_enchantment_bonus(1, 10, level, rng) as i16;
            item.to_damage -= magic_enchantment_bonus(1, 10, level, rng) as i16;
            item.cost = 0;
        }
        2 => {
            set_special_name(item, SN_VULNERABILITY);
            item.to_ac -= magic_enchantment_bonus(10, 100, level + 50, rng) as i16;
            item.cost = 0;
        }
        _ => {
            set_special_name(item, SN_ENVELOPING);
            item.to_ac -= magic_enchantment_bonus(1, 10, level, rng) as i16;
            item.to_hit -= magic_enchantment_bonus(2, 40, level + 10, rng) as i16;
            item.to_damage -= magic_enchantment_bonus(2, 40, level + 10, rng) as i16;
            item.cost = 0;
        }
    }
    item.flags |= TR_CURSED;
    item.is_cursed = true;
}

pub fn magical_chests<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    let magic_type = rng.gen_range(1..=(level + 4));
    match magic_type {
        1 => {
            item.flags = 0;
            set_special_name(item, SN_EMPTY);
        }
        2 => {
            item.flags |= CH_LOCKED;
            set_special_name(item, SN_LOCKED_DESC);
        }
        3 | 4 => {
            item.flags |= CH_LOSE_STR | CH_LOCKED;
            set_special_name(item, SN_POISON_NEEDLE);
        }
        5 | 6 => {
            item.flags |= CH_POISON | CH_LOCKED;
            set_special_name(item, SN_POISON_NEEDLE);
        }
        7..=9 => {
            item.flags |= CH_PARALYSED | CH_LOCKED;
            set_special_name(item, SN_GAS_TRAP);
        }
        10 | 11 => {
            item.flags |= CH_EXPLODE | CH_LOCKED;
            set_special_name(item, SN_EXPLOSION_DEVICE);
        }
        12..=14 => {
            item.flags |= CH_SUMMON | CH_LOCKED;
            set_special_name(item, SN_SUMMONING_RUNES);
        }
        15..=17 => {
            item.flags |= CH_PARALYSED | CH_POISON | CH_LOSE_STR | CH_LOCKED;
            set_special_name(item, SN_MULTIPLE_TRAPS);
        }
        _ => {
            item.flags |= CH_SUMMON | CH_EXPLODE | CH_LOCKED;
            set_special_name(item, SN_MULTIPLE_TRAPS);
        }
    }
}

pub fn magical_projectile<R: rand::Rng>(
    item: &mut Item,
    special: i32,
    level: i32,
    chance: i32,
    cursed: i32,
    rng: &mut R,
) {
    if matches!(
        item.category_id,
        TV_SLING_AMMO | TV_BOLT | TV_ARROW | TV_SPIKE
    ) {
        if magic_should_be_enchanted(chance, rng) {
            item.to_hit += magic_enchantment_bonus(1, 35, level, rng) as i16;
            item.to_damage += magic_enchantment_bonus(1, 35, level, rng) as i16;

            if magic_should_be_enchanted(3 * special / 2, rng) {
                match rng.gen_range(1..=10) {
                    1..=3 => {
                        set_special_name(item, SN_SLAYING);
                        item.to_hit += 5;
                        item.to_damage += 5;
                        item.cost += 20;
                    }
                    4 | 5 => {
                        item.flags |= TR_FLAME_TONGUE;
                        item.to_hit += 2;
                        item.to_damage += 4;
                        set_special_name(item, SN_FIRE);
                        item.cost += 25;
                    }
                    6 | 7 => {
                        item.flags |= TR_SLAY_EVIL;
                        item.to_hit += 3;
                        item.to_damage += 3;
                        set_special_name(item, SN_SLAY_EVIL);
                        item.cost += 25;
                    }
                    8 | 9 => {
                        item.flags |= TR_SLAY_ANIMAL;
                        item.to_hit += 2;
                        item.to_damage += 2;
                        set_special_name(item, SN_SLAY_ANIMAL);
                        item.cost += 30;
                    }
                    _ => {
                        item.flags |= TR_SLAY_DRAGON;
                        item.to_hit += 3;
                        item.to_damage += 3;
                        set_special_name(item, SN_DRAGON_SLAYING);
                        item.cost += 35;
                    }
                }
            }
        } else if magic_should_be_enchanted(cursed, rng) {
            item.to_hit -= magic_enchantment_bonus(5, 55, level, rng) as i16;
            item.to_damage -= magic_enchantment_bonus(5, 55, level, rng) as i16;
            item.flags |= TR_CURSED;
            item.is_cursed = true;
            item.cost = 0;
        }
    }

    let mut count = 0;
    for _ in 0..7 {
        count += rng.gen_range(1..=6);
    }
    item.count = count;

    let ctr = MISSILES_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    item.misc_use = ctr as i32;
}

pub fn apply_magical_ability<R: rand::Rng>(item: &mut Item, level: i32, rng: &mut R) {
    let mut chance = (15 + level).min(70);
    let mut special = chance / 6;
    let cursed = (10 * chance) / 13;

    match item.category_id {
        TV_SHIELD | TV_HARD_ARMOR | TV_SOFT_ARMOR => {
            if magic_should_be_enchanted(chance, rng) {
                magical_armor(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_armor(item, level, rng);
            }
        }
        TV_HAFTED | TV_POLEARM | TV_SWORD => {
            if magic_should_be_enchanted(chance, rng) {
                magical_sword(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_sword(item, level, rng);
            }
        }
        TV_BOW => {
            if magic_should_be_enchanted(chance, rng) {
                magical_bow(item, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_bow(item, level, rng);
            }
        }
        TV_DIGGING => {
            if magic_should_be_enchanted(chance, rng) {
                if rng.gen_range(1..=3) < 3 {
                    magical_digging_tool(item, level, rng);
                } else {
                    cursed_digging_tool(item, level, rng);
                }
            }
        }
        TV_GLOVES => {
            if magic_should_be_enchanted(chance, rng) {
                magical_gloves(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_gloves(item, special, level, rng);
            }
        }
        TV_BOOTS => {
            if magic_should_be_enchanted(chance, rng) {
                magical_boots(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_boots(item, level, rng);
            }
        }
        TV_HELM => {
            if item.sub_category_id >= 6 && item.sub_category_id <= 8 {
                chance += (item.cost / 100) as i32;
                special += special;
            }
            if magic_should_be_enchanted(chance, rng) {
                magical_helms(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_helms(item, special, level, rng);
            }
        }
        TV_RING => {
            process_rings(item, level, cursed, rng);
        }
        TV_AMULET => {
            process_amulets(item, level, cursed, rng);
        }
        TV_LIGHT => {
            if (item.sub_category_id % 2) == 1 {
                if item.misc_use > 0 {
                    item.misc_use = rng.gen_range(1..=item.misc_use);
                }
                item.sub_category_id = item.sub_category_id.saturating_sub(1);
            }
        }
        TV_WAND => {
            let magic_amount = wand_magic(item.sub_category_id, rng);
            if magic_amount != -1 {
                item.misc_use = magic_amount;
                if let ItemType::Wand {
                    ref mut charges, ..
                } = item.item_type
                {
                    *charges = magic_amount as u32;
                }
            }
        }
        TV_STAFF => {
            let magic_amount = staff_magic(item.sub_category_id, rng);
            if magic_amount != -1 {
                item.misc_use = magic_amount;
                if let ItemType::Staff {
                    ref mut charges, ..
                } = item.item_type
                {
                    *charges = magic_amount as u32;
                }
            }
        }
        TV_CLOAK => {
            if magic_should_be_enchanted(chance, rng) {
                magical_cloak(item, special, level, rng);
            } else if magic_should_be_enchanted(cursed, rng) {
                cursed_cloak(item, level, rng);
            }
        }
        TV_CHEST => {
            magical_chests(item, level, rng);
        }
        TV_SLING_AMMO | TV_SPIKE | TV_BOLT | TV_ARROW => {
            magical_projectile(item, special, level, chance, cursed, rng);
        }
        _ => {}
    }
}

pub fn apply_slaying_damage(
    weapon_flags: u32,
    base_damage: i32,
    monster_creature_id: usize,
) -> i32 {
    if monster_creature_id >= crate::entity::monster_data::CREATURES_LIST.len() {
        return base_damage;
    }
    let creature = &crate::entity::monster_data::CREATURES_LIST[monster_creature_id];
    let defs = creature.defenses;

    if (defs & crate::entity::monster_data::CD_DRAGON) != 0 && (weapon_flags & TR_SLAY_DRAGON) != 0
    {
        base_damage * 4
    } else if (defs & crate::entity::monster_data::CD_UNDEAD) != 0
        && (weapon_flags & TR_SLAY_UNDEAD) != 0
    {
        base_damage * 3
    } else if ((defs & crate::entity::monster_data::CD_ANIMAL) != 0
        && (weapon_flags & TR_SLAY_ANIMAL) != 0)
        || ((defs & crate::entity::monster_data::CD_EVIL) != 0
            && (weapon_flags & TR_SLAY_EVIL) != 0)
    {
        base_damage * 2
    } else if ((defs & crate::entity::monster_data::CD_FROST) != 0
        && (weapon_flags & TR_FROST_BRAND) != 0)
        || ((defs & crate::entity::monster_data::CD_FIRE) != 0
            && (weapon_flags & TR_FLAME_TONGUE) != 0)
    {
        base_damage * 3 / 2
    } else {
        base_damage
    }
}

pub fn calculate_slaying_multiplier(weapon_flags: u32, monster_creature_id: usize) -> u32 {
    if monster_creature_id >= crate::entity::monster_data::CREATURES_LIST.len() {
        return 1;
    }
    let creature = &crate::entity::monster_data::CREATURES_LIST[monster_creature_id];
    let defs = creature.defenses;

    if (defs & crate::entity::monster_data::CD_DRAGON) != 0 && (weapon_flags & TR_SLAY_DRAGON) != 0
    {
        4
    } else if (defs & crate::entity::monster_data::CD_UNDEAD) != 0
        && (weapon_flags & TR_SLAY_UNDEAD) != 0
    {
        3
    } else if ((defs & crate::entity::monster_data::CD_ANIMAL) != 0
        && (weapon_flags & TR_SLAY_ANIMAL) != 0)
        || ((defs & crate::entity::monster_data::CD_EVIL) != 0
            && (weapon_flags & TR_SLAY_EVIL) != 0)
    {
        2
    } else {
        1
    }
}

pub fn get_canonical_store_choices(shop_type: ShopType) -> &'static [u16; 26] {
    &STORE_CHOICES[shop_type as usize]
}

pub fn generate_canonical_shop_items(shop_type: ShopType) -> Vec<(String, u32, ItemType)> {
    let choices = get_canonical_store_choices(shop_type);
    let mut items = Vec::new();
    let mut seen_indices = std::collections::HashSet::new();

    for &idx in choices {
        if seen_indices.insert(idx) {
            let def = &GAME_OBJECTS[idx as usize];
            let item_type = match def.category_id {
                TV_SWORD | TV_HAFTED | TV_POLEARM | TV_DIGGING => {
                    ItemType::Weapon { damage: def.damage }
                }
                TV_BOW => ItemType::Bow {
                    multiplier: def.misc_use.max(1) as u32,
                },
                TV_ARROW | TV_BOLT | TV_SLING_AMMO => ItemType::Missile { damage: def.damage },
                TV_SPIKE => ItemType::Missile {
                    damage: Dice::new(1, 1),
                },
                TV_BOOTS | TV_GLOVES | TV_CLOAK | TV_HELM | TV_SHIELD | TV_HARD_ARMOR
                | TV_SOFT_ARMOR => ItemType::Armor { ac: def.ac as i32 },
                TV_RING => ItemType::Ring {
                    bonus: def.misc_use as i32,
                },
                TV_AMULET => ItemType::Amulet {
                    bonus: def.misc_use as i32,
                },
                TV_LIGHT => ItemType::Light {
                    fuel: def.misc_use as i32,
                },
                TV_FOOD => ItemType::Food {
                    nutrition: def.misc_use as i32,
                },
                TV_FLASK => ItemType::Potion { heal_amount: 0 },
                TV_POTION1 | TV_POTION2 => {
                    let n_lower = def.name.to_lowercase();
                    let heal = if n_lower.contains("critical") {
                        40
                    } else if n_lower.contains("serious") {
                        20
                    } else if n_lower.contains("light") {
                        10
                    } else if n_lower.contains("healing") {
                        50
                    } else {
                        0
                    };
                    ItemType::Potion { heal_amount: heal }
                }
                TV_SCROLL1 | TV_SCROLL2 => {
                    let n_lower = def.name.to_lowercase();
                    let tele = n_lower.contains("phase door") || n_lower.contains("teleport");
                    ItemType::Scroll { teleport: tele }
                }
                TV_MAGIC_BOOK => ItemType::MagicBook {
                    spell_flags: def.flags,
                },
                TV_PRAYER_BOOK => ItemType::PrayerBook {
                    spell_flags: def.flags,
                },
                TV_WAND => ItemType::Wand {
                    charges: def.misc_use.max(1) as u32,
                    spell_index: def.sub_category_id as usize,
                },
                TV_STAFF => ItemType::Staff {
                    charges: def.misc_use.max(1) as u32,
                    prayer_index: def.sub_category_id as usize,
                },
                _ => ItemType::Food { nutrition: 0 },
            };
            let clean_name = clean_item_name(def.name, def.category_id, 1);
            items.push((clean_name, def.cost.max(1) as u32, item_type));
        }
    }
    items
}

pub fn get_canonical_shop_item(shop_type: ShopType, slot_idx: usize) -> Option<Item> {
    let choices = get_canonical_store_choices(shop_type);
    let mut seen_indices = std::collections::HashSet::new();
    let mut current_slot = 0;

    for &idx in choices {
        if seen_indices.insert(idx) {
            if current_slot == slot_idx {
                return Some(create_item_from_index(idx as usize, 1));
            }
            current_slot += 1;
        }
    }
    None
}

pub fn generate_canonical_floor_item<R: rand::Rng>(depth: u32, rng: &mut R) -> Item {
    let roll = rng.gen_range(0..100);
    if roll < 15 {
        let gold_names = [
            ("copper", 3),
            ("copper", 4),
            ("copper", 5),
            ("silver", 6),
            ("silver", 7),
            ("silver", 8),
            ("garnets", 9),
            ("garnets", 10),
            ("gold", 12),
            ("gold", 14),
            ("gold", 16),
            ("opals", 18),
            ("sapphires", 20),
            ("rubies", 24),
            ("diamonds", 28),
            ("emeralds", 32),
            ("mithril", 40),
            ("adamantite", 80),
        ];
        let mut gold_type_id =
            (((rng.gen_range(1..=(depth as i32 + 2)) + 2) / 2) - 1).max(0) as usize;
        if rng.gen_range(1..=12) == 1 {
            gold_type_id += rng.gen_range(1..=(depth as i32 + 1)) as usize;
        }
        if gold_type_id >= gold_names.len() {
            gold_type_id = gold_names.len() - 1;
        }
        let (_, base_cost) = gold_names[gold_type_id];
        let amount = (base_cost + 8 * rng.gen_range(1..=base_cost) + rng.gen_range(1..=8)) as u32;
        return Item::new(
            &format!("Gold Pile [{} gp]", amount),
            1,
            1,
            ItemType::Food { nutrition: 0 },
        );
    }

    let chosen_idx = TREASURE_TABLES.get_random_object_id(depth as usize, false, rng);
    let def = &GAME_OBJECTS[chosen_idx];
    let count = match def.category_id {
        TV_ARROW | TV_BOLT | TV_SLING_AMMO => rng.gen_range(10..=25),
        TV_SPIKE => rng.gen_range(5..=15),
        TV_FOOD => rng.gen_range(1..=3),
        _ => 1,
    };

    let mut item = create_item_from_index(chosen_idx, count);
    apply_magical_ability(&mut item, depth as i32, rng);

    match def.category_id {
        TV_POTION1 | TV_POTION2 | TV_SCROLL1 | TV_SCROLL2 | TV_WAND | TV_STAFF | TV_RING
        | TV_AMULET => {
            item.identified = false;
        }
        _ => {}
    }

    item
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_game_objects_count_and_identity() {
        assert_eq!(GAME_OBJECTS.len(), 420);
        // Entry 28: Dagger
        assert_eq!(GAME_OBJECTS[28].name, "& Dagger (Main Gauche)");
        assert_eq!(GAME_OBJECTS[28].category_id, TV_SWORD);
        assert_eq!(GAME_OBJECTS[28].damage, Dice { num: 1, sides: 5 });
        assert_eq!(GAME_OBJECTS[28].cost, 25);

        // Entry 37: Broadsword
        assert_eq!(GAME_OBJECTS[37].name, "& Broadsword");
        assert_eq!(GAME_OBJECTS[37].damage, Dice { num: 2, sides: 5 });

        // Entry 366: Flask of Oil
        assert_eq!(GAME_OBJECTS[366].name, "& Flask~ of Oil");
        assert_eq!(GAME_OBJECTS[366].category_id, TV_FLASK);
    }

    #[test]
    fn test_clean_item_name() {
        assert_eq!(
            clean_item_name("& Dagger (Main Gauche)", TV_SWORD, 1),
            "Dagger (Main Gauche)"
        );
        assert_eq!(
            clean_item_name("& Ration~ of Food", TV_FOOD, 1),
            "Ration of Food"
        );
        assert_eq!(
            clean_item_name("& Ration~ of Food", TV_FOOD, 3),
            "Rations of Food"
        );
        assert_eq!(clean_item_name("Strength", TV_RING, 1), "Ring of Strength");
        assert_eq!(clean_item_name("Wisdom", TV_AMULET, 1), "Amulet of Wisdom");
        assert_eq!(
            clean_item_name("Identify", TV_SCROLL1, 1),
            "Scroll of Identify"
        );
        assert_eq!(
            clean_item_name("Healing", TV_POTION1, 1),
            "Potion of Healing"
        );
    }

    #[test]
    fn test_canonical_store_choices() {
        for choices in &STORE_CHOICES {
            assert_eq!(choices.len(), 26);
            for &c in choices {
                assert!((c as usize) < GAME_OBJECTS.len());
                assert!(!GAME_OBJECTS[c as usize].name.is_empty());
            }
        }
    }

    #[test]
    fn test_ego_weapon_and_slaying_multipliers() {
        let mut dagger = create_item_from_index(28, 1);
        dagger.flags |= TR_SLAY_EVIL;
        dagger.ego_name = Some("(SE)".to_string());

        // Evil monster: Orc (creature 10 or similar evil creature)
        let orc_id = crate::entity::monster_data::CREATURES_LIST
            .iter()
            .position(|c| (c.defenses & crate::entity::monster_data::CD_EVIL) != 0)
            .expect("Must find an evil creature");

        let mult = calculate_slaying_multiplier(dagger.flags, orc_id);
        assert_eq!(mult, 2);

        // Dragon slayer
        let mut sword = create_item_from_index(37, 1);
        sword.flags |= TR_SLAY_DRAGON;
        let dragon_id = crate::entity::monster_data::CREATURES_LIST
            .iter()
            .position(|c| (c.defenses & crate::entity::monster_data::CD_DRAGON) != 0)
            .expect("Must find a dragon creature");
        let dragon_mult = calculate_slaying_multiplier(sword.flags, dragon_id);
        assert_eq!(dragon_mult, 4);
    }

    #[test]
    fn test_canonical_shop_items_generation() {
        for shop in [
            ShopType::General,
            ShopType::Armory,
            ShopType::Weaponsmith,
            ShopType::Temple,
            ShopType::Alchemy,
            ShopType::Magic,
        ] {
            let shop_items = generate_canonical_shop_items(shop);
            assert!(!shop_items.is_empty(), "Store {:?} must have items", shop);
            for (name, cost, _item_type) in &shop_items {
                assert!(!name.is_empty());
                assert!(*cost > 0);
            }
            // Check get_canonical_shop_item
            let first_item = get_canonical_shop_item(shop, 0).expect("First item must exist");
            assert!(!first_item.name.is_empty());
            assert!(first_item.cost > 0);
        }
    }

    #[test]
    fn test_canonical_floor_item_generation() {
        let mut rng = rand::thread_rng();
        for depth in [0, 5, 20, 50] {
            for _ in 0..20 {
                let item = generate_canonical_floor_item(depth, &mut rng);
                assert!(!item.name.is_empty());
                assert!(item.count >= 1);
            }
        }
    }

    #[test]
    fn test_player_slaying_combat_integration() {
        use crate::player::{Class, Player, Race};
        let mut player = Player::new("Hero", Race::Human, Class::Warrior, 0, 0);
        let mut dragon_slayer = create_item_from_index(37, 1); // Broadsword (2d5)
        dragon_slayer.flags |= TR_SLAY_DRAGON;
        dragon_slayer.to_damage = 5;
        dragon_slayer.equipped_slot = Some("wielded".to_string());
        player.equipment.push(dragon_slayer);

        let dragon_id = crate::entity::monster_data::CREATURES_LIST
            .iter()
            .position(|c| (c.defenses & crate::entity::monster_data::CD_DRAGON) != 0)
            .expect("Must find a dragon");

        let normal_id = crate::entity::monster_data::CREATURES_LIST
            .iter()
            .position(|c| {
                (c.defenses & crate::entity::monster_data::CD_DRAGON) == 0
                    && (c.defenses & crate::entity::monster_data::CD_EVIL) == 0
            })
            .expect("Must find a non-dragon non-evil");

        let mut rng = rand::thread_rng();
        let weapon = player.equipped_weapon();
        let dmg_dragon = player.single_blown_damage(&mut rng, weapon, Some(dragon_id));
        let dmg_normal = player.single_blown_damage(&mut rng, weapon, Some(normal_id));

        // Dragon damage should be substantially higher due to 4x multiplier on weapon dice (2d5 * 4 = 8..40 vs 2..10)
        assert!(dmg_dragon > dmg_normal || dmg_dragon >= 13);
    }

    #[test]
    fn test_player_armor_to_ac_and_speed_integration() {
        use crate::player::{Class, Player, Race};
        let mut player = Player::new("Hero", Race::Human, Class::Warrior, 0, 0);
        let base_ac = player.calculate_ac();
        let base_speed = player.speed_modifier();

        let mut boots = create_item_from_index(127, 1); // Pair of Hard Leather Boots
        boots.to_ac = 4;
        boots.flags |= TR_SPEED;
        boots.equipped_slot = Some("feet".to_string());
        player.equipment.push(boots);

        assert_eq!(player.calculate_ac(), base_ac + 3 + 4); // 3 base AC + 4 to_ac
        assert_eq!(player.speed_modifier(), base_speed - 1); // 1 faster
    }

    #[test]
    fn test_item_display_name_formatting() {
        let mut item = create_item_from_index(37, 1);
        assert_eq!(item.display_name(), "Broadsword");

        item.to_hit = 2;
        item.to_damage = 3;
        assert_eq!(item.display_name(), "Broadsword (+2,+3)");

        item.to_ac = 1;
        assert_eq!(item.display_name(), "Broadsword (+2,+3) [+1]");

        item.ego_name = Some("(SD)".to_string());
        assert_eq!(item.display_name(), "Broadsword (SD) (+2,+3) [+1]");
    }
}
