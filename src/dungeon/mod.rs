use serde::{Serialize, Deserialize};

pub mod tile;

pub use tile::{Tile, TileType};

#[derive(Serialize, Deserialize, Clone)]
pub struct DungeonLevel {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub depth: u32,
}

impl DungeonLevel {
    pub fn new(width: usize, height: usize, depth: u32) -> Self {
        let tiles = vec![Tile::new(TileType::Wall); width * height];
        Self { width, height, tiles, depth }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            Some(&self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            Some(&mut self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    /// Generates a simple floor map and places Stairs Up (<) and Stairs Down (>)
    pub fn generate_simple_floor(&mut self) {
        // 1. Fill interior with Floor
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                if let Some(tile) = self.get_tile_mut(x, y) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }

        // 2. Add some random pillars/obstacles
        if let Some(tile) = self.get_tile_mut(10, 5) { tile.tile_type = TileType::Wall; }
        if let Some(tile) = self.get_tile_mut(10, 6) { tile.tile_type = TileType::Wall; }
        if let Some(tile) = self.get_tile_mut(25, 8) { tile.tile_type = TileType::Wall; }
        if let Some(tile) = self.get_tile_mut(25, 9) { tile.tile_type = TileType::Wall; }
        if let Some(tile) = self.get_tile_mut(50, 16) { tile.tile_type = TileType::Wall; }
        if let Some(tile) = self.get_tile_mut(50, 17) { tile.tile_type = TileType::Wall; }

        // 3. Place Stairs Down (>) if not at maximum depth (e.g. 50)
        if self.depth < 50 {
            if let Some(tile) = self.get_tile_mut(40, 15) {
                tile.tile_type = TileType::StairsDown;
            }
        }

        // 4. Place Stairs Up (<) if not on Town level (depth > 0)
        if self.depth > 0 {
            if let Some(tile) = self.get_tile_mut(15, 15) {
                tile.tile_type = TileType::StairsUp;
            }
        } else {
            // On Town Level (depth 0), place General Store door (+) at (25, 5)
            if let Some(tile) = self.get_tile_mut(25, 5) {
                tile.tile_type = TileType::DoorClosed;
            }
        }
    }
}
