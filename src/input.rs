use serde::{Serialize, Deserialize};

/// Directions mapped to numeric keypad logic (1-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    SouthWest = 1,
    South = 2,
    SouthEast = 3,
    West = 4,
    Rest = 5,
    East = 6,
    NorthWest = 7,
    North = 8,
    NorthEast = 9,
}

/// Internal actions representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
    Quaff,              // Drink potion
    WearWield,          // Equip item
    InventoryList,      // View inventory (i)
    EquipmentList,      // View equipment (I)
    AimWand,            // Aim a wand
    SearchOneTurn,      // Search around player
    DropItem,           // Drop inventory item
    Disarm,             // Disarm a trap
    CloseDoor,          // Close door/chest
    CharacterStats,     // Display character sheet
    ExchangeWeapon,     // Swap primary/secondary weapons
    BrowseBook,         // Read spellbook
    JamDoor,            // Jam/spike door
    Look,               // Look/examine surroundings
    Inscribe,           // Add inscription to item
    OpenDoor,           // Open door/chest
    CastSpell,          // Mage spellcasting
    Pray,               // Cleric prayers
    ReadScroll,         // Read scroll
    TakeOff,            // Unequip item
    UseStaff,           // Activate staff
    Eat,                // Eat food (E)
    Rest,               // Rest a while (R)
    Bash,               // Bash door/chest/monster
    Tunnel,             // Tunnel/Mine (T)
    GoUpStairs,         // Go up stairs (<)
    GoDownStairs,       // Go down stairs (>)
    Quit,               // Save and quit or exit
    Help,               // Display help screen
    RefillLight,        // Refill lamp/lantern (F)
    ToggleSearch,       // Toggle search mode (#)
    Unknown(char),
}

/// Keyboard Profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum KeyboardProfile {
    StandardQweasd,
    RoguelikeQweasd,
}

pub struct InputMapper {
    profile: KeyboardProfile,
}

impl InputMapper {
    pub fn new(profile: KeyboardProfile) -> Self {
        Self { profile }
    }

    /// Map a keypress to an Action.
    pub fn map_key(&self, key: char) -> Action {
        match self.profile {
            KeyboardProfile::StandardQweasd => self.map_standard(key),
            KeyboardProfile::RoguelikeQweasd => self.map_roguelike(key),
        }
    }

    fn map_standard(&self, key: char) -> Action {
        match key {
            // Movement (qweasdzxc layout)
            'q' => Action::Move(Direction::NorthWest),
            'w' => Action::Move(Direction::North),
            'e' => Action::Move(Direction::NorthEast),
            'a' => Action::Move(Direction::West),
            's' => Action::Move(Direction::Rest),
            'd' => Action::Move(Direction::East),
            'z' => Action::Move(Direction::SouthWest),
            'x' => Action::Move(Direction::South),
            'c' => Action::Move(Direction::SouthEast),

            // Remapped Actions (Standard mode conflicts resolved)
            'h' => Action::Quaff,            // Relocated from 'q'
            'W' => Action::WearWield,        // Relocated from 'w'
            'I' => Action::EquipmentList,    // Relocated from 'e'
            'y' => Action::AimWand,          // Relocated from 'a' / 'z'
            'k' => Action::SearchOneTurn,    // Relocated from 's'
            'D' => Action::DropItem,         // Relocated from 'd'
            'n' => Action::Disarm,           // Relocated from 'D'
            'C' => Action::CloseDoor,        // Relocated from 'c'
            'H' => Action::CharacterStats,   // Relocated from 'C'
            'X' => Action::ExchangeWeapon,   // Relocated from 'x'
            'E' => Action::Eat,              // Eat food
            'R' => Action::Rest,             // Rest a while
            'f' => Action::Bash,             // Bash
            'T' => Action::Tunnel,           // Tunnel
            'F' => Action::RefillLight,      // Refill light
            '#' => Action::ToggleSearch,     // Toggle search



            // Unchanged Standard Commands
            'b' => Action::BrowseBook,
            'j' => Action::JamDoor,
            'l' => Action::Look,
            'u' => Action::UseStaff,
            'i' => Action::InventoryList,    // Inventory view remains 'i'
            'm' => Action::CastSpell,
            'o' => Action::OpenDoor,
            'p' => Action::Pray,
            'r' => Action::ReadScroll,
            't' => Action::TakeOff,
            '{' => Action::Inscribe,
            '<' => Action::GoUpStairs,
            '>' => Action::GoDownStairs,
            'Q' => Action::Quit,
            '?' => Action::Help,

            other => Action::Unknown(other),
        }
    }

    fn map_roguelike(&self, key: char) -> Action {
        match key {
            // Movement (qweasdzxc layout)
            'q' => Action::Move(Direction::NorthWest),
            'w' => Action::Move(Direction::North),
            'e' => Action::Move(Direction::NorthEast),
            'a' => Action::Move(Direction::West),
            's' => Action::Move(Direction::Rest),
            'd' => Action::Move(Direction::East),
            'z' => Action::Move(Direction::SouthWest),
            'x' => Action::Move(Direction::South),
            'c' => Action::Move(Direction::SouthEast),

            // Remapped Actions (Roguelike mode conflicts resolved)
            'Q' => Action::Quit,             // Relocated from 'q'
            'W' => Action::WearWield,        // Relocated from 'w'
            'E' => Action::Eat,              // Eat food
            'R' => Action::Rest,             // Rest a while
            'B' => Action::Bash,             // Bash
            'T' => Action::Tunnel,           // Tunnel
            'F' => Action::RefillLight,      // Refill light
            '#' => Action::ToggleSearch,     // Toggle search
            'h' => Action::Quaff,            // Quaff potion (same as standard)
            'k' => Action::SearchOneTurn,    // Relocated from 's'
            'D' => Action::DropItem,         // Relocated from 'd' (originally disarm)
            'n' => Action::Disarm,           // Relocated from 'D'
            'y' => Action::AimWand,          // Relocated from 'z'
            'l' => Action::Look,             // Relocated from 'x'
            'C' => Action::CloseDoor,        // Relocated from 'c'
            'H' => Action::CharacterStats,   // Relocated from 'C'

            // Unchanged Roguelike Commands
            'i' => Action::InventoryList,
            'I' => Action::EquipmentList,
            'p' => Action::Pray,
            'r' => Action::ReadScroll,
            'o' => Action::OpenDoor,
            'm' => Action::CastSpell,
            't' => Action::TakeOff,
            '{' => Action::Inscribe,
            '<' => Action::GoUpStairs,
            '>' => Action::GoDownStairs,
            '?' => Action::Help,
            'j' => Action::JamDoor,
            'u' => Action::UseStaff,

            other => Action::Unknown(other),
        }
    }
}
