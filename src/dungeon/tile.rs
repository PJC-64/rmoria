use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    OpenPit,
    ArrowPit,
    CoveredPit,
    TrapDoor,
    SleepingGas,
    HiddenObject,
    DartOfStr,
    Teleport,
    Rockfall,
    CorrodingGas,
    SummonMonster,
    FireTrap,
    AcidTrap,
    PoisonGas,
    BlindGas,
    ConfuseGas,
    SlowDart,
    DartOfCon,
}

impl TrapType {
    pub const ALL: [TrapType; 18] = [
        TrapType::OpenPit,
        TrapType::ArrowPit,
        TrapType::CoveredPit,
        TrapType::TrapDoor,
        TrapType::SleepingGas,
        TrapType::HiddenObject,
        TrapType::DartOfStr,
        TrapType::Teleport,
        TrapType::Rockfall,
        TrapType::CorrodingGas,
        TrapType::SummonMonster,
        TrapType::FireTrap,
        TrapType::AcidTrap,
        TrapType::PoisonGas,
        TrapType::BlindGas,
        TrapType::ConfuseGas,
        TrapType::SlowDart,
        TrapType::DartOfCon,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            TrapType::OpenPit => "an open pit",
            TrapType::ArrowPit => "an arrow trap",
            TrapType::CoveredPit => "a covered pit",
            TrapType::TrapDoor => "a trap door",
            TrapType::SleepingGas => "a gas trap",
            TrapType::HiddenObject => "a loose rock",
            TrapType::DartOfStr => "a dart trap",
            TrapType::Teleport => "a strange rune",
            TrapType::Rockfall => "some loose rock",
            TrapType::CorrodingGas => "a gas trap",
            TrapType::SummonMonster => "a strange rune",
            TrapType::FireTrap => "a blackened spot",
            TrapType::AcidTrap => "some corroded rock",
            TrapType::PoisonGas => "a gas trap",
            TrapType::BlindGas => "a gas trap",
            TrapType::ConfuseGas => "a gas trap",
            TrapType::SlowDart => "a dart trap",
            TrapType::DartOfCon => "a dart trap",
        }
    }

    pub fn difficulty(&self) -> i32 {
        match self {
            TrapType::OpenPit => 1,
            TrapType::ArrowPit => 3,
            TrapType::CoveredPit => 2,
            TrapType::TrapDoor => 5,
            TrapType::SleepingGas => 3,
            TrapType::HiddenObject => 0,
            TrapType::DartOfStr => 5,
            TrapType::Teleport => 5,
            TrapType::Rockfall => 5,
            TrapType::CorrodingGas => 10,
            TrapType::SummonMonster => 5,
            TrapType::FireTrap => 10,
            TrapType::AcidTrap => 10,
            TrapType::PoisonGas => 5,
            TrapType::BlindGas => 5,
            TrapType::ConfuseGas => 5,
            TrapType::SlowDart => 5,
            TrapType::DartOfCon => 5,
        }
    }

    pub fn disarm_exp(&self) -> i32 {
        match self {
            TrapType::OpenPit => 50,
            TrapType::ArrowPit => 90,
            TrapType::CoveredPit => 60,
            TrapType::TrapDoor => 75,
            TrapType::SleepingGas => 95,
            TrapType::HiddenObject => 10,
            TrapType::DartOfStr => 110,
            TrapType::Teleport => 90,
            TrapType::Rockfall => 90,
            TrapType::CorrodingGas => 105,
            TrapType::SummonMonster => 90,
            TrapType::FireTrap => 110,
            TrapType::AcidTrap => 110,
            TrapType::PoisonGas => 105,
            TrapType::BlindGas => 110,
            TrapType::ConfuseGas => 105,
            TrapType::SlowDart => 110,
            TrapType::DartOfCon => 110,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Empty,
    Wall,
    Floor,
    DoorClosed { spikes: u32 },
    DoorOpen,
    StairsUp,
    StairsDown,
    ShopDoor(u8),
    SecretDoor,
    Trap { detected: bool, trap_type: TrapType },
    Chest { trapped: bool, locked: bool },
    MagmaVein { gold: bool },
    QuartzVein { gold: bool },
    Rubble,
    GlyphOfWarding,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub visible: bool,
    pub remembered: bool,
    pub is_lit: bool,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            visible: false,
            remembered: false,
            is_lit: false,
        }
    }

    pub fn is_passable(&self) -> bool {
        match self.tile_type {
            TileType::Floor
            | TileType::DoorOpen
            | TileType::StairsUp
            | TileType::StairsDown
            | TileType::ShopDoor(_)
            | TileType::GlyphOfWarding => true,
            TileType::Trap { .. } => true, // Traps are on passable floor tiles
            _ => false,
        }
    }
}
