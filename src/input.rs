use serde::{Deserialize, Serialize};

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

impl Direction {
    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            1 => Some(Direction::SouthWest),
            2 => Some(Direction::South),
            3 => Some(Direction::SouthEast),
            4 => Some(Direction::West),
            5 => Some(Direction::Rest),
            6 => Some(Direction::East),
            7 => Some(Direction::NorthWest),
            8 => Some(Direction::North),
            9 => Some(Direction::NorthEast),
            _ => None,
        }
    }

    pub fn to_dx_dy(self) -> (isize, isize) {
        match self {
            Direction::SouthWest => (-1, 1),
            Direction::South => (0, 1),
            Direction::SouthEast => (1, 1),
            Direction::West => (-1, 0),
            Direction::Rest => (0, 0),
            Direction::East => (1, 0),
            Direction::NorthWest => (-1, -1),
            Direction::North => (0, -1),
            Direction::NorthEast => (1, -1),
        }
    }
}

/// Internal actions representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
    Run(Direction),
    MoveWithoutPickup(Direction),
    Quaff,              // Drink potion
    WearWield,          // Equip item
    InventoryList,      // View inventory (i)
    EquipmentList,      // View equipment (I or e)
    AimWand,            // Aim a wand (a or z)
    SearchOneTurn,      // Search around player (s)
    DropItem,           // Drop inventory item (d)
    Disarm,             // Disarm a trap (D)
    CloseDoor,          // Close door/chest (c)
    CharacterStats,     // Display character sheet (C)
    ExchangeWeapon,     // Swap primary/secondary weapons (x or X)
    BrowseBook,         // Read spellbook (b or P)
    JamDoor,            // Jam/spike door (j or S)
    Look,               // Look/examine surroundings (l or x)
    Inscribe,           // Add inscription to item ({)
    OpenDoor,           // Open door/chest (o)
    CastSpell,          // Mage spellcasting (m)
    Pray,               // Cleric prayers (p)
    ReadScroll,         // Read scroll (r)
    TakeOff,            // Unequip item (t or T)
    Fire,               // Fire missile weapon (f or t)
    MultiBlow,          // Multi-blow attack (M)
    Throw,              // Throw an item (v or t)
    UseStaff,           // Activate staff (u or Z)
    Eat,                // Eat food (E)
    Rest,               // Rest a while (R)
    Bash,               // Bash door/chest/monster (B or f)
    Tunnel,             // Tunnel/Mine (T or g)
    GoUpStairs,         // Go up stairs (<)
    GoDownStairs,       // Go down stairs (>)
    Quit,               // Retire / Quit character (Q)
    Help,               // Display help screen (?)
    RefillLight,        // Refill lamp/lantern (F)
    ToggleSearch,       // Toggle search mode (# or S)
    GainSpells,         // Study/learn magic spells or prayers (G)
    ReducedMap,         // Reduced-size map (M)
    LocateMap,          // Locate / pan map (L or W)
    IdentifySymbol,     // Identify symbol / recall monster (/)
    OptionsMenu,        // Options menu (=)
    HighScores,         // View high scores (V)
    Repeat(u32),        // Count repeat prefix
    WizardToggle,       // Toggle wizard mode (Ctrl-W)
    WizardCureAll,      // Wizard cure all (Ctrl-A)
    WizardEditChar,     // Wizard edit character (Ctrl-E)
    WizardMassGenocide, // Wizard mass genocide (Ctrl-F)
    WizardDropItems,    // Wizard drop random items (Ctrl-G)
    WizardJumpLevel,    // Wizard jump depth (Ctrl-D)
    WizardIdentify,     // Wizard identify (Ctrl-I)
    WizardLight,        // Wizard light dungeon (*)
    WizardMap,          // Wizard map current panel (:)
    WizardTeleport,     // Wizard teleport (Ctrl-T)
    WizardGainExp,      // Wizard gain exp (+)
    WizardSummon,       // Wizard summon monster (&)
    Unknown(char),
}

/// Keybinding Profiles / Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeybindingMode {
    Original,
    #[default]
    Roguelike,
    ModernWasd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardProfile {
    StandardQweasd,
    RoguelikeQweasd,
    OriginalKeys,
}

impl From<KeyboardProfile> for KeybindingMode {
    fn from(p: KeyboardProfile) -> Self {
        match p {
            KeyboardProfile::StandardQweasd => KeybindingMode::ModernWasd,
            KeyboardProfile::RoguelikeQweasd => KeybindingMode::Roguelike,
            KeyboardProfile::OriginalKeys => KeybindingMode::Original,
        }
    }
}

impl From<KeybindingMode> for KeyboardProfile {
    fn from(m: KeybindingMode) -> Self {
        match m {
            KeybindingMode::Original => KeyboardProfile::OriginalKeys,
            KeybindingMode::Roguelike => KeyboardProfile::RoguelikeQweasd,
            KeybindingMode::ModernWasd => KeyboardProfile::StandardQweasd,
        }
    }
}

pub struct InputMapper {
    pub mode: KeybindingMode,
}

impl InputMapper {
    pub fn new(mode: impl Into<KeybindingMode>) -> Self {
        Self { mode: mode.into() }
    }

    pub fn profile(&self) -> KeyboardProfile {
        self.mode.into()
    }

    /// Map a keypress to an Action based on the active KeybindingMode.
    pub fn map_key(&self, key: char) -> Action {
        match self.mode {
            KeybindingMode::Original => self.map_original(key),
            KeybindingMode::Roguelike => self.map_roguelike(key),
            KeybindingMode::ModernWasd => self.map_modern(key),
        }
    }

    /// Maps a direction key when expecting a direction input.
    pub fn map_direction_key(&self, key: char) -> Option<Direction> {
        map_direction_key(key, self.mode)
    }

    /// Maps a crossterm KeyCode when expecting a direction input.
    pub fn map_direction_code(&self, code: crossterm::event::KeyCode) -> Option<Direction> {
        map_direction_code(code, self.mode)
    }

    /// Maps a crossterm KeyCode and modifiers to an Action.
    pub fn map_event(
        &self,
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Action {
        match code {
            crossterm::event::KeyCode::Char(c) => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) && c.is_ascii_lowercase() {
                    self.map_key(c.to_ascii_uppercase())
                } else {
                    self.map_key(c)
                }
            }
            crossterm::event::KeyCode::Up => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::North)
                } else {
                    Action::Move(Direction::North)
                }
            }
            crossterm::event::KeyCode::Down => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::South)
                } else {
                    Action::Move(Direction::South)
                }
            }
            crossterm::event::KeyCode::Left => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::West)
                } else {
                    Action::Move(Direction::West)
                }
            }
            crossterm::event::KeyCode::Right => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::East)
                } else {
                    Action::Move(Direction::East)
                }
            }
            crossterm::event::KeyCode::Home => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::NorthWest)
                } else {
                    Action::Move(Direction::NorthWest)
                }
            }
            crossterm::event::KeyCode::PageUp => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::NorthEast)
                } else {
                    Action::Move(Direction::NorthEast)
                }
            }
            crossterm::event::KeyCode::End => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::SouthWest)
                } else {
                    Action::Move(Direction::SouthWest)
                }
            }
            crossterm::event::KeyCode::PageDown => {
                if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    Action::Run(Direction::SouthEast)
                } else {
                    Action::Move(Direction::SouthEast)
                }
            }
            _ => Action::Unknown(' '),
        }
    }

    fn map_original(&self, key: char) -> Action {
        match key {
            // Numeric Keypad / Number Movement (canonical Original)
            '1' => Action::Move(Direction::SouthWest),
            '2' => Action::Move(Direction::South),
            '3' => Action::Move(Direction::SouthEast),
            '4' => Action::Move(Direction::West),
            '5' => Action::Rest,
            '6' => Action::Move(Direction::East),
            '7' => Action::Move(Direction::NorthWest),
            '8' => Action::Move(Direction::North),
            '9' => Action::Move(Direction::NorthEast),

            // Capital directional keys in original mode are run commands
            'B' => Action::Run(Direction::SouthWest),
            'J' => Action::Run(Direction::South),
            'N' => Action::Run(Direction::SouthEast),
            'H' => Action::Run(Direction::West),
            'L' => Action::LocateMap,
            'Y' => Action::Run(Direction::NorthWest),
            'K' => Action::Run(Direction::North),
            'U' => Action::Run(Direction::NorthEast),

            // Canonical Original Commands
            'a' => Action::AimWand,
            'b' => Action::BrowseBook,
            'c' => Action::CloseDoor,
            'd' => Action::DropItem,
            'e' => Action::EquipmentList,
            'f' => Action::Bash,
            'h' => Action::Help,
            'i' => Action::InventoryList,
            'j' => Action::JamDoor,
            'l' => Action::Look,
            'm' => Action::CastSpell,
            'o' => Action::OpenDoor,
            'p' => Action::Pray,
            'q' => Action::Quaff,
            'r' => Action::ReadScroll,
            's' => Action::SearchOneTurn,
            't' => Action::Fire,
            'u' => Action::UseStaff,
            'w' => Action::WearWield,
            'x' => Action::ExchangeWeapon,

            'C' => Action::CharacterStats,
            'D' => Action::Disarm,
            'E' => Action::Eat,
            'F' => Action::RefillLight,
            'G' => Action::GainSpells,
            'M' => Action::ReducedMap,
            'P' => Action::BrowseBook,
            'Q' => Action::Quit,
            'R' => Action::Rest,
            'S' => Action::ToggleSearch,
            'T' => Action::Tunnel,
            'V' => Action::HighScores,
            'W' => Action::LocateMap,
            'X' => Action::ExchangeWeapon,

            '/' => Action::IdentifySymbol,
            '=' => Action::OptionsMenu,
            '{' => Action::Inscribe,
            '<' => Action::GoUpStairs,
            '>' => Action::GoDownStairs,
            '?' => Action::Help,
            '#' => Action::ToggleSearch,
            '.' => Action::Rest,

            other => Action::Unknown(other),
        }
    }

    fn map_roguelike(&self, key: char) -> Action {
        match key {
            // Movement (hjklyubn canonical roguelike)
            'h' => Action::Move(Direction::West),
            'j' => Action::Move(Direction::South),
            'k' => Action::Move(Direction::North),
            'l' => Action::Move(Direction::East),
            'y' => Action::Move(Direction::NorthWest),
            'u' => Action::Move(Direction::NorthEast),
            'b' => Action::Move(Direction::SouthWest),
            'n' => Action::Move(Direction::SouthEast),

            // Continuous Run (Shift+movement key)
            'H' => Action::Run(Direction::West),
            'J' => Action::Run(Direction::South),
            'K' => Action::Run(Direction::North),
            'L' => Action::Run(Direction::East),
            'Y' => Action::Run(Direction::NorthWest),
            'U' => Action::Run(Direction::NorthEast),
            'B' => Action::Run(Direction::SouthWest),
            'N' => Action::Run(Direction::SouthEast),

            // Keypad numbers also work for movement in Roguelike
            '1' => Action::Move(Direction::SouthWest),
            '2' => Action::Move(Direction::South),
            '3' => Action::Move(Direction::SouthEast),
            '4' => Action::Move(Direction::West),
            '5' => Action::Rest,
            '6' => Action::Move(Direction::East),
            '7' => Action::Move(Direction::NorthWest),
            '8' => Action::Move(Direction::North),
            '9' => Action::Move(Direction::NorthEast),

            // Remapped Actions (Roguelike mode conflicts resolved)
            'Q' => Action::Quit,
            'W' => Action::LocateMap,
            'w' => Action::WearWield,
            'E' => Action::Eat,
            'R' => Action::Rest,
            'f' => Action::Fire,
            'v' => Action::Throw,
            'M' => Action::ReducedMap,
            't' => Action::TakeOff,
            'T' => Action::Tunnel,
            'g' => Action::Tunnel,
            'F' => Action::RefillLight,
            '#' => Action::ToggleSearch,
            'a' | 'z' => Action::AimWand,
            'x' => Action::Look,
            'd' => Action::DropItem,
            'D' => Action::Disarm,
            'c' => Action::CloseDoor,
            'C' => Action::CharacterStats,
            'X' => Action::ExchangeWeapon,
            's' => Action::SearchOneTurn,
            'q' => Action::Quaff,
            'P' => Action::BrowseBook,
            'S' => Action::JamDoor,
            'Z' => Action::UseStaff,
            'V' => Action::HighScores,

            // Standard commands shared
            'i' => Action::InventoryList,
            'I' | 'e' => Action::EquipmentList,
            'p' => Action::Pray,
            'r' => Action::ReadScroll,
            'o' => Action::OpenDoor,
            'm' => Action::CastSpell,
            '{' => Action::Inscribe,
            '<' => Action::GoUpStairs,
            '>' => Action::GoDownStairs,
            '?' => Action::Help,
            'G' => Action::GainSpells,
            '/' => Action::IdentifySymbol,
            '=' => Action::OptionsMenu,
            '.' => Action::Rest,

            other => Action::Unknown(other),
        }
    }

    fn map_modern(&self, key: char) -> Action {
        match key {
            // Movement (qweasdzxc layout)
            'q' => Action::Move(Direction::NorthWest),
            'w' => Action::Move(Direction::North),
            'e' => Action::Move(Direction::NorthEast),
            'a' => Action::Move(Direction::West),
            's' => Action::Rest,
            'd' => Action::Move(Direction::East),
            'z' => Action::Move(Direction::SouthWest),
            'x' => Action::Move(Direction::South),
            'c' => Action::Move(Direction::SouthEast),

            // Keypad numbers also work for movement
            '1' => Action::Move(Direction::SouthWest),
            '2' => Action::Move(Direction::South),
            '3' => Action::Move(Direction::SouthEast),
            '4' => Action::Move(Direction::West),
            '5' => Action::Rest,
            '6' => Action::Move(Direction::East),
            '7' => Action::Move(Direction::NorthWest),
            '8' => Action::Move(Direction::North),
            '9' => Action::Move(Direction::NorthEast),

            // Remapped Actions (Standard mode conflicts resolved)
            'h' => Action::Quaff,
            'W' => Action::WearWield,
            'I' => Action::EquipmentList,
            'y' => Action::AimWand,
            'k' => Action::SearchOneTurn,
            'D' => Action::DropItem,
            'n' => Action::Disarm,
            'C' => Action::CloseDoor,
            'H' => Action::CharacterStats,
            'X' => Action::ExchangeWeapon,
            'E' => Action::Eat,
            'R' => Action::Rest,
            'f' => Action::Fire,
            'v' => Action::Throw,
            'M' => Action::ReducedMap,
            't' => Action::TakeOff,
            'T' => Action::Tunnel,
            'B' => Action::Bash,
            'g' => Action::Tunnel,
            'F' => Action::RefillLight,
            '#' => Action::ToggleSearch,
            'L' => Action::LocateMap,
            'V' => Action::HighScores,

            // Unchanged Standard Commands
            'b' => Action::BrowseBook,
            'j' => Action::JamDoor,
            'l' => Action::Look,
            'u' => Action::UseStaff,
            'i' => Action::InventoryList,
            'm' => Action::CastSpell,
            'o' => Action::OpenDoor,
            'p' => Action::Pray,
            'r' => Action::ReadScroll,
            '{' => Action::Inscribe,
            '<' => Action::GoUpStairs,
            '>' => Action::GoDownStairs,
            'Q' => Action::Quit,
            'G' => Action::GainSpells,
            '?' => Action::Help,
            '/' => Action::IdentifySymbol,
            '=' => Action::OptionsMenu,
            '.' => Action::Rest,

            other => Action::Unknown(other),
        }
    }
}

pub fn map_direction_key(key: char, mode: KeybindingMode) -> Option<Direction> {
    // Numeric keypad / 1..9 always works across all modes
    match key {
        '1' => return Some(Direction::SouthWest),
        '2' => return Some(Direction::South),
        '3' => return Some(Direction::SouthEast),
        '4' => return Some(Direction::West),
        '5' => return Some(Direction::Rest),
        '6' => return Some(Direction::East),
        '7' => return Some(Direction::NorthWest),
        '8' => return Some(Direction::North),
        '9' => return Some(Direction::NorthEast),
        _ => {}
    }

    match mode {
        KeybindingMode::Original | KeybindingMode::Roguelike => match key {
            'h' | 'H' => Some(Direction::West),
            'j' | 'J' => Some(Direction::South),
            'k' | 'K' => Some(Direction::North),
            'l' | 'L' => Some(Direction::East),
            'y' | 'Y' => Some(Direction::NorthWest),
            'u' | 'U' => Some(Direction::NorthEast),
            'b' | 'B' => Some(Direction::SouthWest),
            'n' | 'N' => Some(Direction::SouthEast),
            '.' => Some(Direction::Rest),
            _ => None,
        },
        KeybindingMode::ModernWasd => match key {
            'q' | 'Q' => Some(Direction::NorthWest),
            'w' | 'W' => Some(Direction::North),
            'e' | 'E' => Some(Direction::NorthEast),
            'a' | 'A' => Some(Direction::West),
            's' | 'S' => Some(Direction::Rest),
            'd' | 'D' => Some(Direction::East),
            'z' | 'Z' => Some(Direction::SouthWest),
            'x' | 'X' => Some(Direction::South),
            'c' | 'C' => Some(Direction::SouthEast),
            _ => None,
        },
    }
}

pub fn map_direction_code(
    code: crossterm::event::KeyCode,
    mode: KeybindingMode,
) -> Option<Direction> {
    match code {
        crossterm::event::KeyCode::Char(c) => map_direction_key(c, mode),
        crossterm::event::KeyCode::Up => Some(Direction::North),
        crossterm::event::KeyCode::Down => Some(Direction::South),
        crossterm::event::KeyCode::Left => Some(Direction::West),
        crossterm::event::KeyCode::Right => Some(Direction::East),
        crossterm::event::KeyCode::Home => Some(Direction::NorthWest),
        crossterm::event::KeyCode::PageUp => Some(Direction::NorthEast),
        crossterm::event::KeyCode::End => Some(Direction::SouthWest),
        crossterm::event::KeyCode::PageDown => Some(Direction::SouthEast),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_and_takeoff_key_mappings() {
        let std_mapper = InputMapper::new(KeyboardProfile::StandardQweasd);
        assert_eq!(std_mapper.map_key('T'), Action::Tunnel);
        assert_eq!(std_mapper.map_key('g'), Action::Tunnel);
        assert_eq!(std_mapper.map_key('t'), Action::TakeOff);
        assert_eq!(std_mapper.map_key('W'), Action::WearWield);
        assert_eq!(std_mapper.map_key('f'), Action::Fire);
        assert_eq!(std_mapper.map_key('v'), Action::Throw);

        let rl_mapper = InputMapper::new(KeyboardProfile::RoguelikeQweasd);
        assert_eq!(rl_mapper.map_key('T'), Action::Tunnel);
        assert_eq!(rl_mapper.map_key('g'), Action::Tunnel);
        assert_eq!(rl_mapper.map_key('t'), Action::TakeOff);
        assert_eq!(rl_mapper.map_key('W'), Action::LocateMap);
        assert_eq!(rl_mapper.map_key('f'), Action::Fire);
        assert_eq!(rl_mapper.map_key('v'), Action::Throw);
    }

    #[test]
    fn test_original_keys_profile() {
        let orig_mapper = InputMapper::new(KeybindingMode::Original);
        // All 9 keypad keys for movement & rest
        assert_eq!(orig_mapper.map_key('1'), Action::Move(Direction::SouthWest));
        assert_eq!(orig_mapper.map_key('2'), Action::Move(Direction::South));
        assert_eq!(orig_mapper.map_key('3'), Action::Move(Direction::SouthEast));
        assert_eq!(orig_mapper.map_key('4'), Action::Move(Direction::West));
        assert_eq!(orig_mapper.map_key('5'), Action::Rest);
        assert_eq!(orig_mapper.map_key('6'), Action::Move(Direction::East));
        assert_eq!(orig_mapper.map_key('7'), Action::Move(Direction::NorthWest));
        assert_eq!(orig_mapper.map_key('8'), Action::Move(Direction::North));
        assert_eq!(orig_mapper.map_key('9'), Action::Move(Direction::NorthEast));

        // Running commands
        assert_eq!(orig_mapper.map_key('B'), Action::Run(Direction::SouthWest));
        assert_eq!(orig_mapper.map_key('K'), Action::Run(Direction::North));
        assert_eq!(orig_mapper.map_key('H'), Action::Run(Direction::West));

        // Commands
        assert_eq!(orig_mapper.map_key('M'), Action::ReducedMap);
        assert_eq!(orig_mapper.map_key('L'), Action::LocateMap);
        assert_eq!(orig_mapper.map_key('V'), Action::HighScores);
        assert_eq!(orig_mapper.map_key('/'), Action::IdentifySymbol);
    }

    #[test]
    fn test_event_and_direction_code_mapping() {
        use crossterm::event::{KeyCode, KeyModifiers};

        let orig_mapper = InputMapper::new(KeybindingMode::Original);

        // Arrow keys in map_event
        assert_eq!(
            orig_mapper.map_event(KeyCode::Up, KeyModifiers::empty()),
            Action::Move(Direction::North)
        );
        assert_eq!(
            orig_mapper.map_event(KeyCode::Up, KeyModifiers::SHIFT),
            Action::Run(Direction::North)
        );
        assert_eq!(
            orig_mapper.map_event(KeyCode::Left, KeyModifiers::empty()),
            Action::Move(Direction::West)
        );
        assert_eq!(
            orig_mapper.map_event(KeyCode::Right, KeyModifiers::SHIFT),
            Action::Run(Direction::East)
        );
        assert_eq!(
            orig_mapper.map_event(KeyCode::Home, KeyModifiers::empty()),
            Action::Move(Direction::NorthWest)
        );
        assert_eq!(
            orig_mapper.map_event(KeyCode::PageDown, KeyModifiers::empty()),
            Action::Move(Direction::SouthEast)
        );

        // Direction code mapping in Original mode: keypad, arrows, and vi keys
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Char('8')),
            Some(Direction::North)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Char('k')),
            Some(Direction::North)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Char('h')),
            Some(Direction::West)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Up),
            Some(Direction::North)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Down),
            Some(Direction::South)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Left),
            Some(Direction::West)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Right),
            Some(Direction::East)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::Home),
            Some(Direction::NorthWest)
        );
        assert_eq!(
            orig_mapper.map_direction_code(KeyCode::End),
            Some(Direction::SouthWest)
        );
    }

    #[test]
    fn test_direction_key_mapping_across_modes() {
        assert_eq!(
            map_direction_key('4', KeybindingMode::Original),
            Some(Direction::West)
        );
        assert_eq!(
            map_direction_key('h', KeybindingMode::Roguelike),
            Some(Direction::West)
        );
        assert_eq!(
            map_direction_key('a', KeybindingMode::ModernWasd),
            Some(Direction::West)
        );
    }

    #[test]
    fn test_roguelike_and_modern_wasd_mappings() {
        let rl_mapper = InputMapper::new(KeybindingMode::Roguelike);
        assert_eq!(rl_mapper.map_key('h'), Action::Move(Direction::West));
        assert_eq!(rl_mapper.map_key('l'), Action::Move(Direction::East));
        assert_eq!(rl_mapper.map_key('H'), Action::Run(Direction::West));
        assert_eq!(rl_mapper.map_key('L'), Action::Run(Direction::East));
        assert_eq!(rl_mapper.map_key('/'), Action::IdentifySymbol);
        assert_eq!(rl_mapper.map_key('M'), Action::ReducedMap);
        assert_eq!(rl_mapper.map_key('V'), Action::HighScores);

        let wasd_mapper = InputMapper::new(KeybindingMode::ModernWasd);
        assert_eq!(wasd_mapper.map_key('w'), Action::Move(Direction::North));
        assert_eq!(wasd_mapper.map_key('a'), Action::Move(Direction::West));
        assert_eq!(wasd_mapper.map_key('d'), Action::Move(Direction::East));
        assert_eq!(wasd_mapper.map_key('W'), Action::WearWield);
        assert_eq!(wasd_mapper.map_key('M'), Action::ReducedMap);
        assert_eq!(wasd_mapper.map_key('V'), Action::HighScores);
    }

    #[test]
    fn test_direction_helpers() {
        assert_eq!(Direction::from_u8(1), Some(Direction::SouthWest));
        assert_eq!(Direction::from_u8(5), Some(Direction::Rest));
        assert_eq!(Direction::from_u8(8), Some(Direction::North));
        assert_eq!(Direction::from_u8(0), None);
        assert_eq!(Direction::from_u8(10), None);

        assert_eq!(Direction::North.to_dx_dy(), (0, -1));
        assert_eq!(Direction::South.to_dx_dy(), (0, 1));
        assert_eq!(Direction::West.to_dx_dy(), (-1, 0));
        assert_eq!(Direction::East.to_dx_dy(), (1, 0));
        assert_eq!(Direction::NorthWest.to_dx_dy(), (-1, -1));
        assert_eq!(Direction::NorthEast.to_dx_dy(), (1, -1));
        assert_eq!(Direction::SouthWest.to_dx_dy(), (-1, 1));
        assert_eq!(Direction::SouthEast.to_dx_dy(), (1, 1));
        assert_eq!(Direction::Rest.to_dx_dy(), (0, 0));
    }
}
