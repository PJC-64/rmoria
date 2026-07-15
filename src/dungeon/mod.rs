use serde::{Serialize, Deserialize};
use rand::Rng;

pub mod tile;

pub use tile::{Tile, TileType, TrapType};
use crate::player::{Item, ItemType};
use crate::dice::Dice;

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
pub struct RoomInfo {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub is_lit: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FloorItem {
    pub x: usize,
    pub y: usize,
    pub item: Item,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DungeonLevel {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub depth: u32,
    pub shops: Vec<ShopInfo>,
    pub max_depth: u32,
    pub rooms: Vec<RoomInfo>,
    pub items: Vec<FloorItem>,
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

pub fn generate_random_floor_item<R: Rng>(depth: u32, rng: &mut R) -> Item {
    let roll = rng.gen_range(0..100);
    if roll < 25 {
        let is_heal = rng.gen_bool(0.3);
        if is_heal && depth >= 3 {
            Item::new("Potion of Healing", 1, 5, ItemType::Potion { heal_amount: 25 })
        } else {
            Item::new("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 })
        }
    } else if roll < 50 {
        let book_roll = rng.gen_range(0..10);
        if book_roll == 0 {
            Item::new("Mage Spellbook [Beginner's Magick]", 1, 20, ItemType::Scroll { teleport: false })
        } else if book_roll == 1 {
            Item::new("Priest Prayerbook [Beginner's Handbook]", 1, 20, ItemType::Scroll { teleport: false })
        } else {
            let is_teleport = rng.gen_bool(0.3);
            if is_teleport && depth >= 2 {
                Item::new("Scroll of Teleportation", 1, 2, ItemType::Scroll { teleport: true })
            } else {
                Item::new("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true })
            }
        }
    } else if roll < 70 {
        let piece = rng.gen_range(0..4);
        match piece {
            0 => Item::new("Short Sword", 1, 120, ItemType::Weapon { damage: Dice::new(1, 6) }),
            1 => Item::new("Broadsword", 1, 150, ItemType::Weapon { damage: Dice::new(2, 5) }),
            2 => Item::new("Chain Mail", 1, 250, ItemType::Armor { ac: 7 }),
            _ => Item::new("Iron Shield", 1, 100, ItemType::Armor { ac: 3 }),
        }
    } else if roll < 85 {
        Item::new("Wooden Torch", 1, 15, ItemType::Scroll { teleport: false })
    } else {
        let gold_amount = rng.gen_range(15..=40) * (depth + 1);
        Item::new(&format!("Gold Pile [{} gp]", gold_amount), 1, 1, ItemType::Scroll { teleport: false })
    }
}

impl DungeonLevel {
    pub fn new(width: usize, height: usize, depth: u32, max_depth: u32) -> Self {
        let tiles = vec![Tile::new(TileType::Wall); width * height];
        Self { width, height, tiles, depth, shops: Vec::new(), max_depth, rooms: Vec::new(), items: Vec::new() }
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

    fn find_random_floor_tile(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..2000 {
            let tx = rng.gen_range(1..(self.width - 1));
            let ty = rng.gen_range(1..(self.height - 1));
            if let Some(tile) = self.get_tile(tx, ty) {
                if tile.tile_type == TileType::Floor {
                    return (tx, ty);
                }
            }
        }
        (30, 10)
    }

    pub fn generate_simple_floor(&mut self) {
        let mut rng = rand::thread_rng();
        self.items.clear();

        if self.depth == 0 {
            // --- TOWN LEVEL GENERATION ---
            self.rooms.clear();
            for y in 1..(self.height - 1) {
                for x in 1..(self.width - 1) {
                    if let Some(tile) = self.get_tile_mut(x, y) {
                        tile.tile_type = TileType::Floor;
                    }
                }
            }

            for x in 0..self.width {
                if let Some(t) = self.get_tile_mut(x, 0) { t.tile_type = TileType::Wall; }
                if let Some(t) = self.get_tile_mut(x, self.height - 1) { t.tile_type = TileType::Wall; }
            }
            for y in 0..self.height {
                if let Some(t) = self.get_tile_mut(0, y) { t.tile_type = TileType::Wall; }
                if let Some(t) = self.get_tile_mut(self.width - 1, y) { t.tile_type = TileType::Wall; }
            }

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

                    if sx <= stairs_x + 2 && sx + sw >= stairs_x - 2 &&
                       sy <= stairs_y + 2 && sy + sh >= stairs_y - 2 {
                        continue;
                    }

                    let mut overlaps = false;
                    for &(ox, oy, ow, oh) in &placed_rects {
                        if sx <= ox + ow + 1 && sx + sw + 1 >= ox &&
                           sy <= oy + oh + 1 && sy + sh + 1 >= oy {
                            overlaps = true;
                            break;
                        }
                    }

                    if !overlaps {
                        for y in sy..(sy + sh) {
                            for x in sx..(sx + sw) {
                                if let Some(tile) = self.get_tile_mut(x, y) {
                                    tile.tile_type = TileType::Wall;
                                }
                            }
                        }

                        let door_side = rng.gen_range(0..4);
                        let (door_x, door_y) = match door_side {
                            0 => (sx + sw / 2, sy + sh - 1),
                            1 => (sx + sw / 2, sy),
                            2 => (sx, sy + sh / 2),
                            _ => (sx + sw - 1, sy + sh / 2),
                        };

                        let shop_num = match shop_type {
                            ShopType::General => 1,
                            ShopType::Armory => 2,
                            ShopType::Weaponsmith => 3,
                            ShopType::Temple => 4,
                            ShopType::Alchemy => 5,
                            ShopType::Magic => 6,
                        };

                        if let Some(tile) = self.get_tile_mut(door_x, door_y) {
                            tile.tile_type = TileType::ShopDoor(shop_num);
                        }

                        self.shops.push(ShopInfo { door_x, door_y, shop_type });
                        placed_rects.push((sx, sy, sw, sh));
                        break;
                    }
                }
            }

            if let Some(tile) = self.get_tile_mut(stairs_x, stairs_y) {
                tile.tile_type = TileType::StairsDown;
            }
        } else {
            // --- DUNGEON LEVEL GENERATION ---
            self.rooms.clear();
            for tile in self.tiles.iter_mut() {
                tile.tile_type = TileType::Wall;
            }

            let mut rooms_temp: Vec<Room> = Vec::new();
            for _ in 0..15 {
                let w = rng.gen_range(5..=10);
                let h = rng.gen_range(4..=7);
                let rx = rng.gen_range(2..(self.width - w - 2));
                let ry = rng.gen_range(2..(self.height - h - 2));

                let new_room = Room::new(rx, ry, w, h);
                let mut overlaps = false;
                for r in &rooms_temp {
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

                    let lit_chance = 1.0 - (self.depth as f64 / 10.0).min(1.0);
                    let is_lit = rng.gen_bool(lit_chance);

                    self.rooms.push(RoomInfo { x: rx, y: ry, w, h, is_lit });
                    rooms_temp.push(new_room);
                }
                if rooms_temp.len() >= 6 {
                    break;
                }
            }

            for i in 0..(rooms_temp.len() - 1) {
                let (x1, y1) = rooms_temp[i].center();
                let (x2, y2) = rooms_temp[i+1].center();

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
                                        // 25% chance this door is a Secret Door!
                                        if rng.gen_bool(0.25) {
                                            mut_tile.tile_type = TileType::SecretDoor;
                                        } else {
                                            mut_tile.tile_type = TileType::DoorClosed;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- SPAWN TRAPS ---
            let num_traps = rng.gen_range(3..=6);
            for _ in 0..num_traps {
                let (tx, ty) = self.find_random_floor_tile();
                let trap_type = match rng.gen_range(0..3) {
                    0 => TrapType::Arrow,
                    1 => TrapType::PoisonGas,
                    _ => TrapType::Teleport,
                };
                if let Some(tile) = self.get_tile_mut(tx, ty) {
                    tile.tile_type = TileType::Trap { detected: false, trap_type };
                }
            }

            // --- SPAWN RANDOM ITEMS ---
            let num_items = rng.gen_range(4..=8);
            for _ in 0..num_items {
                let (ix, iy) = self.find_random_floor_tile();
                let item = generate_random_floor_item(self.depth, &mut rng);
                self.items.push(FloorItem { x: ix, y: iy, item });
            }

            let (su_x, su_y) = rooms_temp[0].center();
            if let Some(tile) = self.get_tile_mut(su_x, su_y) {
                tile.tile_type = TileType::StairsUp;
            }

            if self.depth < self.max_depth {
                let (sd_x, sd_y) = rooms_temp[rooms_temp.len() - 1].center();
                if let Some(tile) = self.get_tile_mut(sd_x, sd_y) {
                    tile.tile_type = TileType::StairsDown;
                }
            }
        }
    }
}
