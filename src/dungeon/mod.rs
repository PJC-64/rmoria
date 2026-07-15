use serde::{Serialize, Deserialize};
use rand::Rng;

pub mod tile;

pub use tile::{Tile, TileType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopType {
    General,
    Armory,
    Weaponsmith,
    Temple,
    Alchemy,
    Magic,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShopInfo {
    pub door_x: usize,
    pub door_y: usize,
    pub shop_type: ShopType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DungeonLevel {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub depth: u32,
    pub shops: Vec<ShopInfo>,
}

struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Room {
    fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.w && self.x + self.w >= other.x &&
        self.y <= other.y + other.h && self.y + self.h >= other.y
    }
}

impl DungeonLevel {
    pub fn new(width: usize, height: usize, depth: u32) -> Self {
        let tiles = vec![Tile::new(TileType::Wall); width * height];
        Self { width, height, tiles, depth, shops: Vec::new() }
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

    fn carve_h_corridor(&mut self, x1: usize, x2: usize, y: usize) {
        let start = x1.min(x2);
        let end = x1.max(x2);
        for x in start..=end {
            if let Some(tile) = self.get_tile_mut(x, y) {
                if tile.tile_type == TileType::Wall {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
    }

    fn carve_v_corridor(&mut self, y1: usize, y2: usize, x: usize) {
        let start = y1.min(y2);
        let end = y1.max(y2);
        for y in start..=end {
            if let Some(tile) = self.get_tile_mut(x, y) {
                if tile.tile_type == TileType::Wall {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
    }

    /// Generates the dungeon level based on depth.
    /// Town Level (depth 0) generates shops and streets.
    /// Deep Levels (depth > 0) generate random rooms, corridors, and doorways.
    pub fn generate_simple_floor(&mut self) {
        let mut rng = rand::thread_rng();

        if self.depth == 0 {
            // --- TOWN LEVEL GENERATION ---
            // 1. Fill entire Town with Floor
            for y in 1..(self.height - 1) {
                for x in 1..(self.width - 1) {
                    if let Some(tile) = self.get_tile_mut(x, y) {
                        tile.tile_type = TileType::Floor;
                    }
                }
            }

            // 2. Add outer border walls
            for x in 0..self.width {
                if let Some(t) = self.get_tile_mut(x, 0) { t.tile_type = TileType::Wall; }
                if let Some(t) = self.get_tile_mut(x, self.height - 1) { t.tile_type = TileType::Wall; }
            }
            for y in 0..self.height {
                if let Some(t) = self.get_tile_mut(0, y) { t.tile_type = TileType::Wall; }
                if let Some(t) = self.get_tile_mut(self.width - 1, y) { t.tile_type = TileType::Wall; }
            }

            // 3. Place shops as distinct buildings at random locations
            self.shops.clear();
            let shop_types = vec![
                ShopType::General,
                ShopType::Armory,
                ShopType::Weaponsmith,
                ShopType::Temple,
                ShopType::Alchemy,
                ShopType::Magic,
            ];

            let mut placed_rects: Vec<(usize, usize, usize, usize)> = Vec::new();
            let stairs_x = 33;
            let stairs_y = 11;

            for shop_type in shop_types {
                for _ in 0..150 {
                    let sw = rng.gen_range(4..=6);
                    let sh = rng.gen_range(4..=6);
                    let sx = rng.gen_range(2..(self.width - sw - 2));
                    let sy = rng.gen_range(2..(self.height - sh - 2));

                    // Avoid stairs area
                    if sx <= stairs_x + 2 && sx + sw >= stairs_x - 2 &&
                       sy <= stairs_y + 2 && sy + sh >= stairs_y - 2 {
                        continue;
                    }

                    // Avoid overlapping existing shops
                    let mut overlaps = false;
                    for &(ox, oy, ow, oh) in &placed_rects {
                        if sx <= ox + ow + 1 && sx + sw + 1 >= ox &&
                           sy <= oy + oh + 1 && sy + sh + 1 >= oy {
                            overlaps = true;
                            break;
                        }
                    }

                    if !overlaps {
                        // Carve shop building walls
                        for y in sy..(sy + sh) {
                            for x in sx..(sx + sw) {
                                if let Some(tile) = self.get_tile_mut(x, y) {
                                    tile.tile_type = TileType::Wall;
                                }
                            }
                        }

                        // Pick a random side of the shop building to place a closed door (+)
                        let door_side = rng.gen_range(0..4);
                        let (door_x, door_y) = match door_side {
                            0 => (sx + sw / 2, sy + sh - 1), // bottom wall
                            1 => (sx + sw / 2, sy),          // top wall
                            2 => (sx, sy + sh / 2),          // left wall
                            _ => (sx + sw - 1, sy + sh / 2),  // right wall
                        };

                        if let Some(tile) = self.get_tile_mut(door_x, door_y) {
                            tile.tile_type = TileType::DoorClosed;
                        }

                        self.shops.push(ShopInfo { door_x, door_y, shop_type });
                        placed_rects.push((sx, sy, sw, sh));
                        break;
                    }
                }
            }

            // 4. Place Stairs Down (>) in the center of town
            if let Some(tile) = self.get_tile_mut(stairs_x, stairs_y) {
                tile.tile_type = TileType::StairsDown;
            }
        } else {
            // --- DUNGEON LEVEL GENERATION ---
            for tile in self.tiles.iter_mut() {
                tile.tile_type = TileType::Wall;
            }

            let mut rooms: Vec<Room> = Vec::new();
            for _ in 0..15 {
                let w = rng.gen_range(5..=10);
                let h = rng.gen_range(4..=7);
                let rx = rng.gen_range(2..(self.width - w - 2));
                let ry = rng.gen_range(2..(self.height - h - 2));

                let new_room = Room::new(rx, ry, w, h);
                let mut overlaps = false;
                for r in &rooms {
                    if new_room.intersects(r) {
                        overlaps = true;
                        break;
                    }
                }

                if !overlaps {
                    for y in ry..(ry + h) {
                        for x in rx..(rx + w) {
                            if let Some(tile) = self.get_tile_mut(x, y) {
                                tile.tile_type = TileType::Floor;
                            }
                        }
                    }
                    rooms.push(new_room);
                }
                if rooms.len() >= 6 {
                    break;
                }
            }

            for i in 0..(rooms.len() - 1) {
                let (x1, y1) = rooms[i].center();
                let (x2, y2) = rooms[i+1].center();

                if rng.gen_bool(0.5) {
                    self.carve_h_corridor(x1, x2, y1);
                    self.carve_v_corridor(y1, y2, x2);
                } else {
                    self.carve_v_corridor(y1, y2, x1);
                    self.carve_h_corridor(x1, x2, y2);
                }
            }

            for y in 2..(self.height - 2) {
                for x in 2..(self.width - 2) {
                    if let Some(tile) = self.get_tile(x, y) {
                        if tile.tile_type == TileType::Floor {
                            let w_l = self.get_tile(x - 1, y).map(|t| t.tile_type == TileType::Wall).unwrap_or(false);
                            let w_r = self.get_tile(x + 1, y).map(|t| t.tile_type == TileType::Wall).unwrap_or(false);
                            let f_t = self.get_tile(x, y - 1).map(|t| t.tile_type == TileType::Floor).unwrap_or(false);
                            let f_b = self.get_tile(x, y + 1).map(|t| t.tile_type == TileType::Floor).unwrap_or(false);

                            let w_t = self.get_tile(x, y - 1).map(|t| t.tile_type == TileType::Wall).unwrap_or(false);
                            let w_b = self.get_tile(x, y + 1).map(|t| t.tile_type == TileType::Wall).unwrap_or(false);
                            let f_l = self.get_tile(x - 1, y).map(|t| t.tile_type == TileType::Floor).unwrap_or(false);
                            let f_r = self.get_tile(x + 1, y).map(|t| t.tile_type == TileType::Floor).unwrap_or(false);

                            if (w_l && w_r && f_t && f_b) || (w_t && w_b && f_l && f_r) {
                                if rng.gen_bool(0.35) {
                                    if let Some(mut_tile) = self.get_tile_mut(x, y) {
                                        mut_tile.tile_type = TileType::DoorClosed;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let (su_x, su_y) = rooms[0].center();
            if let Some(tile) = self.get_tile_mut(su_x, su_y) {
                tile.tile_type = TileType::StairsUp;
            }

            if self.depth < 50 {
                let (sd_x, sd_y) = rooms[rooms.len() - 1].center();
                if let Some(tile) = self.get_tile_mut(sd_x, sd_y) {
                    tile.tile_type = TileType::StairsDown;
                }
            }
        }
    }
}
