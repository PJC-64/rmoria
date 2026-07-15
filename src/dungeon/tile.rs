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
            TileType::Floor | TileType::DoorOpen | TileType::StairsUp | TileType::StairsDown => true,
            _ => false,
        }
    }
}
