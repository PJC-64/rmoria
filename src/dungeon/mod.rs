pub mod tile;

pub use tile::{Tile, TileType};

pub struct DungeonLevel {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl DungeonLevel {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![Tile::new(TileType::Wall); width * height];
        Self { width, height, tiles }
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

    pub fn generate_simple_room(&mut self, start_x: usize, start_y: usize, w: usize, h: usize) {
        for y in start_y..(start_y + h) {
            for x in start_x..(start_x + w) {
                if let Some(tile) = self.get_tile_mut(x, y) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
    }
}
