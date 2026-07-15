use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Empty,
    Wall,
    Floor,
    DoorClosed,
    DoorOpen,
    StairsUp,
    StairsDown,
    ShopDoor(u8), // Numbered door representation (1 to 6)
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
            _ => false,
        }
    }
}
