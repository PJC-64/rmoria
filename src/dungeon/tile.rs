use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    Arrow,
    PoisonGas,
    Teleport,
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub visible: bool,
    pub remembered: bool,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            visible: false,
            remembered: false,
        }
    }

    pub fn is_passable(&self) -> bool {
        match self.tile_type {
            TileType::Floor | TileType::DoorOpen | TileType::StairsUp | TileType::StairsDown | TileType::ShopDoor(_) => true,
            TileType::Trap { .. } => true, // Traps are on passable floor tiles
            _ => false,
        }
    }
}
