use crate::dice::Dice;
use crate::dungeon::DungeonLevel;
use crate::dungeon::tile::{TileType, TrapType};
use crate::entity::monster::Monster;
use crate::entity::monster_data::{
    CD_ACID, CD_EVIL, CD_FIRE, CD_FROST, CD_LIGHT, CD_POISON, CS_BR_ACID, CS_BR_FIRE, CS_BR_FROST,
    CS_BR_GAS, CS_BR_LIGHT,
};
use crate::entity::treasure_data::{GAME_OBJECTS, TV_POTION2, TV_SCROLL2};
use crate::player::{Class, Item, ItemType, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicSpellType {
    MagicMissile,
    Lightning,
    PoisonGas,
    Acid,
    Frost,
    Fire,
    HolyOrb,
}

impl MagicSpellType {
    pub fn flags(self) -> (u16, u32) {
        match self {
            MagicSpellType::MagicMissile => (0, 0),
            MagicSpellType::Lightning => (CD_LIGHT, CS_BR_LIGHT),
            MagicSpellType::PoisonGas => (CD_POISON, CS_BR_GAS),
            MagicSpellType::Acid => (CD_ACID, CS_BR_ACID),
            MagicSpellType::Frost => (CD_FROST, CS_BR_FROST),
            MagicSpellType::Fire => (CD_FIRE, CS_BR_FIRE),
            MagicSpellType::HolyOrb => (CD_EVIL, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellDef {
    pub name: &'static str,
    pub level_required: u32,
    pub mana_required: u32,
    pub failure_chance: u32,
    pub exp_gain: u32,
}

pub const MAGE_SPELL_NAMES: [&str; 31] = [
    "Magic Missile",
    "Detect Monsters",
    "Phase Door",
    "Light Area",
    "Cure Light Wounds",
    "Find Hidden Traps/Doors",
    "Stinking Cloud",
    "Confusion",
    "Lightning Bolt",
    "Trap/Door Destruction",
    "Sleep I",
    "Cure Poison",
    "Teleport Self",
    "Remove Curse",
    "Frost Bolt",
    "Turn Stone to Mud",
    "Create Food",
    "Recharge Item I",
    "Sleep II",
    "Polymorph Other",
    "Identify",
    "Sleep III",
    "Fire Bolt",
    "Slow Monster",
    "Frost Ball",
    "Recharge Item II",
    "Teleport Other",
    "Haste Self",
    "Fire Ball",
    "Word of Destruction",
    "Genocide",
];

pub const PRIEST_PRAYER_NAMES: [&str; 31] = [
    "Detect Evil",
    "Cure Light Wounds",
    "Bless",
    "Remove Fear",
    "Call Light",
    "Find Traps",
    "Detect Doors/Stairs",
    "Slow Poison",
    "Blind Creature",
    "Portal",
    "Cure Medium Wounds",
    "Chant",
    "Sanctuary",
    "Create Food",
    "Remove Curse",
    "Resist Heat and Cold",
    "Neutralize Poison",
    "Orb of Draining",
    "Cure Serious Wounds",
    "Sense Invisible",
    "Protection from Evil",
    "Earthquake",
    "Sense Surroundings",
    "Cure Critical Wounds",
    "Turn Undead",
    "Prayer",
    "Dispel Undead",
    "Heal",
    "Dispel Evil",
    "Glyph of Warding",
    "Holy Word",
];

// Mage spells table for Mage class (canonical data_player.cpp)
pub const MAGE_CLASS_SPELLS: [SpellDef; 31] = [
    SpellDef {
        name: "Magic Missile",
        level_required: 1,
        mana_required: 1,
        failure_chance: 22,
        exp_gain: 1,
    },
    SpellDef {
        name: "Detect Monsters",
        level_required: 1,
        mana_required: 1,
        failure_chance: 23,
        exp_gain: 1,
    },
    SpellDef {
        name: "Phase Door",
        level_required: 1,
        mana_required: 2,
        failure_chance: 24,
        exp_gain: 1,
    },
    SpellDef {
        name: "Light Area",
        level_required: 1,
        mana_required: 2,
        failure_chance: 26,
        exp_gain: 1,
    },
    SpellDef {
        name: "Cure Light Wounds",
        level_required: 3,
        mana_required: 3,
        failure_chance: 25,
        exp_gain: 2,
    },
    SpellDef {
        name: "Find Hidden Traps/Doors",
        level_required: 3,
        mana_required: 3,
        failure_chance: 25,
        exp_gain: 1,
    },
    SpellDef {
        name: "Stinking Cloud",
        level_required: 3,
        mana_required: 3,
        failure_chance: 27,
        exp_gain: 2,
    },
    SpellDef {
        name: "Confusion",
        level_required: 3,
        mana_required: 4,
        failure_chance: 30,
        exp_gain: 1,
    },
    SpellDef {
        name: "Lightning Bolt",
        level_required: 5,
        mana_required: 4,
        failure_chance: 30,
        exp_gain: 6,
    },
    SpellDef {
        name: "Trap/Door Destruction",
        level_required: 5,
        mana_required: 5,
        failure_chance: 30,
        exp_gain: 8,
    },
    SpellDef {
        name: "Sleep I",
        level_required: 5,
        mana_required: 5,
        failure_chance: 30,
        exp_gain: 5,
    },
    SpellDef {
        name: "Cure Poison",
        level_required: 5,
        mana_required: 5,
        failure_chance: 35,
        exp_gain: 6,
    },
    SpellDef {
        name: "Teleport Self",
        level_required: 7,
        mana_required: 6,
        failure_chance: 35,
        exp_gain: 9,
    },
    SpellDef {
        name: "Remove Curse",
        level_required: 7,
        mana_required: 6,
        failure_chance: 50,
        exp_gain: 10,
    },
    SpellDef {
        name: "Frost Bolt",
        level_required: 7,
        mana_required: 6,
        failure_chance: 40,
        exp_gain: 12,
    },
    SpellDef {
        name: "Turn Stone to Mud",
        level_required: 9,
        mana_required: 7,
        failure_chance: 44,
        exp_gain: 19,
    },
    SpellDef {
        name: "Create Food",
        level_required: 9,
        mana_required: 7,
        failure_chance: 45,
        exp_gain: 19,
    },
    SpellDef {
        name: "Recharge Item I",
        level_required: 9,
        mana_required: 7,
        failure_chance: 75,
        exp_gain: 22,
    },
    SpellDef {
        name: "Sleep II",
        level_required: 9,
        mana_required: 7,
        failure_chance: 45,
        exp_gain: 19,
    },
    SpellDef {
        name: "Polymorph Other",
        level_required: 11,
        mana_required: 7,
        failure_chance: 45,
        exp_gain: 25,
    },
    SpellDef {
        name: "Identify",
        level_required: 11,
        mana_required: 7,
        failure_chance: 99,
        exp_gain: 19,
    },
    SpellDef {
        name: "Sleep III",
        level_required: 13,
        mana_required: 7,
        failure_chance: 50,
        exp_gain: 22,
    },
    SpellDef {
        name: "Fire Bolt",
        level_required: 15,
        mana_required: 9,
        failure_chance: 50,
        exp_gain: 25,
    },
    SpellDef {
        name: "Slow Monster",
        level_required: 17,
        mana_required: 9,
        failure_chance: 50,
        exp_gain: 31,
    },
    SpellDef {
        name: "Frost Ball",
        level_required: 19,
        mana_required: 12,
        failure_chance: 55,
        exp_gain: 38,
    },
    SpellDef {
        name: "Recharge Item II",
        level_required: 21,
        mana_required: 12,
        failure_chance: 90,
        exp_gain: 44,
    },
    SpellDef {
        name: "Teleport Other",
        level_required: 23,
        mana_required: 12,
        failure_chance: 60,
        exp_gain: 50,
    },
    SpellDef {
        name: "Haste Self",
        level_required: 25,
        mana_required: 12,
        failure_chance: 65,
        exp_gain: 63,
    },
    SpellDef {
        name: "Fire Ball",
        level_required: 29,
        mana_required: 18,
        failure_chance: 65,
        exp_gain: 88,
    },
    SpellDef {
        name: "Word of Destruction",
        level_required: 33,
        mana_required: 21,
        failure_chance: 80,
        exp_gain: 125,
    },
    SpellDef {
        name: "Genocide",
        level_required: 37,
        mana_required: 25,
        failure_chance: 95,
        exp_gain: 200,
    },
];

// Mage spells table for Rogue class (canonical data_player.cpp)
pub const ROGUE_CLASS_SPELLS: [SpellDef; 31] = [
    SpellDef {
        name: "Magic Missile",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Detect Monsters",
        level_required: 5,
        mana_required: 1,
        failure_chance: 50,
        exp_gain: 1,
    },
    SpellDef {
        name: "Phase Door",
        level_required: 7,
        mana_required: 2,
        failure_chance: 55,
        exp_gain: 1,
    },
    SpellDef {
        name: "Light Area",
        level_required: 9,
        mana_required: 3,
        failure_chance: 60,
        exp_gain: 2,
    },
    SpellDef {
        name: "Cure Light Wounds",
        level_required: 11,
        mana_required: 4,
        failure_chance: 65,
        exp_gain: 2,
    },
    SpellDef {
        name: "Find Hidden Traps/Doors",
        level_required: 13,
        mana_required: 5,
        failure_chance: 70,
        exp_gain: 3,
    },
    SpellDef {
        name: "Stinking Cloud",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Confusion",
        level_required: 15,
        mana_required: 6,
        failure_chance: 75,
        exp_gain: 3,
    },
    SpellDef {
        name: "Lightning Bolt",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Trap/Door Destruction",
        level_required: 17,
        mana_required: 7,
        failure_chance: 80,
        exp_gain: 4,
    },
    SpellDef {
        name: "Sleep I",
        level_required: 19,
        mana_required: 8,
        failure_chance: 85,
        exp_gain: 5,
    },
    SpellDef {
        name: "Cure Poison",
        level_required: 21,
        mana_required: 9,
        failure_chance: 90,
        exp_gain: 6,
    },
    SpellDef {
        name: "Teleport Self",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Remove Curse",
        level_required: 23,
        mana_required: 10,
        failure_chance: 95,
        exp_gain: 7,
    },
    SpellDef {
        name: "Frost Bolt",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Turn Stone to Mud",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Create Food",
        level_required: 25,
        mana_required: 12,
        failure_chance: 95,
        exp_gain: 9,
    },
    SpellDef {
        name: "Recharge Item I",
        level_required: 27,
        mana_required: 15,
        failure_chance: 99,
        exp_gain: 11,
    },
    SpellDef {
        name: "Sleep II",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Polymorph Other",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Identify",
        level_required: 29,
        mana_required: 18,
        failure_chance: 99,
        exp_gain: 19,
    },
    SpellDef {
        name: "Sleep III",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Fire Bolt",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Slow Monster",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Frost Ball",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Recharge Item II",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Teleport Other",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Haste Self",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Fire Ball",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Word of Destruction",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
    SpellDef {
        name: "Genocide",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
];

// Mage spells table for Ranger class (canonical data_player.cpp)
pub const RANGER_CLASS_SPELLS: [SpellDef; 31] = [
    SpellDef {
        name: "Magic Missile",
        level_required: 3,
        mana_required: 1,
        failure_chance: 30,
        exp_gain: 1,
    },
    SpellDef {
        name: "Detect Monsters",
        level_required: 3,
        mana_required: 2,
        failure_chance: 35,
        exp_gain: 2,
    },
    SpellDef {
        name: "Phase Door",
        level_required: 3,
        mana_required: 2,
        failure_chance: 35,
        exp_gain: 2,
    },
    SpellDef {
        name: "Light Area",
        level_required: 5,
        mana_required: 3,
        failure_chance: 35,
        exp_gain: 2,
    },
    SpellDef {
        name: "Cure Light Wounds",
        level_required: 5,
        mana_required: 3,
        failure_chance: 40,
        exp_gain: 2,
    },
    SpellDef {
        name: "Find Hidden Traps/Doors",
        level_required: 5,
        mana_required: 4,
        failure_chance: 45,
        exp_gain: 3,
    },
    SpellDef {
        name: "Stinking Cloud",
        level_required: 7,
        mana_required: 5,
        failure_chance: 40,
        exp_gain: 6,
    },
    SpellDef {
        name: "Confusion",
        level_required: 7,
        mana_required: 6,
        failure_chance: 40,
        exp_gain: 5,
    },
    SpellDef {
        name: "Lightning Bolt",
        level_required: 9,
        mana_required: 7,
        failure_chance: 40,
        exp_gain: 7,
    },
    SpellDef {
        name: "Trap/Door Destruction",
        level_required: 9,
        mana_required: 8,
        failure_chance: 45,
        exp_gain: 8,
    },
    SpellDef {
        name: "Sleep I",
        level_required: 11,
        mana_required: 8,
        failure_chance: 40,
        exp_gain: 10,
    },
    SpellDef {
        name: "Cure Poison",
        level_required: 11,
        mana_required: 9,
        failure_chance: 45,
        exp_gain: 10,
    },
    SpellDef {
        name: "Teleport Self",
        level_required: 13,
        mana_required: 10,
        failure_chance: 45,
        exp_gain: 12,
    },
    SpellDef {
        name: "Remove Curse",
        level_required: 13,
        mana_required: 11,
        failure_chance: 55,
        exp_gain: 13,
    },
    SpellDef {
        name: "Frost Bolt",
        level_required: 15,
        mana_required: 12,
        failure_chance: 50,
        exp_gain: 15,
    },
    SpellDef {
        name: "Turn Stone to Mud",
        level_required: 15,
        mana_required: 13,
        failure_chance: 50,
        exp_gain: 15,
    },
    SpellDef {
        name: "Create Food",
        level_required: 17,
        mana_required: 17,
        failure_chance: 55,
        exp_gain: 15,
    },
    SpellDef {
        name: "Recharge Item I",
        level_required: 17,
        mana_required: 17,
        failure_chance: 90,
        exp_gain: 17,
    },
    SpellDef {
        name: "Sleep II",
        level_required: 21,
        mana_required: 17,
        failure_chance: 55,
        exp_gain: 17,
    },
    SpellDef {
        name: "Polymorph Other",
        level_required: 21,
        mana_required: 19,
        failure_chance: 60,
        exp_gain: 18,
    },
    SpellDef {
        name: "Identify",
        level_required: 23,
        mana_required: 25,
        failure_chance: 95,
        exp_gain: 20,
    },
    SpellDef {
        name: "Sleep III",
        level_required: 23,
        mana_required: 20,
        failure_chance: 60,
        exp_gain: 20,
    },
    SpellDef {
        name: "Fire Bolt",
        level_required: 25,
        mana_required: 20,
        failure_chance: 60,
        exp_gain: 20,
    },
    SpellDef {
        name: "Slow Monster",
        level_required: 25,
        mana_required: 21,
        failure_chance: 65,
        exp_gain: 20,
    },
    SpellDef {
        name: "Frost Ball",
        level_required: 27,
        mana_required: 21,
        failure_chance: 65,
        exp_gain: 22,
    },
    SpellDef {
        name: "Recharge Item II",
        level_required: 29,
        mana_required: 23,
        failure_chance: 95,
        exp_gain: 23,
    },
    SpellDef {
        name: "Teleport Other",
        level_required: 31,
        mana_required: 25,
        failure_chance: 70,
        exp_gain: 25,
    },
    SpellDef {
        name: "Haste Self",
        level_required: 33,
        mana_required: 25,
        failure_chance: 75,
        exp_gain: 38,
    },
    SpellDef {
        name: "Fire Ball",
        level_required: 35,
        mana_required: 25,
        failure_chance: 80,
        exp_gain: 50,
    },
    SpellDef {
        name: "Word of Destruction",
        level_required: 37,
        mana_required: 30,
        failure_chance: 95,
        exp_gain: 100,
    },
    SpellDef {
        name: "Genocide",
        level_required: 99,
        mana_required: 99,
        failure_chance: 0,
        exp_gain: 0,
    },
];

// Priest prayers table for Priest class (canonical data_player.cpp)
pub const PRIEST_CLASS_PRAYERS: [SpellDef; 31] = [
    SpellDef {
        name: "Detect Evil",
        level_required: 1,
        mana_required: 1,
        failure_chance: 10,
        exp_gain: 1,
    },
    SpellDef {
        name: "Cure Light Wounds",
        level_required: 1,
        mana_required: 2,
        failure_chance: 15,
        exp_gain: 1,
    },
    SpellDef {
        name: "Bless",
        level_required: 1,
        mana_required: 2,
        failure_chance: 20,
        exp_gain: 1,
    },
    SpellDef {
        name: "Remove Fear",
        level_required: 1,
        mana_required: 2,
        failure_chance: 25,
        exp_gain: 1,
    },
    SpellDef {
        name: "Call Light",
        level_required: 3,
        mana_required: 2,
        failure_chance: 25,
        exp_gain: 1,
    },
    SpellDef {
        name: "Find Traps",
        level_required: 3,
        mana_required: 3,
        failure_chance: 27,
        exp_gain: 2,
    },
    SpellDef {
        name: "Detect Doors/Stairs",
        level_required: 3,
        mana_required: 3,
        failure_chance: 27,
        exp_gain: 2,
    },
    SpellDef {
        name: "Slow Poison",
        level_required: 3,
        mana_required: 3,
        failure_chance: 28,
        exp_gain: 3,
    },
    SpellDef {
        name: "Blind Creature",
        level_required: 5,
        mana_required: 4,
        failure_chance: 29,
        exp_gain: 4,
    },
    SpellDef {
        name: "Portal",
        level_required: 5,
        mana_required: 4,
        failure_chance: 30,
        exp_gain: 5,
    },
    SpellDef {
        name: "Cure Medium Wounds",
        level_required: 5,
        mana_required: 4,
        failure_chance: 32,
        exp_gain: 5,
    },
    SpellDef {
        name: "Chant",
        level_required: 5,
        mana_required: 5,
        failure_chance: 34,
        exp_gain: 5,
    },
    SpellDef {
        name: "Sanctuary",
        level_required: 7,
        mana_required: 5,
        failure_chance: 36,
        exp_gain: 6,
    },
    SpellDef {
        name: "Create Food",
        level_required: 7,
        mana_required: 5,
        failure_chance: 38,
        exp_gain: 7,
    },
    SpellDef {
        name: "Remove Curse",
        level_required: 7,
        mana_required: 6,
        failure_chance: 38,
        exp_gain: 9,
    },
    SpellDef {
        name: "Resist Heat and Cold",
        level_required: 7,
        mana_required: 7,
        failure_chance: 38,
        exp_gain: 9,
    },
    SpellDef {
        name: "Neutralize Poison",
        level_required: 9,
        mana_required: 6,
        failure_chance: 38,
        exp_gain: 10,
    },
    SpellDef {
        name: "Orb of Draining",
        level_required: 9,
        mana_required: 7,
        failure_chance: 38,
        exp_gain: 10,
    },
    SpellDef {
        name: "Cure Serious Wounds",
        level_required: 9,
        mana_required: 7,
        failure_chance: 40,
        exp_gain: 10,
    },
    SpellDef {
        name: "Sense Invisible",
        level_required: 11,
        mana_required: 8,
        failure_chance: 42,
        exp_gain: 10,
    },
    SpellDef {
        name: "Protection from Evil",
        level_required: 11,
        mana_required: 8,
        failure_chance: 42,
        exp_gain: 12,
    },
    SpellDef {
        name: "Earthquake",
        level_required: 11,
        mana_required: 9,
        failure_chance: 55,
        exp_gain: 15,
    },
    SpellDef {
        name: "Sense Surroundings",
        level_required: 13,
        mana_required: 10,
        failure_chance: 45,
        exp_gain: 15,
    },
    SpellDef {
        name: "Cure Critical Wounds",
        level_required: 13,
        mana_required: 11,
        failure_chance: 45,
        exp_gain: 16,
    },
    SpellDef {
        name: "Turn Undead",
        level_required: 15,
        mana_required: 12,
        failure_chance: 50,
        exp_gain: 20,
    },
    SpellDef {
        name: "Prayer",
        level_required: 15,
        mana_required: 14,
        failure_chance: 50,
        exp_gain: 22,
    },
    SpellDef {
        name: "Dispel Undead",
        level_required: 17,
        mana_required: 14,
        failure_chance: 55,
        exp_gain: 32,
    },
    SpellDef {
        name: "Heal",
        level_required: 21,
        mana_required: 16,
        failure_chance: 60,
        exp_gain: 38,
    },
    SpellDef {
        name: "Dispel Evil",
        level_required: 25,
        mana_required: 20,
        failure_chance: 70,
        exp_gain: 75,
    },
    SpellDef {
        name: "Glyph of Warding",
        level_required: 33,
        mana_required: 24,
        failure_chance: 90,
        exp_gain: 125,
    },
    SpellDef {
        name: "Holy Word",
        level_required: 39,
        mana_required: 32,
        failure_chance: 80,
        exp_gain: 200,
    },
];

// Priest prayers table for Paladin class (canonical data_player.cpp)
pub const PALADIN_CLASS_PRAYERS: [SpellDef; 31] = [
    SpellDef {
        name: "Detect Evil",
        level_required: 1,
        mana_required: 1,
        failure_chance: 30,
        exp_gain: 1,
    },
    SpellDef {
        name: "Cure Light Wounds",
        level_required: 2,
        mana_required: 2,
        failure_chance: 35,
        exp_gain: 2,
    },
    SpellDef {
        name: "Bless",
        level_required: 3,
        mana_required: 3,
        failure_chance: 35,
        exp_gain: 3,
    },
    SpellDef {
        name: "Remove Fear",
        level_required: 5,
        mana_required: 3,
        failure_chance: 35,
        exp_gain: 5,
    },
    SpellDef {
        name: "Call Light",
        level_required: 5,
        mana_required: 4,
        failure_chance: 35,
        exp_gain: 5,
    },
    SpellDef {
        name: "Find Traps",
        level_required: 7,
        mana_required: 5,
        failure_chance: 40,
        exp_gain: 6,
    },
    SpellDef {
        name: "Detect Doors/Stairs",
        level_required: 7,
        mana_required: 5,
        failure_chance: 40,
        exp_gain: 6,
    },
    SpellDef {
        name: "Slow Poison",
        level_required: 9,
        mana_required: 7,
        failure_chance: 40,
        exp_gain: 7,
    },
    SpellDef {
        name: "Blind Creature",
        level_required: 9,
        mana_required: 7,
        failure_chance: 40,
        exp_gain: 8,
    },
    SpellDef {
        name: "Portal",
        level_required: 9,
        mana_required: 8,
        failure_chance: 40,
        exp_gain: 8,
    },
    SpellDef {
        name: "Cure Medium Wounds",
        level_required: 11,
        mana_required: 9,
        failure_chance: 40,
        exp_gain: 10,
    },
    SpellDef {
        name: "Chant",
        level_required: 11,
        mana_required: 10,
        failure_chance: 45,
        exp_gain: 10,
    },
    SpellDef {
        name: "Sanctuary",
        level_required: 11,
        mana_required: 10,
        failure_chance: 45,
        exp_gain: 10,
    },
    SpellDef {
        name: "Create Food",
        level_required: 13,
        mana_required: 10,
        failure_chance: 45,
        exp_gain: 12,
    },
    SpellDef {
        name: "Remove Curse",
        level_required: 13,
        mana_required: 11,
        failure_chance: 45,
        exp_gain: 13,
    },
    SpellDef {
        name: "Resist Heat and Cold",
        level_required: 15,
        mana_required: 13,
        failure_chance: 45,
        exp_gain: 15,
    },
    SpellDef {
        name: "Neutralize Poison",
        level_required: 15,
        mana_required: 15,
        failure_chance: 50,
        exp_gain: 15,
    },
    SpellDef {
        name: "Orb of Draining",
        level_required: 17,
        mana_required: 15,
        failure_chance: 50,
        exp_gain: 17,
    },
    SpellDef {
        name: "Cure Serious Wounds",
        level_required: 17,
        mana_required: 15,
        failure_chance: 50,
        exp_gain: 18,
    },
    SpellDef {
        name: "Sense Invisible",
        level_required: 19,
        mana_required: 15,
        failure_chance: 50,
        exp_gain: 19,
    },
    SpellDef {
        name: "Protection from Evil",
        level_required: 19,
        mana_required: 15,
        failure_chance: 50,
        exp_gain: 19,
    },
    SpellDef {
        name: "Earthquake",
        level_required: 21,
        mana_required: 17,
        failure_chance: 50,
        exp_gain: 20,
    },
    SpellDef {
        name: "Sense Surroundings",
        level_required: 23,
        mana_required: 17,
        failure_chance: 50,
        exp_gain: 20,
    },
    SpellDef {
        name: "Cure Critical Wounds",
        level_required: 25,
        mana_required: 20,
        failure_chance: 50,
        exp_gain: 20,
    },
    SpellDef {
        name: "Turn Undead",
        level_required: 27,
        mana_required: 21,
        failure_chance: 50,
        exp_gain: 22,
    },
    SpellDef {
        name: "Prayer",
        level_required: 29,
        mana_required: 22,
        failure_chance: 50,
        exp_gain: 24,
    },
    SpellDef {
        name: "Dispel Undead",
        level_required: 31,
        mana_required: 24,
        failure_chance: 60,
        exp_gain: 25,
    },
    SpellDef {
        name: "Heal",
        level_required: 33,
        mana_required: 28,
        failure_chance: 60,
        exp_gain: 31,
    },
    SpellDef {
        name: "Dispel Evil",
        level_required: 35,
        mana_required: 32,
        failure_chance: 70,
        exp_gain: 38,
    },
    SpellDef {
        name: "Glyph of Warding",
        level_required: 37,
        mana_required: 36,
        failure_chance: 90,
        exp_gain: 50,
    },
    SpellDef {
        name: "Holy Word",
        level_required: 39,
        mana_required: 38,
        failure_chance: 90,
        exp_gain: 100,
    },
];

pub fn get_mage_spells(class: Class) -> &'static [SpellDef; 31] {
    match class {
        Class::Rogue => &ROGUE_CLASS_SPELLS,
        Class::Ranger => &RANGER_CLASS_SPELLS,
        _ => &MAGE_CLASS_SPELLS,
    }
}

pub fn get_priest_prayers(class: Class) -> &'static [SpellDef; 31] {
    match class {
        Class::Paladin => &PALADIN_CLASS_PRAYERS,
        _ => &PRIEST_CLASS_PRAYERS,
    }
}

pub fn get_spell_def(class: Class, spell_idx: usize, is_prayer: bool) -> Option<&'static SpellDef> {
    if spell_idx >= 31 {
        return None;
    }
    if is_prayer {
        Some(&get_priest_prayers(class)[spell_idx])
    } else {
        Some(&get_mage_spells(class)[spell_idx])
    }
}

/// Canonical Stat adjustment for INT/WIS (player_stats.cpp:116)
pub fn stat_adjustment(stat_value: i16) -> i32 {
    if stat_value > 117 {
        7
    } else if stat_value > 107 {
        6
    } else if stat_value > 87 {
        5
    } else if stat_value > 67 {
        4
    } else if stat_value > 17 {
        3
    } else if stat_value > 14 {
        2
    } else if stat_value > 7 {
        1
    } else {
        0
    }
}

/// First level where a class can use magic/prayers (canonical classes[] in data_player.cpp)
pub fn min_level_for_spells(class: Class) -> u32 {
    match class {
        Class::Warrior => 99,
        Class::Mage => 1,
        Class::Priest => 1,
        Class::Rogue => 5,
        Class::Ranger => 3,
        Class::Paladin => 1,
    }
}

/// Number of spells allowed according to level, class and relevant stat (INT/WIS)
/// Canonical Umoria player.cpp:1474
pub fn number_of_spells_allowed(level: u32, class: Class, stat_value: i16) -> u32 {
    let min_lvl = min_level_for_spells(class);
    if level < min_lvl {
        return 0;
    }
    let levels = (level - min_lvl + 1) as i32;
    let adj = stat_adjustment(stat_value);
    let allowed = match adj {
        1..=3 => levels,
        4 | 5 => 3 * levels / 2,
        6 => 2 * levels,
        7 => 5 * levels / 2,
        _ => 0,
    };
    allowed.max(0) as u32
}

/// Calculate maximum mana according to level, class, stat, and whether at least 1 spell is learned
/// Canonical Umoria player.cpp:981 & 1004
pub fn calculate_max_mana(level: u32, class: Class, stat_value: i16, spells_learnt: u32) -> i32 {
    if spells_learnt == 0 {
        return 0;
    }
    let min_lvl = min_level_for_spells(class);
    if level < min_lvl {
        return 0;
    }
    let levels = (level - min_lvl + 1) as i32;
    let adj = stat_adjustment(stat_value);
    let mut mana = match adj {
        1 | 2 => levels,
        3 => 3 * levels / 2,
        4 => 2 * levels,
        5 => 5 * levels / 2,
        6 => 3 * levels,
        7 => 4 * levels,
        _ => 0,
    };
    if mana > 0 {
        mana += 1;
    }
    mana
}

/// Calculates spell chance of failure (5% to 95%)
/// Canonical Umoria mage_spells.cpp:270
pub fn spell_chance_of_success(
    player_level: u32,
    _player_class: Class,
    stat_val: i16,
    current_mana: i32,
    spell: &SpellDef,
) -> u32 {
    let mut chance =
        spell.failure_chance as i32 - 3 * (player_level as i32 - spell.level_required as i32);
    let stat_adj = stat_adjustment(stat_val);
    chance -= 3 * (stat_adj - 1);
    if spell.mana_required as i32 > current_mana {
        chance += 5 * (spell.mana_required as i32 - current_mana);
    }
    chance.clamp(5, 95) as u32
}

/// Returns whether a spell requires a direction vector from the player
pub fn is_directional_spell(spell_idx: usize, is_prayer: bool) -> bool {
    if is_prayer {
        // Blind Creature (8), Orb of Draining (17)
        matches!(spell_idx, 8 | 17)
    } else {
        // Magic Missile (0), Stinking Cloud (6), Confusion (7), Lightning Bolt (8),
        // Sleep I (10), Frost Bolt (14), Wall to Mud (15), Polymorph Other (19),
        // Fire Bolt (22), Slow Monster (23), Frost Ball (24), Teleport Other (26), Fire Ball (28)
        matches!(
            spell_idx,
            0 | 6 | 7 | 8 | 10 | 14 | 15 | 19 | 22 | 23 | 24 | 26 | 28
        )
    }
}

/// Canonical Book definitions
#[derive(Debug, Clone)]
pub struct BookDef {
    pub name: &'static str,
    pub flags: u32,
    pub cost: u32,
    pub weight: u32,
    pub is_prayer: bool,
}

pub const MAGE_BOOKS: [BookDef; 4] = [
    BookDef {
        name: "Mage Spellbook [Beginners-Magick]",
        flags: 0x0000007F,
        cost: 25,
        weight: 30,
        is_prayer: false,
    },
    BookDef {
        name: "Mage Spellbook [Magick I]",
        flags: 0x0000FF80,
        cost: 100,
        weight: 30,
        is_prayer: false,
    },
    BookDef {
        name: "Mage Spellbook [Magick II]",
        flags: 0x00FF0000,
        cost: 400,
        weight: 30,
        is_prayer: false,
    },
    BookDef {
        name: "Mage Spellbook [The Mages' Guide to Power]",
        flags: 0x7F000000,
        cost: 800,
        weight: 30,
        is_prayer: false,
    },
];

pub const PRAYER_BOOKS: [BookDef; 4] = [
    BookDef {
        name: "Priest Prayerbook [Beginners Handbook]",
        flags: 0x000000FF,
        cost: 25,
        weight: 30,
        is_prayer: true,
    },
    BookDef {
        name: "Priest Prayerbook [Words of Wisdom]",
        flags: 0x0000FF00,
        cost: 100,
        weight: 30,
        is_prayer: true,
    },
    BookDef {
        name: "Priest Prayerbook [Chants and Blessings]",
        flags: 0x01FF0000,
        cost: 400,
        weight: 30,
        is_prayer: true,
    },
    BookDef {
        name: "Priest Prayerbook [Exorcisms and Dispellings]",
        flags: 0x7E000000,
        cost: 800,
        weight: 30,
        is_prayer: true,
    },
];

pub fn create_book(book_idx: usize, is_prayer: bool) -> Item {
    let def = if is_prayer {
        &PRAYER_BOOKS[book_idx.min(3)]
    } else {
        &MAGE_BOOKS[book_idx.min(3)]
    };
    let item_type = if is_prayer {
        ItemType::PrayerBook {
            spell_flags: def.flags,
        }
    } else {
        ItemType::MagicBook {
            spell_flags: def.flags,
        }
    };
    Item::new(def.name, 1, def.weight, item_type)
}

/// Returns list of spell indices 0..30 that are contained within a book's flags mask
pub fn get_spells_in_book(flags: u32) -> Vec<usize> {
    let mut list = Vec::new();
    for i in 0..31 {
        if (flags & (1 << i)) != 0 {
            list.push(i);
        }
    }
    list
}

/// Applies mana cost and checks for fainting / fatigue / permanent CON drain if player over-casts
/// Canonical Umoria mage_spells.cpp:251 & player_pray.cpp:256
pub fn apply_spell_mana_and_fatigue<R: rand::Rng>(
    player: &mut Player,
    mana_required: u32,
    is_prayer: bool,
    rng: &mut R,
    status_msg: &mut String,
) {
    if mana_required as i32 > player.mana {
        let diff = mana_required as i32 - player.mana;
        if is_prayer {
            status_msg.push_str(" You faint from fatigue!");
        } else {
            status_msg.push_str(" You faint from the effort!");
        }
        let paralysis_turns = rng.gen_range(1..=(5 * diff).max(1)) as i16;
        player.flags.paralysis += paralysis_turns;
        player.mana = 0;
        if rng.gen_range(1..=3) == 1 {
            status_msg.push_str(" You have damaged your health!");
            player.drain_stat(4, rng);
        }
    } else {
        player.mana -= mana_required as i32;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cast_spell_bolt(
    spell_name: &str,
    spell_type: MagicSpellType,
    dx: isize,
    dy: isize,
    base_damage: i32,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    status_msg: &mut String,
) {
    let (harm_type, weapon_type) = spell_type.flags();
    let mut cx = player.x as isize;
    let mut cy = player.y as isize;
    let mut hit = false;
    for _ in 0..20 {
        cx += dx;
        cy += dy;
        if cx <= 0
            || cx >= (level.width - 1) as isize
            || cy <= 0
            || cy >= (level.height - 1) as isize
        {
            break;
        }
        let (ux, uy) = (cx as usize, cy as usize);
        if let Some(tile) = level.get_tile(ux, uy)
            && matches!(
                tile.tile_type,
                TileType::Wall
                    | TileType::SecretDoor
                    | TileType::DoorClosed { .. }
                    | TileType::MagmaVein { .. }
                    | TileType::QuartzVein { .. }
                    | TileType::Rubble
            )
        {
            *status_msg = format!("Your {} strikes a wall!", spell_name);
            hit = true;
            break;
        }
        if let Some(m_idx) = monsters.iter().position(|m| m.x == ux && m.y == uy) {
            let m_name = monsters[m_idx].name.clone();
            let exp = monsters[m_idx].experience_reward;
            let m_lvl = monsters[m_idx].level;
            let is_win = monsters[m_idx].is_win_creature();

            let mut damage = base_damage;
            if (harm_type & monsters[m_idx].defenses()) != 0 {
                damage *= 2;
            } else if (weapon_type & monsters[m_idx].spells()) != 0 {
                damage /= 4;
            }

            *status_msg = format!("Your {} strikes {}.", spell_name, m_name);
            monsters[m_idx].was_attacked = true;
            monsters[m_idx].asleep = 0;
            if monsters[m_idx].take_damage(damage) {
                status_msg.push_str(&format!(" {} dies in a fit of agony.", m_name));
                player.gain_kill_experience(exp, m_lvl);
                if is_win {
                    player.balrog_killed = true;
                    player.total_winner = true;
                    status_msg.push_str(" *** CONGRATULATIONS *** You have won the game!");
                }
                if level.depth == 0 {
                    player.killed_town_npcs += 1;
                }
                monsters.remove(m_idx);
            } else if damage > 0 {
                status_msg.push_str(&format!(" {} screams in agony.", m_name));
            }
            hit = true;
            break;
        }
    }
    if !hit {
        *status_msg = format!("Your {} dissipates into the dark.", spell_name);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cast_spell_ball(
    spell_name: &str,
    spell_type: MagicSpellType,
    dx: isize,
    dy: isize,
    radius: usize,
    base_damage: i32,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    status_msg: &mut String,
) {
    let (harm_type, weapon_type) = spell_type.flags();
    let mut cx = player.x as isize;
    let mut cy = player.y as isize;
    let mut impact = (player.x, player.y);
    for _ in 0..20 {
        cx += dx;
        cy += dy;
        if cx <= 0
            || cx >= (level.width - 1) as isize
            || cy <= 0
            || cy >= (level.height - 1) as isize
        {
            break;
        }
        let (ux, uy) = (cx as usize, cy as usize);
        impact = (ux, uy);
        if let Some(tile) = level.get_tile(ux, uy)
            && matches!(
                tile.tile_type,
                TileType::Wall
                    | TileType::SecretDoor
                    | TileType::DoorClosed { .. }
                    | TileType::MagmaVein { .. }
                    | TileType::QuartzVein { .. }
                    | TileType::Rubble
            )
        {
            break;
        }
        if monsters.iter().any(|m| m.x == ux && m.y == uy) {
            break;
        }
    }

    *status_msg = format!("Your {} detonates in a roaring explosion!", spell_name);

    let mut killed = Vec::new();
    let mut hit_count = 0;
    for (m_idx, m) in monsters.iter_mut().enumerate() {
        let dy = (impact.1 as isize - m.y as isize).abs();
        let dx = (impact.0 as isize - m.x as isize).abs();
        let dist = ((dy + dx) * 2 - dx.min(dy)) / 2;
        if dist <= radius as isize {
            hit_count += 1;
            let mut damage = base_damage;
            if (harm_type & m.defenses()) != 0 {
                damage *= 2;
            } else if (weapon_type & m.spells()) != 0 {
                damage /= 4;
            }
            let decayed_damage = damage / (dist as i32 + 1);
            m.was_attacked = true;
            m.asleep = 0;
            if m.take_damage(decayed_damage) {
                killed.push(m_idx);
            }
        }
    }
    for &idx in killed.iter().rev() {
        let exp = monsters[idx].experience_reward;
        let m_lvl = monsters[idx].level;
        let is_win = monsters[idx].is_win_creature();
        player.gain_kill_experience(exp, m_lvl);
        if is_win {
            player.balrog_killed = true;
            player.total_winner = true;
        }
        if level.depth == 0 {
            player.killed_town_npcs += 1;
        }
        monsters.remove(idx);
    }
    if !killed.is_empty() {
        status_msg.push_str(&format!(
            " {} creatures perished in the blast!",
            killed.len()
        ));
        if player.total_winner {
            status_msg.push_str(" *** CONGRATULATIONS *** You have won the game!");
        }
    } else if hit_count > 0 {
        status_msg.push_str(&format!(
            " {} creatures were caught in the blast!",
            hit_count
        ));
    }

    let p_dy = (player.y as isize - impact.1 as isize).abs();
    let p_dx = (player.x as isize - impact.0 as isize).abs();
    let p_dist = ((p_dy + p_dx) * 2 - p_dx.min(p_dy)) / 2;
    if p_dist <= radius as isize {
        let p_dmg = (base_damage / (p_dist as i32 + 1)) / 2;
        player.hp -= p_dmg;
        status_msg.push_str(" You are caught in your own blast!");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_directional_spell<R: rand::Rng>(
    spell_idx: usize,
    is_prayer: bool,
    dx: isize,
    dy: isize,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    rng: &mut R,
    status_msg: &mut String,
) {
    if is_prayer {
        match spell_idx {
            8 => {
                // Blind Creature
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        m.confused += rng.gen_range(5..=15);
                        m.was_attacked = true;
                        m.asleep = 0;
                        *status_msg = format!("{} is blinded and stumbles in confusion!", m.name);
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your prayer of blinding has no effect on empty air.".to_string();
                }
            }
            17 => {
                // Orb of Draining
                let dmg = (Dice::new(3, 6).roll(rng) + player.level) as i32;
                cast_spell_ball(
                    "Orb of Draining",
                    MagicSpellType::HolyOrb,
                    dx,
                    dy,
                    2,
                    dmg,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            _ => {
                *status_msg = "Nothing happens.".to_string();
            }
        }
    } else {
        match spell_idx {
            0 => {
                // Magic Missile (2d6)
                let dmg = Dice::new(2, 6).roll(rng) as i32;
                cast_spell_bolt(
                    "Magic Missile",
                    MagicSpellType::MagicMissile,
                    dx,
                    dy,
                    dmg,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            6 => {
                // Stinking Cloud (12 dmg ball)
                cast_spell_ball(
                    "Stinking Cloud",
                    MagicSpellType::PoisonGas,
                    dx,
                    dy,
                    2,
                    12,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            7 => {
                // Confusion
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        m.confused += rng.gen_range(5..=15);
                        m.was_attacked = true;
                        m.asleep = 0;
                        *status_msg = format!("{} appears confused!", m.name);
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your beam of confusion strikes nothing.".to_string();
                }
            }
            8 => {
                // Lightning Bolt (4d8)
                let dmg = Dice::new(4, 8).roll(rng) as i32;
                cast_spell_bolt(
                    "Lightning Bolt",
                    MagicSpellType::Lightning,
                    dx,
                    dy,
                    dmg,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            10 => {
                // Sleep I
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        m.asleep += rng.gen_range(5..=15);
                        *status_msg = format!("{} falls into a deep slumber!", m.name);
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your sleeping spell dissipated into the air.".to_string();
                }
            }
            14 => {
                // Frost Bolt (6d8)
                let dmg = Dice::new(6, 8).roll(rng) as i32;
                cast_spell_bolt(
                    "Frost Bolt",
                    MagicSpellType::Frost,
                    dx,
                    dy,
                    dmg,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            15 => {
                // Turn Stone to Mud
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(tile) = level.get_tile_mut(ux, uy)
                        && matches!(
                            tile.tile_type,
                            TileType::Wall
                                | TileType::MagmaVein { .. }
                                | TileType::QuartzVein { .. }
                                | TileType::Rubble
                        )
                    {
                        tile.tile_type = TileType::Floor;
                        *status_msg = "The stone dissolves into mud and flows away!".to_string();
                        hit = true;
                        break;
                    }
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        let dmg = 20;
                        m.was_attacked = true;
                        m.asleep = 0;
                        if m.take_damage(dmg) {
                            *status_msg = format!("{} turns to mud and collapses!", m.name);
                        } else {
                            *status_msg = format!("{} is eroded for {} damage!", m.name, dmg);
                        }
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Nothing happens.".to_string();
                }
            }
            19 => {
                // Polymorph Other
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        if m.is_win_creature() {
                            *status_msg = format!("The {} is unaffected.", m.name);
                        } else if rng.gen_range(1..=50) > m.level as u32 {
                            let (mx, my) = (m.x, m.y);
                            let new_id = crate::entity::monster_data::get_monster_for_level(
                                level.depth,
                                rng,
                            );
                            let old_name = m.name.clone();
                            *m = Monster::from_creature_id(new_id, mx, my, false, rng);
                            *status_msg = format!("{} mutates into a {}!", old_name, m.name);
                        } else {
                            *status_msg = format!("The {} is unaffected.", m.name);
                        }
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your beam of transmutation hits nothing.".to_string();
                }
            }
            22 => {
                // Fire Bolt (9d8)
                let dmg = Dice::new(9, 8).roll(rng) as i32;
                cast_spell_bolt(
                    "Fire Bolt",
                    MagicSpellType::Fire,
                    dx,
                    dy,
                    dmg,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            23 => {
                // Slow Monster
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        m.stunned += rng.gen_range(5..=15);
                        m.was_attacked = true;
                        *status_msg = format!("{} starts moving much more slowly!", m.name);
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your slowing spell strikes nothing.".to_string();
                }
            }
            24 => {
                // Frost Ball (48 dmg ball)
                cast_spell_ball(
                    "Frost Ball",
                    MagicSpellType::Frost,
                    dx,
                    dy,
                    2,
                    48,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            26 => {
                // Teleport Other
                let mut cx = player.x as isize;
                let mut cy = player.y as isize;
                let mut hit = false;
                for _ in 0..20 {
                    cx += dx;
                    cy += dy;
                    if cx <= 0
                        || cx >= (level.width - 1) as isize
                        || cy <= 0
                        || cy >= (level.height - 1) as isize
                    {
                        break;
                    }
                    let (ux, uy) = (cx as usize, cy as usize);
                    if let Some(m) = monsters.iter_mut().find(|m| m.x == ux && m.y == uy) {
                        if m.is_win_creature() {
                            *status_msg = format!("The {} is unaffected.", m.name);
                        } else {
                            let dest = level.find_random_floor_tile();
                            m.x = dest.0;
                            m.y = dest.1;
                            m.was_attacked = true;
                            *status_msg = format!("{} vanishes into thin air!", m.name);
                        }
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    *status_msg = "Your teleportation beam strikes nothing.".to_string();
                }
            }
            28 => {
                // Fire Ball (72 dmg ball)
                cast_spell_ball(
                    "Fire Ball",
                    MagicSpellType::Fire,
                    dx,
                    dy,
                    2,
                    72,
                    player,
                    level,
                    monsters,
                    status_msg,
                );
            }
            _ => {
                *status_msg = "Nothing happens.".to_string();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_non_directional_spell<R: rand::Rng>(
    spell_idx: usize,
    is_prayer: bool,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    rng: &mut R,
    status_msg: &mut String,
) {
    if is_prayer {
        match spell_idx {
            0 => {
                // Detect Evil
                for m in monsters.iter() {
                    if let Some(tile) = level.get_tile_mut(m.x, m.y) {
                        tile.remembered = true;
                    }
                }
                *status_msg = format!("You sense {} dark presences!", monsters.len());
            }
            1 => {
                // Cure Light Wounds (3d3)
                let heal = Dice::new(3, 3).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                *status_msg = format!("Divine radiance heals you for {} HP.", heal);
            }
            2 => {
                // Bless
                player.flags.blessed += rng.gen_range(12..=24);
                *status_msg = "You feel the righteous blessing of your deity!".to_string();
            }
            3 => {
                // Remove Fear
                player.flags.afraid = 0;
                *status_msg = "Your spirit is emboldened; fear leaves your heart.".to_string();
            }
            4 => {
                // Call Light
                level.illuminate_area(player.x, player.y, 3);
                *status_msg = "Brilliant divine light illuminates the area.".to_string();
            }
            5 => {
                // Find Traps
                let found = level.detect_traps_near(player.x, player.y, 15);
                *status_msg = format!("You sense {} hidden traps in the vicinity.", found);
            }
            6 => {
                // Detect Doors/Stairs
                let found = level.detect_doors_and_stairs_near(player.x, player.y, 15);
                *status_msg = format!("You sense {} doors/stairs in the vicinity.", found);
            }
            7 => {
                // Slow Poison
                player.flags.poisoned /= 2;
                *status_msg = "The venom in your veins slows down.".to_string();
            }
            9 => {
                // Portal
                let max_dist = (player.level * 3).max(5) as usize;
                let dest = level.find_random_floor_tile_near(player.x, player.y, max_dist);
                player.move_to(dest.0, dest.1);
                *status_msg = "You are drawn into a shimmering celestial portal!".to_string();
            }
            10 => {
                // Cure Medium Wounds (4d4)
                let heal = Dice::new(4, 4).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                *status_msg = format!("Divine favor restores {} HP.", heal);
            }
            11 => {
                // Chant
                player.flags.blessed += rng.gen_range(24..=48);
                *status_msg =
                    "You chant sacred verses and feel divine favor descend upon you!".to_string();
            }
            12 => {
                // Sanctuary
                let mut count = 0;
                for m in monsters.iter_mut() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 5 {
                        m.asleep += rng.gen_range(5..=15);
                        count += 1;
                    }
                }
                *status_msg = format!("A wave of peaceful sanctuary subdues {} foes!", count);
            }
            13 => {
                // Create Food
                let food = Item::new(
                    "Ration of Food",
                    1,
                    20,
                    ItemType::Food { nutrition: 5000 },
                );
                if player.add_item_to_inventory(food.clone()) {
                    *status_msg = "Manna falls from heaven into your pack!".to_string();
                } else {
                    level.items.push(crate::dungeon::FloorItem {
                        x: player.x,
                        y: player.y,
                        item: food,
                    });
                    *status_msg =
                        "Your pack is full; manna falls from heaven to the floor.".to_string();
                }
            }
            14 => {
                // Remove Curse
                for item in &mut player.equipment {
                    item.is_cursed = false;
                }
                *status_msg = "Holy benediction lifts the curses on your equipment.".to_string();
            }
            15 => {
                // Resist Heat and Cold
                player.flags.heat_resistance += rng.gen_range(10..=20);
                player.flags.cold_resistance += rng.gen_range(10..=20);
                *status_msg = "A protective aura shields you from heat and cold.".to_string();
            }
            16 => {
                // Neutralize Poison
                player.flags.poisoned = 0;
                *status_msg = "All impurities and poison are cleansed from your body!".to_string();
            }
            18 => {
                // Cure Serious Wounds (8d4)
                let heal = Dice::new(8, 4).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                *status_msg = format!("Divine power mends serious wounds! Restored {} HP.", heal);
            }
            19 => {
                // Sense Invisible
                player.flags.detect_invisible += rng.gen_range(24..=48);
                player.flags.see_invisible = true;
                *status_msg = "Your eyes glow with insight; you see the unseen!".to_string();
            }
            20 => {
                // Protection from Evil
                player.flags.protect_evil += rng.gen_range(1..=25) + 3 * player.level as i16;
                *status_msg = "A divine barrier rises to repel the forces of darkness!".to_string();
            }
            21 => {
                // Earthquake
                level.cause_earthquake(player.x, player.y, 10, rng);
                *status_msg = "The ground shakes violently in righteous wrath!".to_string();
            }
            22 => {
                // Sense Surroundings
                level.map_area(player.x, player.y, 25);
                *status_msg = "A divine vision reveals the layout of the surroundings.".to_string();
            }
            23 => {
                // Cure Critical Wounds (16d4)
                let heal = Dice::new(16, 4).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                *status_msg = format!(
                    "Miraculous healing surges through you! Restored {} HP.",
                    heal
                );
            }
            24 => {
                // Turn Undead
                let mut count = 0;
                for m in monsters.iter_mut() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.stunned += rng.gen_range(10..=20);
                        count += 1;
                    }
                }
                *status_msg = format!("You command {} undead to flee in terror!", count);
            }
            25 => {
                // Prayer
                player.flags.blessed += rng.gen_range(48..=96);
                *status_msg =
                    "Your prayers resonate across the heavens; blessing surges!".to_string();
            }
            26 => {
                // Dispel Undead
                let dmg = (3 * player.level) as i32;
                let mut killed = Vec::new();
                for (idx, m) in monsters.iter_mut().enumerate() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.was_attacked = true;
                        m.asleep = 0;
                        if m.take_damage(dmg) {
                            killed.push(idx);
                        }
                    }
                }
                for &idx in killed.iter().rev() {
                    let exp = monsters[idx].experience_reward;
                    player.add_experience(exp);
                    if level.depth == 0 {
                        player.killed_town_npcs += 1;
                    }
                    monsters.remove(idx);
                }
                *status_msg = format!("Holy light scours the undead! {} destroyed.", killed.len());
            }
            27 => {
                // Heal (200 HP)
                player.hp = (player.hp + 200).min(player.max_hp);
                *status_msg = "Pure life force heals you completely (200 HP)!".to_string();
            }
            28 => {
                // Dispel Evil
                let dmg = (3 * player.level) as i32;
                let mut killed = Vec::new();
                for (idx, m) in monsters.iter_mut().enumerate() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.was_attacked = true;
                        m.asleep = 0;
                        if m.take_damage(dmg) {
                            killed.push(idx);
                        }
                    }
                }
                for &idx in killed.iter().rev() {
                    let exp = monsters[idx].experience_reward;
                    player.add_experience(exp);
                    if level.depth == 0 {
                        player.killed_town_npcs += 1;
                    }
                    monsters.remove(idx);
                }
                *status_msg = format!(
                    "Holy wrath burns into the darkness! {} evil foes destroyed.",
                    killed.len()
                );
            }
            29 => {
                // Glyph of Warding
                level.place_glyph_of_warding(player.x, player.y);
                *status_msg =
                    "You trace a glowing holy Glyph of Warding upon the floor!".to_string();
            }
            30 => {
                // Holy Word
                player.hp = (player.hp + 1000).min(player.max_hp);
                player.flags.afraid = 0;
                player.flags.poisoned = 0;
                player.restore_all_stats();
                player.flags.invulnerability = player.flags.invulnerability.max(3) + 1;
                let dmg = (4 * player.level) as i32;
                let mut killed = Vec::new();
                for (idx, m) in monsters.iter_mut().enumerate() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.was_attacked = true;
                        m.asleep = 0;
                        if m.take_damage(dmg) {
                            killed.push(idx);
                        }
                    }
                }
                for &idx in killed.iter().rev() {
                    let exp = monsters[idx].experience_reward;
                    player.add_experience(exp);
                    if level.depth == 0 {
                        player.killed_town_npcs += 1;
                    }
                    monsters.remove(idx);
                }
                *status_msg = format!(
                    "You utter a Word of divine power! Foes perish and your body is restored! ({} destroyed)",
                    killed.len()
                );
            }
            _ => {
                *status_msg = "Nothing happens.".to_string();
            }
        }
    } else {
        match spell_idx {
            1 => {
                // Detect Monsters
                for m in monsters.iter() {
                    if let Some(tile) = level.get_tile_mut(m.x, m.y) {
                        tile.remembered = true;
                    }
                }
                *status_msg = format!("You sense {} monsters!", monsters.len());
            }
            2 => {
                // Phase Door
                let dest = level.find_random_floor_tile_near(player.x, player.y, 10);
                player.move_to(dest.0, dest.1);
                *status_msg = "You cast Phase Door and warp!".to_string();
            }
            3 => {
                // Light Area
                level.illuminate_area(player.x, player.y, 3);
                *status_msg = "Brilliant light flashes and illuminates the area!".to_string();
            }
            4 => {
                // Cure Light Wounds (4d4)
                let heal = Dice::new(4, 4).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                *status_msg = format!("You feel a little better. Healed {} HP.", heal);
            }
            5 => {
                // Find Hidden Traps/Doors
                let traps = level.detect_traps_near(player.x, player.y, 15);
                let doors = level.detect_doors_and_stairs_near(player.x, player.y, 15);
                *status_msg = format!(
                    "You sense {} traps and {} secret doors nearby.",
                    traps, doors
                );
            }
            9 => {
                // Trap/Door Destruction
                let count = level.destroy_adjacent_doors_and_traps(player.x, player.y);
                *status_msg = format!(
                    "There is a sudden burst of energy! Destroyed {} doors/traps.",
                    count
                );
            }
            11 => {
                // Cure Poison
                player.flags.poisoned = 0;
                *status_msg = "You feel the poison leave your veins.".to_string();
            }
            12 => {
                // Teleport Self
                let dest = level.find_random_floor_tile();
                player.move_to(dest.0, dest.1);
                *status_msg = "You teleport yourself across space!".to_string();
            }
            13 => {
                // Remove Curse
                for item in &mut player.equipment {
                    item.is_cursed = false;
                }
                *status_msg =
                    "You feel a malignant aura dissipate from your equipment.".to_string();
            }
            16 => {
                // Create Food
                let food = Item::new(
                    "Ration of Food",
                    1,
                    20,
                    ItemType::Food { nutrition: 5000 },
                );
                if player.add_item_to_inventory(food.clone()) {
                    *status_msg = "A nutritious ration of food appears in your pack!".to_string();
                } else {
                    level.items.push(crate::dungeon::FloorItem {
                        x: player.x,
                        y: player.y,
                        item: food,
                    });
                    *status_msg =
                        "Your pack is full; a nutritious ration of food drops to the floor."
                            .to_string();
                }
            }
            17 => {
                // Recharge Item I
                let (_, rmsg) = spell_recharge_item(player, 20, rng);
                *status_msg = rmsg;
            }
            18 => {
                // Sleep II
                let mut count = 0;
                for m in monsters.iter_mut() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.asleep += rng.gen_range(5..=15);
                        count += 1;
                    }
                }
                *status_msg = format!("A drowsy fog blankets {} surrounding creatures!", count);
            }
            20 => {
                // Identify
                let mut identified_any = false;
                for item in &mut player.inventory {
                    if !item.identified {
                        item.identified = true;
                        identified_any = true;
                        break;
                    }
                }
                *status_msg = if identified_any {
                    "Magical runes reveal the true nature of your gear!".to_string()
                } else {
                    "All your items are already identified.".to_string()
                };
            }
            21 => {
                // Sleep III
                let count = monsters.len();
                for m in monsters.iter_mut() {
                    m.asleep += rng.gen_range(10..=25);
                }
                *status_msg = format!(
                    "A deep magical slumber falls across {} monsters on this level!",
                    count
                );
            }
            25 => {
                // Recharge Item II
                let (_, rmsg) = spell_recharge_item(player, 60, rng);
                *status_msg = rmsg;
            }
            27 => {
                // Haste Self
                player.flags.fast += rng.gen_range(10..=20) + player.level as i16;
                *status_msg = "You start moving much faster!".to_string();
            }
            29 => {
                // Word of Destruction
                level.destroy_area(player.x, player.y, 12);
                *status_msg =
                    "The dungeon violently dissolves into raw stone around you!".to_string();
            }
            30 => {
                // Genocide
                let (count, gmsg) = spell_genocide('o', player, monsters);
                *status_msg = format!("{} ({} destroyed)", gmsg, count);
            }
            _ => {
                *status_msg = "Nothing happens.".to_string();
            }
        }
    }
}

pub fn spell_surround_player_with_doors(player: &Player, level: &mut DungeonLevel) -> bool {
    let mut created = false;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = player.x as isize + dx;
            let ty = player.y as isize + dy;
            if tx > 0
                && ty > 0
                && (tx as usize) < (level.width - 1)
                && (ty as usize) < (level.height - 1)
            {
                let (ux, uy) = (tx as usize, ty as usize);
                if let Some(tile) = level.get_tile_mut(ux, uy)
                    && tile.tile_type == TileType::Floor
                {
                    tile.tile_type = TileType::DoorClosed { spikes: 0 };
                    tile.remembered = true;
                    created = true;
                }
            }
        }
    }
    created
}

pub fn spell_surround_player_with_traps<R: rand::Rng>(
    player: &Player,
    level: &mut DungeonLevel,
    rng: &mut R,
) -> bool {
    let mut created = false;
    let trap_types = TrapType::ALL;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = player.x as isize + dx;
            let ty = player.y as isize + dy;
            if tx > 0
                && ty > 0
                && (tx as usize) < (level.width - 1)
                && (ty as usize) < (level.height - 1)
            {
                let (ux, uy) = (tx as usize, ty as usize);
                if let Some(tile) = level.get_tile_mut(ux, uy)
                    && tile.tile_type == TileType::Floor
                {
                    let trap_type = trap_types[rng.gen_range(0..trap_types.len())];
                    tile.tile_type = TileType::Trap {
                        detected: true,
                        trap_type,
                    };
                    tile.remembered = true;
                    created = true;
                }
            }
        }
    }
    created
}

pub fn spell_recharge_item<R: rand::Rng>(
    player: &mut Player,
    num_charges: i32,
    rng: &mut R,
) -> (bool, String) {
    if let Some(pos) = player
        .inventory
        .iter()
        .position(|i| matches!(i.item_type, ItemType::Wand { .. } | ItemType::Staff { .. }))
    {
        let item = &mut player.inventory[pos];
        let current_charges = match item.item_type {
            ItemType::Wand { charges, .. } => charges as i32,
            ItemType::Staff { charges, .. } => charges as i32,
            _ => 0,
        };
        let depth = item.sub_category_id as i32;
        let fail_chance = num_charges + 50 - depth - current_charges;
        let failed = if fail_chance < 19 {
            true
        } else {
            rng.gen_range(1..=(fail_chance / 10).max(1)) == 1
        };

        if failed {
            let name = item.name.clone();
            player.inventory.remove(pos);
            (
                true,
                format!(
                    "There is a bright flash of light! Your {} is destroyed!",
                    name
                ),
            )
        } else {
            let divisor = (depth + 2).max(1);
            let added_base = (num_charges / divisor) + 1;
            let added = 2 + rng.gen_range(1..=added_base.max(1)) as u32;
            let name = item.name.clone();
            match &mut item.item_type {
                ItemType::Wand { charges, .. } => *charges += added,
                ItemType::Staff { charges, .. } => *charges += added,
                _ => {}
            }
            (
                true,
                format!(
                    "A burst of magical energy recharges your {} (+{} charges)!",
                    name, added
                ),
            )
        }
    } else {
        (
            false,
            "You have no wands or staves to recharge.".to_string(),
        )
    }
}

pub fn spell_mass_genocide(player: &Player, monsters: &mut Vec<Monster>) -> (usize, String) {
    let mut killed = 0;
    monsters.retain(|m| {
        let dy = (player.y as isize - m.y as isize).abs();
        let dx = (player.x as isize - m.x as isize).abs();
        let dist = ((dy + dx) * 2 - dx.min(dy)) / 2;
        if dist <= 20 && !m.is_win_creature() {
            killed += 1;
            false
        } else {
            true
        }
    });
    (
        killed,
        "A cosmic flash annihilates nearby creatures!".to_string(),
    )
}

pub fn spell_genocide(
    symbol: char,
    _player: &Player,
    monsters: &mut Vec<Monster>,
) -> (usize, String) {
    let mut killed = 0;
    let mut win_unaffected = false;
    monsters.retain(|m| {
        if m.symbol == symbol {
            if m.is_win_creature() {
                win_unaffected = true;
                true
            } else {
                killed += 1;
                false
            }
        } else {
            true
        }
    });
    let mut msg = format!(
        "Annihilated {} creatures of type '{}' across the level!",
        killed, symbol
    );
    if win_unaffected {
        msg.push_str(" The Balrog is unaffected.");
    }
    (killed, msg)
}

pub fn spell_teleport_level<R: rand::Rng>(current_depth: u32, rng: &mut R) -> u32 {
    let offset = -3 + 2 * rng.gen_range(1..=2);
    (current_depth as i32 + offset).max(1) as u32
}

pub fn quaff_potion<R: rand::Rng>(
    item: &Item,
    player: &mut Player,
    _level: &mut DungeonLevel,
    _monsters: &mut Vec<Monster>,
    rng: &mut R,
    status_msg: &mut String,
) -> bool {
    let mut flags = item.flags;
    let mut cat = item.category_id;
    if flags == 0
        && let Some(def) = GAME_OBJECTS
            .iter()
            .find(|o| o.name.eq_ignore_ascii_case(&item.name))
    {
        flags = def.flags;
        cat = def.category_id;
    }
    if flags == 0 {
        let nl = item.name.to_lowercase();
        if nl.contains("cure light") {
            flags = 1 << 12;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("cure serious") {
            flags = 1 << 13;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("cure critical") {
            flags = 1 << 14;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("healing") {
            flags = 1 << 15;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("speed") || nl.contains("haste") {
            flags = 1 << 22;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("heroism") {
            flags = 1 << 4;
            cat = TV_POTION2;
        } else if nl.contains("restore strength") {
            flags = 1 << 2;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("restore intelligence") {
            flags = 1 << 5;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("restore wisdom") {
            flags = 1 << 8;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("restore charisma") {
            flags = 1 << 11;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("restore dexterity") {
            flags = 1 << 26;
            cat = crate::entity::treasure_data::TV_POTION1;
        } else if nl.contains("restore constitution") {
            flags = 1 << 27;
            cat = crate::entity::treasure_data::TV_POTION1;
        }
    }

    let mut msgs = Vec::new();
    let mut item_flags = flags;
    while item_flags != 0 {
        let bit = item_flags.trailing_zeros();
        item_flags &= !(1 << bit);
        let mut potion_id = bit + 1;
        if cat == TV_POTION2 {
            potion_id += 32;
        }

        match potion_id {
            1 => {
                if player.increase_stat(0, rng) {
                    msgs.push("Wow! What bulging muscles!".to_string());
                }
            }
            2 => {
                if player.drain_stat(0, rng) {
                    msgs.push("You feel your strength draining away.".to_string());
                }
            }
            3 => {
                if player.restore_stat(0) {
                    msgs.push("You feel warm all over.".to_string());
                }
            }
            4 => {
                if player.increase_stat(1, rng) {
                    msgs.push("Aren't you brilliant!".to_string());
                }
            }
            5 => {
                if player.drain_stat(1, rng) {
                    msgs.push("Your mind feels fuzzy and dull.".to_string());
                }
            }
            6 => {
                if player.restore_stat(1) {
                    msgs.push("You have a warm feeling.".to_string());
                }
            }
            7 => {
                if player.increase_stat(2, rng) {
                    msgs.push("You suddenly have a profound thought!".to_string());
                }
            }
            8 => {
                if player.drain_stat(2, rng) {
                    msgs.push("You feel naive and foolish.".to_string());
                }
            }
            9 => {
                if player.restore_stat(2) {
                    msgs.push("You feel your wisdom returning.".to_string());
                }
            }
            10 => {
                if player.increase_stat(5, rng) {
                    msgs.push("Gee, ain't you cute!".to_string());
                }
            }
            11 => {
                if player.drain_stat(5, rng) {
                    msgs.push("You feel repulsive and hideous.".to_string());
                }
            }
            12 => {
                if player.restore_stat(5) {
                    msgs.push("You feel your looks returning.".to_string());
                }
            }
            13 => {
                let heal = Dice::new(2, 7).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                msgs.push(format!("You feel a little better. Healed {} HP.", heal));
            }
            14 => {
                let heal = Dice::new(4, 7).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                msgs.push(format!("You feel much better. Healed {} HP.", heal));
            }
            15 => {
                let heal = Dice::new(6, 7).roll(rng) as i32;
                player.hp = (player.hp + heal).min(player.max_hp);
                msgs.push(format!("You feel very healthy! Healed {} HP.", heal));
            }
            16 => {
                player.hp = (player.hp + 1000).min(player.max_hp);
                player.flags.blind = 0;
                player.flags.afraid = 0;
                player.flags.poisoned = 0;
                player.flags.confused = 0;
                msgs.push("You feel completely healed and rejuvenated!".to_string());
            }
            17 => {
                if player.increase_stat(4, rng) {
                    msgs.push("You feel tingly for a moment.".to_string());
                }
            }
            18 => {
                let gain = ((player.exp / 2) + 10).min(100_000);
                player.add_experience(gain);
                msgs.push("You feel more experienced.".to_string());
            }
            19 => {
                if !player.flags.free_action {
                    let sleep_dur = rng.gen_range(1..=4) + 4;
                    player.flags.paralysis += sleep_dur;
                    msgs.push("You fall asleep.".to_string());
                }
            }
            20 => {
                if player.flags.blind == 0 {
                    msgs.push("You are covered by a veil of darkness.".to_string());
                }
                player.flags.blind += rng.gen_range(1..=100) + 100;
            }
            21 => {
                if player.flags.confused == 0 {
                    msgs.push("Hey! This is good stuff! *Hick!*".to_string());
                }
                player.flags.confused += rng.gen_range(1..=20) + 12;
            }
            22 => {
                if player.flags.poisoned == 0 {
                    msgs.push("You feel very sick.".to_string());
                }
                player.flags.poisoned += rng.gen_range(1..=15) + 10;
            }
            23 => {
                player.flags.fast += rng.gen_range(15..=40);
                msgs.push("You feel yourself moving faster!".to_string());
            }
            24 => {
                player.flags.slow += rng.gen_range(15..=40);
                msgs.push("You feel yourself moving slower!".to_string());
            }
            26 => {
                if player.increase_stat(3, rng) {
                    msgs.push("You feel more limber!".to_string());
                }
            }
            27 => {
                if player.restore_stat(3) {
                    msgs.push("You feel less clumsy.".to_string());
                }
            }
            28 => {
                if player.restore_stat(4) {
                    msgs.push("You feel your health returning!".to_string());
                }
            }
            29 => {
                player.flags.blind = 0;
                msgs.push("Your vision clears.".to_string());
            }
            30 => {
                player.flags.confused = 0;
                msgs.push("You feel less confused.".to_string());
            }
            31 => {
                player.flags.poisoned = 0;
                msgs.push("The poison leaves your veins.".to_string());
            }
            34 => {
                if player.exp > 0 {
                    let loss = player.exp / 5 + rng.gen_range(0..=(player.exp / 5));
                    player.lose_experience(loss);
                    msgs.push("You feel your memories fade.".to_string());
                }
            }
            35 => {
                player.flags.poisoned = 0;
                if player.food > 150 {
                    player.food = 150;
                }
                player.flags.paralysis = 4;
                msgs.push("The potion makes you vomit!".to_string());
            }
            36 => {
                player.flags.invulnerability += rng.gen_range(10..=20);
                msgs.push("You feel completely invulnerable!".to_string());
            }
            37 => {
                player.flags.afraid = 0;
                player.flags.heroism += rng.gen_range(25..=50);
                msgs.push("You feel like a hero!".to_string());
            }
            38 => {
                player.flags.afraid = 0;
                player.flags.super_heroism += rng.gen_range(25..=50);
                msgs.push("You feel like a SUPER hero!".to_string());
            }
            39 => {
                player.flags.afraid = 0;
                msgs.push("Fear leaves your heart.".to_string());
            }
            40 => {
                if player.restore_life_levels() {
                    msgs.push("You feel your lost life energies returning!".to_string());
                }
            }
            41 => {
                player.flags.heat_resistance += rng.gen_range(10..=20);
                msgs.push("You feel resistant to heat.".to_string());
            }
            42 => {
                player.flags.cold_resistance += rng.gen_range(10..=20);
                msgs.push("You feel resistant to cold.".to_string());
            }
            43 => {
                player.flags.detect_invisible += rng.gen_range(12..=24);
                player.flags.see_invisible = true;
                msgs.push("Your eyes tingle; you can see invisible creatures!".to_string());
            }
            44 => {
                player.flags.poisoned /= 2;
                msgs.push("The poison in your veins slows down.".to_string());
            }
            45 => {
                player.flags.poisoned = 0;
                msgs.push("The poison leaves your veins.".to_string());
            }
            46 => {
                if player.mana < player.max_mana {
                    player.mana = player.max_mana;
                    msgs.push("You feel your head clear and mana fully restored.".to_string());
                }
            }
            47 => {
                player.flags.timed_infra += 100 + rng.gen_range(1..=100);
                msgs.push("Your eyes begin to tingle.".to_string());
            }
            _ => {}
        }
    }

    if msgs.is_empty() {
        *status_msg = format!("You quaffed {}. Nothing seems to happen.", item.name);
    } else {
        *status_msg = format!("You quaffed {}! {}", item.name, msgs.join(" "));
    }
    true
}

pub fn read_scroll<R: rand::Rng>(
    item: &Item,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    rng: &mut R,
    status_msg: &mut String,
) -> bool {
    let mut flags = item.flags;
    let mut cat = item.category_id;
    if flags == 0
        && let Some(def) = GAME_OBJECTS
            .iter()
            .find(|o| o.name.eq_ignore_ascii_case(&item.name))
    {
        flags = def.flags;
        cat = def.category_id;
    }
    if flags == 0 {
        let nl = item.name.to_lowercase();
        if nl.contains("phase door") {
            flags = 1 << 7;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("teleport level") {
            flags = 1 << 9;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("teleport") {
            flags = 1 << 8;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("identify") {
            flags = 1 << 3;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("recall") {
            flags = 1 << 8;
            cat = TV_SCROLL2;
        } else if nl.contains("light") {
            flags = 1 << 5;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("mapping") {
            flags = 1 << 11;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("blessing") {
            flags = 1 << 5;
            cat = TV_SCROLL2;
        } else if nl.contains("chant") {
            flags = 1 << 6;
            cat = TV_SCROLL2;
        } else if nl.contains("prayer") {
            flags = 1 << 7;
            cat = TV_SCROLL2;
        } else if nl.contains("genocide") {
            flags = 1 << 25;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("mass genocide") {
            flags = 1 << 18;
            cat = crate::entity::treasure_data::TV_SCROLL1;
        } else if nl.contains("destruction") {
            flags = 1 << 9;
            cat = TV_SCROLL2;
        }
    }

    let mut msgs = Vec::new();
    let mut item_flags = flags;
    while item_flags != 0 {
        let bit = item_flags.trailing_zeros();
        item_flags &= !(1 << bit);
        let mut scroll_type = bit + 1;
        if cat == TV_SCROLL2 {
            scroll_type += 32;
        }

        match scroll_type {
            1 => {
                if let Some(wield) = player
                    .equipment
                    .iter_mut()
                    .find(|e| e.equipped_slot.as_deref() == Some("Wielding"))
                {
                    let chance = if wield.to_hit > 0 {
                        (50 - wield.to_hit as i32 * 5).max(5)
                    } else {
                        100
                    };
                    if rng.gen_range(1..=100) <= chance {
                        wield.to_hit += 1;
                        wield.is_cursed = false;
                        msgs.push(format!(
                            "Your {} glows faintly! To-hit increased to (+{}).",
                            wield.name, wield.to_hit
                        ));
                    } else {
                        msgs.push("The enchantment fails.".to_string());
                    }
                } else {
                    msgs.push("You are not wielding a weapon.".to_string());
                }
            }
            2 => {
                if let Some(wield) = player
                    .equipment
                    .iter_mut()
                    .find(|e| e.equipped_slot.as_deref() == Some("Wielding"))
                {
                    let chance = if wield.to_damage > 0 {
                        (50 - wield.to_damage as i32 * 5).max(5)
                    } else {
                        100
                    };
                    if rng.gen_range(1..=100) <= chance {
                        wield.to_damage += 1;
                        wield.is_cursed = false;
                        msgs.push(format!(
                            "Your {} glows faintly! Damage increased to (+{}).",
                            wield.name, wield.to_damage
                        ));
                    } else {
                        msgs.push("The enchantment fails.".to_string());
                    }
                } else {
                    msgs.push("You are not wielding a weapon.".to_string());
                }
            }
            3 => {
                if let Some(armor) = player
                    .equipment
                    .iter_mut()
                    .find(|e| matches!(e.item_type, ItemType::Armor { .. }))
                {
                    let chance = if armor.to_ac > 0 {
                        (50 - armor.to_ac as i32 * 5).max(5)
                    } else {
                        100
                    };
                    if rng.gen_range(1..=100) <= chance {
                        armor.to_ac += 1;
                        armor.is_cursed = false;
                        msgs.push(format!(
                            "Your {} glows faintly! AC increased to [{:+}].",
                            armor.name, armor.to_ac
                        ));
                    } else {
                        msgs.push("The enchantment fails.".to_string());
                    }
                } else {
                    msgs.push("You are not wearing any armor.".to_string());
                }
            }
            4 => {
                if let Some(unid) = player.inventory.iter_mut().find(|i| !i.identified) {
                    unid.identified = true;
                    msgs.push(format!("You identify the {}!", unid.name));
                } else {
                    msgs.push("All your items are already identified.".to_string());
                }
            }
            5 => {
                let mut uncursed = 0;
                for eq in &mut player.equipment {
                    if eq.is_cursed {
                        eq.is_cursed = false;
                        uncursed += 1;
                    }
                }
                if uncursed > 0 {
                    msgs.push(
                        "You feel as if someone is watching over you. Curses lifted!".to_string(),
                    );
                } else {
                    msgs.push("You feel as if someone is watching over you.".to_string());
                }
            }
            6 => {
                level.illuminate_area(player.x, player.y, 3);
                msgs.push("Brilliant light illuminates the area.".to_string());
            }
            7 => {
                let count = rng.gen_range(1..=3);
                for _ in 0..count {
                    let (sx, sy) = level.find_random_floor_tile_near(player.x, player.y, 3);
                    let c_idx =
                        crate::entity::monster_data::get_monster_for_level(level.depth, rng);
                    monsters.push(Monster::from_creature_id(c_idx, sx, sy, false, rng));
                }
                msgs.push("You hear footsteps approaching from the shadows!".to_string());
            }
            8 => {
                let dest = level.find_random_floor_tile_near(player.x, player.y, 10);
                player.move_to(dest.0, dest.1);
                msgs.push("You phase door across short distance!".to_string());
            }
            9 => {
                let dest = level.find_random_floor_tile();
                player.move_to(dest.0, dest.1);
                msgs.push("You teleport across the level!".to_string());
            }
            10 => {
                let offset = -3 + 2 * rng.gen_range(1..=2);
                let new_depth = (level.depth as i32 + offset).max(1) as u32;
                level.depth = new_depth;
                msgs.push(format!(
                    "You sink through the floor and warp to dungeon level {}!",
                    new_depth
                ));
            }
            11 => {
                player.flags.confuse_monster = true;
                msgs.push("Your hands begin to glow with confusing energy.".to_string());
            }
            12 => {
                level.map_area(player.x, player.y, 50);
                msgs.push("A map of the surroundings flashes into your mind.".to_string());
            }
            13 => {
                let mut count = 0;
                for m in monsters.iter_mut() {
                    let dist = (m.x as isize - player.x as isize)
                        .abs()
                        .max((m.y as isize - player.y as isize).abs());
                    if dist <= 10 {
                        m.asleep += rng.gen_range(5..=15);
                        count += 1;
                    }
                }
                msgs.push(format!(
                    "A sleep fog blankets {} surrounding monsters!",
                    count
                ));
            }
            14 => {
                level.place_glyph_of_warding(player.x, player.y);
                msgs.push("You trace a glowing holy Glyph of Warding upon the floor!".to_string());
            }
            15 => {
                let count = level.detect_treasures_near(player.x, player.y, 25);
                msgs.push(format!("You sense {} precious treasures nearby.", count));
            }
            16 => {
                let count = level.items.len();
                msgs.push(format!("You sense {} objects on the level.", count));
            }
            17 => {
                let count = level.detect_traps_near(player.x, player.y, 20);
                msgs.push(format!("You sense {} traps in the vicinity.", count));
            }
            18 => {
                let count = level.detect_doors_and_stairs_near(player.x, player.y, 20);
                msgs.push(format!(
                    "You sense {} secret doors and stairs in the vicinity.",
                    count
                ));
            }
            19 => {
                let (count, mmsg) = spell_mass_genocide(player, monsters);
                msgs.push(format!("{} ({} destroyed)", mmsg, count));
            }
            20 => {
                player.flags.detect_invisible += rng.gen_range(12..=24);
                player.flags.see_invisible = true;
                msgs.push("Your eyes tingle; you can see invisible creatures!".to_string());
            }
            21 => {
                for m in monsters.iter_mut() {
                    m.asleep = 0;
                    m.was_attacked = true;
                }
                msgs.push(
                    "There is a high pitched humming noise! All monsters awaken!".to_string(),
                );
            }
            22 => {
                spell_surround_player_with_traps(player, level, rng);
                msgs.push("You hear clicking and grinding! Traps surround you!".to_string());
            }
            23 => {
                let count = level.destroy_adjacent_doors_and_traps(player.x, player.y);
                msgs.push(format!(
                    "A flash of energy destroys {} adjacent doors and traps.",
                    count
                ));
            }
            24 => {
                spell_surround_player_with_doors(player, level);
                msgs.push("Heavy stone doors slam shut all around you!".to_string());
            }
            25 => {
                let (_, rmsg) = spell_recharge_item(player, 60, rng);
                msgs.push(rmsg);
            }
            26 => {
                let (count, gmsg) = spell_genocide('o', player, monsters);
                msgs.push(format!("{} ({} destroyed)", gmsg, count));
            }
            27 => {
                msgs.push("Darkness envelops the area!".to_string());
            }
            28 => {
                player.flags.protect_evil += rng.gen_range(1..=25) + 3 * player.level as i16;
                msgs.push("A divine barrier rises to repel evil creatures!".to_string());
            }
            29 => {
                let food = Item::new(
                    "Ration of Food",
                    1,
                    20,
                    ItemType::Food { nutrition: 5000 },
                );
                if player.add_item_to_inventory(food.clone()) {
                    msgs.push("A nutritious ration of food appears in your pack!".to_string());
                } else {
                    level.items.push(crate::dungeon::FloorItem {
                        x: player.x,
                        y: player.y,
                        item: food,
                    });
                    msgs.push(
                        "Your pack is full; a nutritious ration of food drops to the floor."
                            .to_string(),
                    );
                }
            }
            30 => {
                let mut count = 0;
                let mut killed = Vec::new();
                for (idx, m) in monsters.iter_mut().enumerate() {
                    if (m.defenses() & crate::entity::monster_data::CD_UNDEAD) != 0 {
                        m.was_attacked = true;
                        m.asleep = 0;
                        count += 1;
                        if m.take_damage(60) {
                            killed.push(idx);
                        }
                    }
                }
                for &idx in killed.iter().rev() {
                    player
                        .gain_kill_experience(monsters[idx].experience_reward, monsters[idx].level);
                    monsters.remove(idx);
                }
                msgs.push(format!(
                    "Holy radiance dispels the undead! Affected {}, killed {}.",
                    count,
                    killed.len()
                ));
            }
            33 => {
                if let Some(wield) = player
                    .equipment
                    .iter_mut()
                    .find(|e| e.equipped_slot.as_deref() == Some("Wielding"))
                {
                    for _ in 0..2 {
                        let c = if wield.to_hit > 0 {
                            (50 - wield.to_hit as i32 * 5).max(5)
                        } else {
                            100
                        };
                        if rng.gen_range(1..=100) <= c {
                            wield.to_hit += 1;
                        }
                        let cd = if wield.to_damage > 0 {
                            (50 - wield.to_damage as i32 * 5).max(5)
                        } else {
                            100
                        };
                        if rng.gen_range(1..=100) <= cd {
                            wield.to_damage += 1;
                        }
                    }
                    wield.is_cursed = false;
                    msgs.push(format!(
                        "Your {} glows brilliantly! (+{},+{})",
                        wield.name, wield.to_hit, wield.to_damage
                    ));
                } else {
                    msgs.push("You are not wielding a weapon.".to_string());
                }
            }
            34 => {
                if let Some(wield) = player
                    .equipment
                    .iter_mut()
                    .find(|e| e.equipped_slot.as_deref() == Some("Wielding"))
                {
                    wield.is_cursed = true;
                    wield.to_hit = -(rng.gen_range(1..=5) + rng.gen_range(1..=5));
                    wield.to_damage = -(rng.gen_range(1..=5) + rng.gen_range(1..=5));
                    msgs.push(format!(
                        "Your {} glows black and fades! It is cursed!",
                        wield.name
                    ));
                }
            }
            35 => {
                if let Some(armor) = player
                    .equipment
                    .iter_mut()
                    .find(|e| matches!(e.item_type, ItemType::Armor { .. }))
                {
                    let attempts = rng.gen_range(1..=3);
                    for _ in 0..attempts {
                        let c = if armor.to_ac > 0 {
                            (50 - armor.to_ac as i32 * 5).max(5)
                        } else {
                            100
                        };
                        if rng.gen_range(1..=100) <= c {
                            armor.to_ac += 1;
                        }
                    }
                    armor.is_cursed = false;
                    msgs.push(format!(
                        "Your {} glows brightly! [{:+}]",
                        armor.name, armor.to_ac
                    ));
                }
            }
            36 => {
                if let Some(armor) = player
                    .equipment
                    .iter_mut()
                    .find(|e| matches!(e.item_type, ItemType::Armor { .. }))
                {
                    armor.is_cursed = true;
                    armor.to_ac = -(rng.gen_range(1..=5) + rng.gen_range(1..=5));
                    msgs.push(format!("Your {} glows black! It is cursed!", armor.name));
                }
            }
            37 => {
                let count = rng.gen_range(1..=3);
                for _ in 0..count {
                    let (sx, sy) = level.find_random_floor_tile_near(player.x, player.y, 3);
                    monsters.push(Monster::from_creature_id(40, sx, sy, false, rng));
                }
                msgs.push("Cold dread fills the air as undead rise!".to_string());
            }
            38 => {
                player.flags.blessed += rng.gen_range(1..=12) + 6;
                msgs.push("You feel a righteous blessing descend upon you!".to_string());
            }
            39 => {
                player.flags.blessed += rng.gen_range(1..=24) + 12;
                msgs.push("A sacred chant echoes around you, conferring divine favor!".to_string());
            }
            40 => {
                player.flags.blessed += rng.gen_range(1..=48) + 24;
                msgs.push("A profound holy prayer fills your soul with divine power!".to_string());
            }
            41 => {
                if player.flags.word_of_recall == 0 {
                    player.flags.word_of_recall = 25 + rng.gen_range(1..=30);
                    msgs.push("The air about you becomes charged...".to_string());
                } else {
                    player.flags.word_of_recall = 0;
                    msgs.push("A tension leaves the air around you.".to_string());
                }
            }
            42 => {
                level.destroy_area(player.x, player.y, 12);
                msgs.push("The dungeon dissolves into raw stone and rubble!".to_string());
            }
            _ => {}
        }
    }

    if msgs.is_empty() {
        *status_msg = format!("You read {}. Nothing seems to happen.", item.name);
    } else {
        *status_msg = format!("You read {}! {}", item.name, msgs.join(" "));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Race;

    #[test]
    fn test_canonical_stat_adjustments() {
        assert_eq!(stat_adjustment(3), 0);
        assert_eq!(stat_adjustment(7), 0);
        assert_eq!(stat_adjustment(8), 1);
        assert_eq!(stat_adjustment(14), 1);
        assert_eq!(stat_adjustment(15), 2);
        assert_eq!(stat_adjustment(17), 2);
        assert_eq!(stat_adjustment(18), 3);
        assert_eq!(stat_adjustment(50), 3); // 18/50 -> 68 in linear scale? In 18/xx, 18/00 is 18, 18/50 is 68
        assert_eq!(stat_adjustment(67), 3);
        assert_eq!(stat_adjustment(68), 4);
        assert_eq!(stat_adjustment(88), 5);
        assert_eq!(stat_adjustment(108), 6);
        assert_eq!(stat_adjustment(118), 7);
    }

    #[test]
    fn test_min_level_for_spells() {
        assert_eq!(min_level_for_spells(Class::Mage), 1);
        assert_eq!(min_level_for_spells(Class::Priest), 1);
        assert_eq!(min_level_for_spells(Class::Ranger), 3);
        assert_eq!(min_level_for_spells(Class::Rogue), 5);
        assert_eq!(min_level_for_spells(Class::Paladin), 1);
        assert_eq!(min_level_for_spells(Class::Warrior), 99);
    }

    #[test]
    fn test_calculate_max_mana() {
        // Zero mana if no spells learnt
        assert_eq!(calculate_max_mana(5, Class::Mage, 18, 0), 0);

        // Mage level 1 with INT 18 (stat_adjustment = 3)
        // levels = 1, adj = 3 -> 3 * 1 / 2 = 1 -> mana = 1 + 1 = 2
        assert_eq!(calculate_max_mana(1, Class::Mage, 18, 1), 2);

        // Warrior can never get mana
        assert_eq!(calculate_max_mana(20, Class::Warrior, 18, 1), 0);
    }

    #[test]
    fn test_number_of_spells_allowed() {
        // Below min level
        assert_eq!(number_of_spells_allowed(2, Class::Ranger, 18), 0);
        assert_eq!(number_of_spells_allowed(4, Class::Rogue, 18), 0);

        // Mage level 1 with INT 18 (adj = 3 -> levels = 1 -> allowed = 1)
        assert_eq!(number_of_spells_allowed(1, Class::Mage, 18), 1);
    }

    #[test]
    fn test_spell_chance_of_success_clamping() {
        let spell = SpellDef {
            name: "Test Spell",
            level_required: 1,
            mana_required: 1,
            failure_chance: 99,
            exp_gain: 1,
        };
        // Clamped at 95 max
        let chance = spell_chance_of_success(1, Class::Mage, 10, 10, &spell);
        assert!(chance <= 95);

        // Clamped at 5 min
        let easy_spell = SpellDef {
            name: "Easy",
            level_required: 1,
            mana_required: 1,
            failure_chance: 1,
            exp_gain: 1,
        };
        let low_chance = spell_chance_of_success(50, Class::Mage, 118, 100, &easy_spell);
        assert_eq!(low_chance, 5);
    }

    #[test]
    fn test_book_creation_and_flags() {
        let b0 = create_book(0, false);
        assert!(b0.name.contains("Beginners-Magick"));
        if let ItemType::MagicBook { spell_flags } = b0.item_type {
            assert_eq!(spell_flags, 0x0000007F);
            let spells = get_spells_in_book(spell_flags);
            assert_eq!(spells, vec![0, 1, 2, 3, 4, 5, 6]);
        } else {
            panic!("Expected MagicBook");
        }

        let p0 = create_book(0, true);
        assert!(p0.name.contains("Beginners Handbook"));
        if let ItemType::PrayerBook { spell_flags } = p0.item_type {
            assert_eq!(spell_flags, 0x000000FF);
        } else {
            panic!("Expected PrayerBook");
        }
    }

    #[test]
    fn test_apply_spell_mana_and_fatigue() {
        let mut rng = rand::thread_rng();
        let mut player = Player::new("Tester", Race::Human, Class::Mage, 10, 10);
        player.mana = 10;
        let mut msg = String::new();

        // Normal mana consumption
        apply_spell_mana_and_fatigue(&mut player, 4, false, &mut rng, &mut msg);
        assert_eq!(player.mana, 6);

        // Over-cast: mana is 6, spell requires 10 -> fatigue & paralysis!
        apply_spell_mana_and_fatigue(&mut player, 10, false, &mut rng, &mut msg);
        assert_eq!(player.mana, 0);
        assert!(player.flags.paralysis > 0);
        assert!(msg.contains("faint from the effort"));
    }

    #[test]
    fn test_directional_spell_bolt_and_ball() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Mage, 5, 5);
        for x in 5..=8 {
            if let Some(tile) = level.get_tile_mut(x, 5) {
                tile.tile_type = TileType::Floor;
            }
        }
        let mut monsters = vec![Monster::new("Goblin", 'o', 8, 5, 10, Dice::new(1, 4), 15)];
        let mut msg = String::new();

        // Bolt towards East (dx=1, dy=0)
        cast_spell_bolt(
            "Magic Missile",
            MagicSpellType::MagicMissile,
            1,
            0,
            15,
            &mut player,
            &mut level,
            &mut monsters,
            &mut msg,
        );
        assert!(monsters.is_empty(), "Monster should be killed by 15 damage");
        assert!(msg.contains("dies in a fit of agony") || msg.contains("destroyed"));
    }

    #[test]
    fn test_mage_requires_book_priest_does_not() {
        // Mage at level 1 with INT 18
        let mut mage = Player::new("MageTester", Race::Human, Class::Mage, 5, 5);
        mage.stats.intelligence = 18;
        mage.inventory.clear(); // Remove starting book to test bookless requirement
        mage.update_spells_to_learn();
        assert_eq!(mage.new_spells_to_learn, 1);

        // Without books in inventory, mage cannot learn any spells
        assert!(mage.learnable_spells().is_empty());

        // Give mage Book 0 ([Beginners-Magick])
        mage.inventory.push(create_book(0, false));
        let mage_learnable = mage.learnable_spells();
        assert!(!mage_learnable.is_empty());
        assert!(mage_learnable.contains(&0)); // Magic Missile (lvl 1)
        assert!(mage_learnable.contains(&1)); // Detect Monsters (lvl 1)

        // Priest at level 1 with WIS 18
        let mut priest = Player::new("PriestTester", Race::Human, Class::Priest, 5, 5);
        priest.stats.wisdom = 18;
        priest.inventory.clear(); // Remove starting book to verify priest learns from God
        priest.update_spells_to_learn();
        assert_eq!(priest.new_spells_to_learn, 1);

        // Priest has NO books in inventory, but can still learn from their God!
        let priest_learnable = priest.learnable_spells();
        assert!(!priest_learnable.is_empty());
        assert!(priest_learnable.contains(&0)); // Detect Evil (lvl 1)
    }

    #[test]
    fn test_learn_spell_initializes_mana() {
        let mut mage = Player::new("MageTester", Race::Human, Class::Mage, 5, 5);
        mage.stats.intelligence = 18;
        mage.inventory.push(create_book(0, false));
        mage.update_spells_to_learn();

        assert_eq!(mage.max_mana, 0);
        assert_eq!(mage.mana, 0);
        assert_eq!(mage.spells_learnt, 0);
        assert_eq!(mage.new_spells_to_learn, 1);

        // Learn spell 0 (Magic Missile)
        mage.learn_spell(0);

        assert_eq!(mage.spells_learnt, 1);
        assert_eq!(mage.spells_learned_order[0], 0);
        assert_eq!(mage.new_spells_to_learn, 0);
        assert!(mage.max_mana > 0);
        assert_eq!(mage.mana, mage.max_mana);
    }

    #[test]
    fn test_ball_spell_radius_and_self_damage() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Mage, 5, 5);
        player.hp = 100;
        player.max_hp = 100;

        for y in 4..=6 {
            for x in 4..=8 {
                if let Some(t) = level.get_tile_mut(x, y) {
                    t.tile_type = TileType::Floor;
                }
            }
        }

        // Two monsters adjacent to each other at (7, 5) and (7, 6)
        let mut monsters = vec![
            Monster::new("Goblin A", 'o', 7, 5, 20, Dice::new(1, 4), 10),
            Monster::new("Goblin B", 'o', 7, 6, 20, Dice::new(1, 4), 10),
        ];
        let mut msg = String::new();

        // Fire ball towards East at radius 2, base damage 50
        cast_spell_ball(
            "Fire Ball",
            MagicSpellType::Fire,
            1,
            0,
            2,
            50,
            &mut player,
            &mut level,
            &mut monsters,
            &mut msg,
        );

        // Both goblins within radius 2 of impact at (7, 5) should take sufficient decayed damage and die
        assert!(
            monsters.is_empty(),
            "Both goblins should be killed by area blast"
        );
        assert!(msg.contains("perished in the blast"));
    }

    #[test]
    fn test_turn_stone_to_mud() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Mage, 5, 5);
        let mut rng = rand::thread_rng();

        // Place wall right next to player at (6, 5)
        if let Some(t) = level.get_tile_mut(6, 5) {
            t.tile_type = TileType::Wall;
        }

        let mut monsters = Vec::new();
        let mut msg = String::new();

        // Cast Wall to Mud (Mage spell 15) East towards (6, 5)
        execute_directional_spell(
            15,
            false,
            1,
            0,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );

        assert_eq!(level.get_tile(6, 5).unwrap().tile_type, TileType::Floor);
        assert!(msg.contains("dissolves into mud"));
    }

    #[test]
    fn test_glyph_of_warding() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Priest, 5, 5);
        let mut rng = rand::thread_rng();
        let mut monsters = Vec::new();
        let mut msg = String::new();

        // Cast Glyph of Warding (Priest prayer 29)
        execute_non_directional_spell(
            29,
            true,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );

        assert_eq!(
            level.get_tile(5, 5).unwrap().tile_type,
            TileType::GlyphOfWarding
        );
        assert!(msg.contains("Glyph of Warding"));
    }

    #[test]
    fn test_elemental_weakness_and_resistance() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Mage, 5, 5);

        // Monster A has CD_FIRE defense weakness (Blubbering Icky-Thing)
        let monster_weak =
            Monster::new("Blubbering Icky-Thing", 'i', 6, 5, 100, Dice::new(1, 4), 10);
        assert_ne!(monster_weak.defenses() & CD_FIRE, 0);

        // Monster B has CS_BR_FIRE spell resistance (Young Red Dragon)
        let monster_resist = Monster::new("Young Red Dragon", 'd', 6, 5, 100, Dice::new(1, 4), 10);
        assert_ne!(monster_resist.spells() & CS_BR_FIRE, 0);

        let mut monsters = vec![monster_weak];
        let mut msg = String::new();

        // Base damage 20 with Fire bolt against fire-weak monster: 20 * 2 = 40 damage (HP 100 -> 60)
        cast_spell_bolt(
            "Fire Bolt",
            MagicSpellType::Fire,
            1,
            0,
            20,
            &mut player,
            &mut level,
            &mut monsters,
            &mut msg,
        );
        assert_eq!(
            monsters[0].hp, 60,
            "Fire weakness should double bolt damage to 40"
        );

        // Fire bolt against fire-resistant monster: 20 / 4 = 5 damage (HP 100 -> 95)
        monsters = vec![monster_resist];
        msg.clear();
        cast_spell_bolt(
            "Fire Bolt",
            MagicSpellType::Fire,
            1,
            0,
            20,
            &mut player,
            &mut level,
            &mut monsters,
            &mut msg,
        );
        assert_eq!(
            monsters[0].hp, 95,
            "Fire resistance should quarter bolt damage to 5"
        );
    }

    #[test]
    fn test_canonical_quaff_potions() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 5, 5);
        let mut monsters = Vec::new();
        let mut rng = rand::thread_rng();
        let mut msg = String::new();

        // 1. Potion of Cure Poison (potion_id 45 -> bit 12 of TV_POTION2)
        player.flags.poisoned = 50;
        let potion_cure_poison = Item {
            name: "Potion of Cure Poison".to_string(),
            count: 1,
            weight: 4,
            category_id: TV_POTION2,
            flags: 1 << 12,
            ..Item::new(
                "Potion of Cure Poison",
                1,
                10,
                ItemType::Potion { heal_amount: 0 },
            )
        };
        quaff_potion(
            &potion_cure_poison,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );
        assert_eq!(player.flags.poisoned, 0);
        assert!(msg.contains("leaves your veins"));

        // 2. Potion of Speed (potion_id 23 -> bit 22 of TV_POTION1)
        player.flags.fast = 0;
        let potion_speed = Item {
            name: "Potion of Speed".to_string(),
            count: 1,
            weight: 4,
            category_id: crate::entity::treasure_data::TV_POTION1,
            flags: 1 << 22,
            ..Item::new(
                "Potion of Speed",
                1,
                10,
                ItemType::Potion { heal_amount: 0 },
            )
        };
        quaff_potion(
            &potion_speed,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );
        assert!(player.flags.fast > 0);
        assert!(msg.contains("moving faster"));

        // 3. Multi-bitflag potion (Restore Strength bit 2 + Speed bit 22)
        player.stats.strength = 10;
        player.max_stats.strength = 16;
        player.flags.fast = 0;
        let multi_potion = Item {
            name: "Multi Potion".to_string(),
            count: 1,
            weight: 4,
            category_id: crate::entity::treasure_data::TV_POTION1,
            flags: (1 << 2) | (1 << 22),
            ..Item::new("Multi Potion", 1, 10, ItemType::Potion { heal_amount: 0 })
        };
        quaff_potion(
            &multi_potion,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );
        assert_eq!(player.stats.strength, 16);
        assert!(player.flags.fast > 0);
    }

    #[test]
    fn test_canonical_read_scrolls() {
        let mut level = DungeonLevel::new(40, 20, 1, 50);
        level.generate_simple_floor();
        let mut player = Player::new("Tester", Race::Human, Class::Warrior, 5, 5);
        let mut monsters = Vec::new();
        let mut rng = rand::thread_rng();
        let mut msg = String::new();

        // 1. Word of Recall scroll (scroll_id 40 -> bit 8 of TV_SCROLL2)
        player.flags.word_of_recall = 0;
        let recall_scroll = Item {
            name: "Scroll of Word of Recall".to_string(),
            count: 1,
            weight: 4,
            category_id: TV_SCROLL2,
            flags: 1 << 8,
            ..Item::new(
                "Scroll of Word of Recall",
                1,
                10,
                ItemType::Scroll { teleport: false },
            )
        };
        read_scroll(
            &recall_scroll,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );
        assert!(player.flags.word_of_recall >= 25);
        assert!(msg.contains("air about you becomes charged"));

        // 2. Enchant Weapon To-Hit (scroll_id 1 -> bit 0 of TV_SCROLL1)
        let mut sword = Item::new(
            "Broadsword",
            1,
            100,
            ItemType::Weapon {
                damage: Dice::new(2, 5),
            },
        );
        sword.equipped_slot = Some("Wielding".to_string());
        sword.to_hit = 0;
        player.equipment.push(sword);

        let enchant_hit_scroll = Item {
            name: "Scroll of Enchant Weapon To-Hit".to_string(),
            count: 1,
            weight: 4,
            category_id: crate::entity::treasure_data::TV_SCROLL1,
            flags: 1 << 0,
            ..Item::new(
                "Scroll of Enchant Weapon To-Hit",
                1,
                10,
                ItemType::Scroll { teleport: false },
            )
        };
        read_scroll(
            &enchant_hit_scroll,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );
        let wield = player
            .equipment
            .iter()
            .find(|e| e.equipped_slot.as_deref() == Some("Wielding"))
            .unwrap();
        assert_eq!(wield.to_hit, 1);
        assert!(msg.contains("glows faintly"));
    }

    #[test]
    fn test_create_food_drops_when_inventory_full() {
        let mut player = Player::new("FoodTester", Race::Human, Class::Priest, 5, 5);
        player.inventory.clear();
        for i in 0..22 {
            player.inventory.push(Item::new(
                &format!("Dagger {}", i),
                1,
                10,
                ItemType::Weapon {
                    damage: Dice::new(1, 4),
                },
            ));
        }
        assert_eq!(player.inventory.len(), 22);

        let mut level = DungeonLevel::new(20, 10, 1, 50);
        let mut monsters = Vec::new();
        let mut rng = rand::thread_rng();
        let mut msg = String::new();

        // Cast Create Food (prayer 13)
        execute_non_directional_spell(
            13,
            true,
            &mut player,
            &mut level,
            &mut monsters,
            &mut rng,
            &mut msg,
        );

        // Inventory still at 22 items
        assert_eq!(player.inventory.len(), 22);
        // Food dropped to the floor
        assert_eq!(level.items.len(), 1);
        assert_eq!(level.items[0].item.name, "Ration of Food");
        assert_eq!(level.items[0].x, player.x);
        assert_eq!(level.items[0].y, player.y);
        assert!(msg.contains("pack is full"));
    }
}
