use serde::{Serialize, Deserialize};
use rand::Rng;

pub mod tile;
pub mod shop;

pub use tile::{Tile, TileType, TrapType};
pub use shop::{HaggleState, handle_haggle_input};
use crate::player::{Item, ItemType, Player};
use crate::dice::Dice;
use crate::entity::monster::{Monster, MonsterTemplate, MONSTER_DB};

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
    if roll < 20 {
        let potion_choice = rng.gen_range(0..10);
        match potion_choice {
            0..=2 => Item::new_unidentified("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 }),
            3 | 4 if depth >= 2 => Item::new_unidentified("Potion of Healing", 1, 5, ItemType::Potion { heal_amount: 25 }),
            5 => Item::new_unidentified("Potion of Cure Poison", 1, 5, ItemType::Potion { heal_amount: 0 }),
            6 => Item::new_unidentified("Potion of Speed", 1, 5, ItemType::Potion { heal_amount: 0 }),
            7 => Item::new_unidentified("Potion of Heroism", 1, 5, ItemType::Potion { heal_amount: 10 }),
            8 => Item::new_unidentified("Potion of Restore Strength", 1, 5, ItemType::Potion { heal_amount: 0 }),
            _ => Item::new_unidentified("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 }),
        }
    } else if roll < 40 {
        let book_roll = rng.gen_range(0..10);
        if book_roll == 0 {
            Item::new("Mage Spellbook [Beginner's Magick]", 1, 20, ItemType::Scroll { teleport: false })
        } else if book_roll == 1 {
            Item::new("Priest Prayerbook [Beginner's Handbook]", 1, 20, ItemType::Scroll { teleport: false })
        } else {
            let scroll_roll = rng.gen_range(0..10);
            match scroll_roll {
                0..=2 => Item::new_unidentified("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true }),
                3..=4 => Item::new_unidentified("Scroll of Teleportation", 1, 2, ItemType::Scroll { teleport: true }),
                5..=7 => Item::new_unidentified("Scroll of Identify", 1, 2, ItemType::Scroll { teleport: false }),
                _ => Item::new_unidentified("Scroll of Word of Recall", 1, 2, ItemType::Scroll { teleport: false }),
            }
        }
    } else if roll < 55 {
        let piece = rng.gen_range(0..8);
        match piece {
            0 => Item::new("Short Sword", 1, 120, ItemType::Weapon { damage: Dice::new(1, 6) }),
            1 => Item::new("Broadsword", 1, 150, ItemType::Weapon { damage: Dice::new(2, 5) }),
            2 => Item::new("Chain Mail", 1, 250, ItemType::Armor { ac: 7 }),
            3 => Item::new("Iron Shield", 1, 100, ItemType::Armor { ac: 3 }),
            4 => Item::new("Short Bow", 1, 30, ItemType::Bow { multiplier: 2 }),
            5 => Item::new("Arrow", rng.gen_range(10..=25), 2, ItemType::Missile { damage: Dice::new(1, 4) }),
            6 => Item::new("Light Crossbow", 1, 110, ItemType::Bow { multiplier: 3 }),
            _ => Item::new("Bolt", rng.gen_range(10..=25), 3, ItemType::Missile { damage: Dice::new(1, 5) }),
        }
    } else if roll < 68 {
        let is_wand = rng.gen_bool(0.5);
        if is_wand {
            Item::new("Wand of Magic Missile [5 charges]", 1, 10, ItemType::Wand { charges: 5, spell_index: 0 })
        } else {
            Item::new("Staff of Cure Light Wounds [3 charges]", 1, 12, ItemType::Staff { charges: 3, prayer_index: 1 })
        }
    } else if roll < 78 {
        Item::new("Ration of Food", 1, 10, ItemType::Food { nutrition: 5000 })
    } else if roll < 87 {
        Item::new("Wooden Torch", 1, 15, ItemType::Light { fuel: 4000 })
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
            if let Some(tile) = self.get_tile_mut(x, y)
                && tile.tile_type == TileType::Wall {
                    tile.tile_type = TileType::Floor;
                }
        }
    }

    fn carve_v_corridor(&mut self, y1: usize, y2: usize, x: usize) {
        let start = y1.min(y2);
        let end = y1.max(y2);
        for y in start..=end {
            if let Some(tile) = self.get_tile_mut(x, y)
                && tile.tile_type == TileType::Wall {
                    tile.tile_type = TileType::Floor;
                }
        }
    }

    pub fn is_in_room(&self, x: usize, y: usize) -> bool {
        self.rooms.iter().any(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    }

    pub fn room_index_at(&self, x: usize, y: usize) -> Option<usize> {
        self.rooms.iter().position(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    }

    pub fn is_room_entrance(&self, x: usize, y: usize) -> bool {
        if self.is_in_room(x, y) {
            return false;
        }

        let is_wall = |tt: TileType| matches!(tt, TileType::Wall | TileType::MagmaVein { .. } | TileType::QuartzVein { .. });
        let is_passage = |tt: TileType| !is_wall(tt);

        if let Some(tile) = self.get_tile(x, y) {
            if !is_passage(tile.tile_type) {
                return false;
            }
        } else {
            return false;
        }

        let w_l = self.get_tile(x - 1, y).map(|t| is_wall(t.tile_type)).unwrap_or(false);
        let w_r = self.get_tile(x + 1, y).map(|t| is_wall(t.tile_type)).unwrap_or(false);
        let f_t = self.get_tile(x, y - 1).map(|t| is_passage(t.tile_type)).unwrap_or(false);
        let f_b = self.get_tile(x, y + 1).map(|t| is_passage(t.tile_type)).unwrap_or(false);

        let w_t = self.get_tile(x, y - 1).map(|t| is_wall(t.tile_type)).unwrap_or(false);
        let w_b = self.get_tile(x, y + 1).map(|t| is_wall(t.tile_type)).unwrap_or(false);
        let f_l = self.get_tile(x - 1, y).map(|t| is_passage(t.tile_type)).unwrap_or(false);
        let f_r = self.get_tile(x + 1, y).map(|t| is_passage(t.tile_type)).unwrap_or(false);

        if w_l && w_r && f_t && f_b {
            let r_t = self.room_index_at(x, y - 1);
            let r_b = self.room_index_at(x, y + 1);
            (r_t.is_some() || r_b.is_some()) && r_t != r_b
        } else if w_t && w_b && f_l && f_r {
            let r_l = self.room_index_at(x - 1, y);
            let r_r = self.room_index_at(x + 1, y);
            (r_l.is_some() || r_r.is_some()) && r_l != r_r
        } else {
            false
        }
    }

    pub fn find_random_floor_tile(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..2000 {
            let tx = rng.gen_range(1..(self.width - 1));
            let ty = rng.gen_range(1..(self.height - 1));
            if let Some(tile) = self.get_tile(tx, ty)
                && tile.tile_type == TileType::Floor {
                    return (tx, ty);
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

            // Only place doors at the entrance to rooms, never in corridors
            for y in 2..(self.height - 2) {
                for x in 2..(self.width - 2) {
                    if let Some(tile) = self.get_tile(x, y)
                        && tile.tile_type == TileType::Floor
                        && self.is_room_entrance(x, y) {
                            // 85% chance a room entrance has a door (15% remains an open entryway)
                            if rng.gen_bool(0.85)
                                && let Some(mut_tile) = self.get_tile_mut(x, y) {
                                    let roll = rng.gen_range(0..100);
                                    if roll < 20 {
                                        // 20% secret door
                                        mut_tile.tile_type = TileType::SecretDoor;
                                    } else if roll < 35 {
                                        // 15% open door
                                        mut_tile.tile_type = TileType::DoorOpen;
                                    } else {
                                        // 65% closed door
                                        mut_tile.tile_type = TileType::DoorClosed { spikes: 0 };
                                    }
                                }
                        }
                }
            }

            // Spawn Magma and Quartz veins
            let num_veins = rng.gen_range(4..=8);
            for _ in 0..num_veins {
                let is_magma = rng.gen_bool(0.50);
                let mut vx = rng.gen_range(1..(self.width - 2));
                let mut vy = rng.gen_range(1..(self.height - 2));
                let vein_len = rng.gen_range(6..=14);
                for _ in 0..vein_len {
                    if let Some(tile) = self.get_tile_mut(vx, vy)
                        && tile.tile_type == TileType::Wall {
                            let has_gold = rng.gen_bool(0.25);
                            tile.tile_type = if is_magma {
                                TileType::MagmaVein { gold: has_gold }
                            } else {
                                TileType::QuartzVein { gold: has_gold }
                            };
                        }
                    let dx = rng.gen_range(-1..=1);
                    let dy = rng.gen_range(-1..=1);
                    vx = ((vx as isize + dx).max(1).min(self.width as isize - 2)) as usize;
                    vy = ((vy as isize + dy).max(1).min(self.height as isize - 2)) as usize;
                }
            }

            // Spawn Rubble (avoid blocking room entrances)
            let num_rubble = rng.gen_range(3..=6);
            for _ in 0..num_rubble {
                let (rx, ry) = self.find_random_floor_tile();
                if !self.is_room_entrance(rx, ry)
                    && let Some(tile) = self.get_tile_mut(rx, ry)
                    && tile.tile_type == TileType::Floor {
                        tile.tile_type = TileType::Rubble;
                    }
            }

            // --- SPAWN TRAPS ---
            let num_traps = rng.gen_range(3..=6);
            for _ in 0..num_traps {
                let (tx, ty) = self.find_random_floor_tile();
                let trap_type = match rng.gen_range(0..6) {
                    0 => TrapType::Arrow,
                    1 => TrapType::PoisonGas,
                    2 => TrapType::Teleport,
                    3 => TrapType::SleepingGas,
                    4 => TrapType::BlindGas,
                    _ => TrapType::ConfusionGas,
                };
                if let Some(tile) = self.get_tile_mut(tx, ty) {
                    tile.tile_type = TileType::Trap { detected: false, trap_type };
                }
            }

            // --- SPAWN CHESTS ---
            if self.depth > 0 {
                let num_chests = rng.gen_range(1..=2);
                for _ in 0..num_chests {
                    let (cx, cy) = self.find_random_floor_tile();
                    if !self.is_room_entrance(cx, cy)
                        && let Some(tile) = self.get_tile_mut(cx, cy) {
                            let trapped = rng.gen_bool(0.60);
                            let locked = rng.gen_bool(0.50);
                            tile.tile_type = TileType::Chest { trapped, locked };
                        }
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

    pub fn has_los(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = (y2 as isize - y1 as isize).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        
        let mut cx = x1 as isize;
        let mut cy = y1 as isize;
        
        loop {
            if cx == x2 as isize && cy == y2 as isize {
                return true;
            }
            if (cx != x1 as isize || cy != y1 as isize)
                && let Some(tile) = self.get_tile(cx as usize, cy as usize)
                    && !tile.is_passable() && tile.tile_type != TileType::SecretDoor {
                        return false;
                    }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                cx += sx;
            }
            if e2 < dx {
                err += dx;
                cy += sy;
            }
        }
    }

    pub fn update_fov(&mut self, player: &Player) {
        for tile in self.tiles.iter_mut() {
            tile.visible = false;
        }

        if player.flags.blind > 0 {
            if let Some(tile) = self.get_tile_mut(player.x, player.y) {
                tile.visible = true;
            }
            return;
        }
        
        if self.depth == 0 {
            for tile in self.tiles.iter_mut() {
                tile.visible = true;
                tile.remembered = true;
            }
            return;
        }
        
        let mut lit_rooms_to_reveal = Vec::new();
        for room in &self.rooms {
            if room.is_lit {
                let player_inside = player.x >= room.x && player.x < room.x + room.w &&
                                    player.y >= room.y && player.y < room.y + room.h;
                if player_inside {
                    lit_rooms_to_reveal.push(room.clone());
                }
            }
        }
        
        let radius = player.get_light_radius();
        let max_dist = 8;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut is_revealed_by_room = false;
                for room in &lit_rooms_to_reveal {
                    if x >= room.x - 1 && x <= room.x + room.w &&
                       y >= room.y - 1 && y <= room.y + room.h {
                        is_revealed_by_room = true;
                        break;
                    }
                }

                if is_revealed_by_room {
                    if let Some(tile) = self.get_tile_mut(x, y) {
                        tile.visible = true;
                        tile.remembered = true;
                    }
                    continue;
                }

                let dx = (x as isize - player.x as isize).abs();
                let dy = (y as isize - player.y as isize).abs();
                let dist = dx.max(dy) as usize;
                
                if dist <= max_dist
                    && self.has_los(player.x, player.y, x, y) {
                        if dist <= radius {
                            if let Some(tile) = self.get_tile_mut(x, y) {
                                tile.visible = true;
                                tile.remembered = true;
                            }
                        } else {
                            if let Some(tile) = self.get_tile_mut(x, y) {
                                tile.remembered = true;
                            }
                        }
                    }
            }
        }
    }

    pub fn generate_monsters(&self, player_has_killed_balrog: bool) -> Vec<Monster> {
        let depth = self.depth;
        let max_depth = self.max_depth;
        
        if depth == 0 {
            let mut rng = rand::thread_rng();
            let town_templates = &[
                ("Filthy Street Urchin", 6, Dice::new(1, 2)),
                ("Blubbering Idiot", 2, Dice::new(1, 1)),
                ("Pitiful-Looking Beggar", 4, Dice::new(1, 2)),
                ("Mangy-Looking Leper", 1, Dice::new(1, 1)),
                ("Squint-Eyed Rogue", 10, Dice::new(1, 4)),
                ("Singing, Happy Drunk", 4, Dice::new(1, 3)),
                ("Mean-Looking Mercenary", 25, Dice::new(2, 4)),
                ("Battle-Scarred Veteran", 35, Dice::new(2, 6)),
            ];

            let mut mons = Vec::new();
            for _ in 0..3 {
                let (tx, ty) = self.find_random_floor_tile();
                let r_idx = rng.gen_range(0..town_templates.len());
                let (name, hp, dmg) = town_templates[r_idx];
                mons.push(Monster::new(name, 'p', tx, ty, hp, dmg, 0));
            }
            return mons;
        }
        
        let mut mons = Vec::new();
        let mut rng = rand::thread_rng();
        
        if depth == max_depth {
            if !player_has_killed_balrog {
                let (bx, by) = self.find_random_floor_tile();
                mons.push(Monster::new("The Balrog", 'B', bx, by, 120, Dice::new(3, 8), 500));
            }
            for _ in 0..3 {
                let (gx, gy) = self.find_random_floor_tile();
                mons.push(Monster::new("Lich Guardian", 'L', gx, gy, 55, Dice::new(3, 6), 180));
            }
            return mons;
        }
        
        let mut active_level = depth;
        if rng.gen_bool(0.10) {
            active_level += rng.gen_range(1..=3);
        }
        
        for _ in 0..4 {
            let suitable_templates: Vec<&MonsterTemplate> = MONSTER_DB.iter()
                .filter(|t| t.level <= active_level)
                .collect();
                
            if !suitable_templates.is_empty() {
                let r_idx = rng.gen_range(0..suitable_templates.len());
                let t = suitable_templates[r_idx];
                let (mx, my) = self.find_random_floor_tile();
                mons.push(Monster::new(t.name, t.symbol, mx, my, t.max_hp, t.damage, t.exp_reward));
            }
        }
        
        mons
    }

    pub fn trigger_chest_trap(
        &mut self,
        px: usize,
        py: usize,
        player: &mut Player,
        rng: &mut impl rand::Rng,
        status_msg: &mut String,
        monsters: &mut Vec<Monster>,
    ) {
        match rng.gen_range(0..3) {
            0 => {
                let dmg = rng.gen_range(5..=40);
                player.hp -= dmg;
                status_msg.push_str(&format!(" The chest explodes! You take {} damage.", dmg));
            }
            1 => {
                let dmg = rng.gen_range(2..=16);
                player.hp -= dmg;
                status_msg.push_str(&format!(" The chest sprays poison gas! You take {} poison damage.", dmg));
            }
            _ => {
                status_msg.push_str(" The chest summons a monster!");
                let adj_tiles = &[(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)];
                let mut spawned = false;
                for &(dx, dy) in adj_tiles {
                    let sx = (px as isize + dx) as usize;
                    let sy = (py as isize + dy) as usize;
                    if let Some(t) = self.get_tile(sx, sy)
                        && t.is_passable() && monsters.iter().all(|m| m.x != sx || m.y != sy) {
                            let m_idx = rng.gen_range(0..MONSTER_DB.len());
                            let template = &MONSTER_DB[m_idx];
                            monsters.push(Monster::new(
                                template.name,
                                template.symbol,
                                sx,
                                sy,
                                template.max_hp,
                                template.damage,
                                template.exp_reward,
                            ));
                            spawned = true;
                            break;
                        }
                }
                if !spawned {
                    status_msg.push_str(" But there was no room to spawn.");
                }
            }
        }
    }

    pub fn open_chest(
        &mut self,
        cx: usize,
        cy: usize,
        player: &mut Player,
        rng: &mut impl rand::Rng,
        status_msg: &mut String,
        monsters: &mut Vec<Monster>,
    ) {
        let trapped = if let Some(tile) = self.get_tile(cx, cy) {
            if let TileType::Chest { trapped, .. } = tile.tile_type {
                trapped
            } else {
                false
            }
        } else {
            false
        };

        if trapped {
            *status_msg = "The chest was trapped!".to_string();
            self.trigger_chest_trap(cx, cy, player, rng, status_msg, monsters);
        } else {
            *status_msg = "You open the chest.".to_string();
        }

        let mut loot_desc = Vec::new();
        let num_loots = rng.gen_range(1..=3);
        for _ in 0..num_loots {
            if rng.gen_bool(0.40) {
                let gold_amt = rng.gen_range(20..=80);
                player.gold += gold_amt;
                loot_desc.push(format!("{} gp", gold_amt));
            } else {
                let item = generate_random_floor_item(self.depth, rng);
                let item_name = item.name.clone();
                player.add_item_to_inventory(item);
                loot_desc.push(item_name);
            }
        }
        
        status_msg.push_str(&format!(" Inside you find: {}.", loot_desc.join(", ")));

        if let Some(tile) = self.get_tile_mut(cx, cy) {
            tile.tile_type = TileType::Floor;
        }
    }
}

pub fn get_shop_items(shop: ShopType) -> Vec<(&'static str, u32, ItemType)> {
    match shop {
        ShopType::General => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Scroll of Identify", 50, ItemType::Scroll { teleport: false }),
            ("Mage Spellbook [Beginner's Magick]", 50, ItemType::Scroll { teleport: false }),
            ("Priest Prayerbook [Beginner's Handbook]", 50, ItemType::Scroll { teleport: false }),
            ("Dagger", 50, ItemType::Weapon { damage: Dice::new(1, 4) }),
            ("Leather Armor", 80, ItemType::Armor { ac: 4 }),
            ("Iron Spike", 5, ItemType::Scroll { teleport: false }),
            ("Ration of Food", 10, ItemType::Food { nutrition: 5000 }),
            ("Shovel", 25, ItemType::Weapon { damage: Dice::new(1, 2) }),
            ("Wooden Torch", 15, ItemType::Light { fuel: 4000 }),
            ("Brass Lantern", 50, ItemType::Light { fuel: 7500 }),
            ("Flask of Oil", 10, ItemType::Potion { heal_amount: 0 }),
            ("Sling", 25, ItemType::Bow { multiplier: 2 }),
            ("Iron Shot", 5, ItemType::Missile { damage: Dice::new(1, 3) }),
            ("Arrow", 5, ItemType::Missile { damage: Dice::new(1, 4) }),
        ],
        ShopType::Armory => vec![
            ("Leather Armor", 80, ItemType::Armor { ac: 4 }),
            ("Chain Mail", 250, ItemType::Armor { ac: 7 }),
            ("Iron Shield", 150, ItemType::Armor { ac: 3 }),
        ],
        ShopType::Weaponsmith => vec![
            ("Dagger", 50, ItemType::Weapon { damage: Dice::new(1, 4) }),
            ("Short Sword", 120, ItemType::Weapon { damage: Dice::new(1, 6) }),
            ("Broadsword", 350, ItemType::Weapon { damage: Dice::new(2, 5) }),
            ("Pickaxe", 50, ItemType::Weapon { damage: Dice::new(1, 3) }),
            ("Sling", 25, ItemType::Bow { multiplier: 2 }),
            ("Short Bow", 50, ItemType::Bow { multiplier: 2 }),
            ("Long Bow", 120, ItemType::Bow { multiplier: 3 }),
            ("Light Crossbow", 140, ItemType::Bow { multiplier: 3 }),
            ("Heavy Crossbow", 300, ItemType::Bow { multiplier: 4 }),
            ("Arrow", 5, ItemType::Missile { damage: Dice::new(1, 4) }),
            ("Bolt", 8, ItemType::Missile { damage: Dice::new(1, 5) }),
            ("Iron Shot", 5, ItemType::Missile { damage: Dice::new(1, 3) }),
        ],
        ShopType::Temple => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Potion of Healing", 100, ItemType::Potion { heal_amount: 25 }),
            ("Potion of Cure Poison", 30, ItemType::Potion { heal_amount: 0 }),
            ("Potion of Heroism", 50, ItemType::Potion { heal_amount: 10 }),
            ("Scroll of Identify", 50, ItemType::Scroll { teleport: false }),
            ("Scroll of Word of Recall", 150, ItemType::Scroll { teleport: false }),
            ("Priest Prayerbook [Beginner's Handbook]", 50, ItemType::Scroll { teleport: false }),
        ],
        ShopType::Alchemy => vec![
            ("Potion of Cure Light Wounds", 30, ItemType::Potion { heal_amount: 10 }),
            ("Potion of Healing", 100, ItemType::Potion { heal_amount: 25 }),
            ("Potion of Cure Poison", 30, ItemType::Potion { heal_amount: 0 }),
            ("Potion of Speed", 50, ItemType::Potion { heal_amount: 0 }),
            ("Potion of Restore Strength", 100, ItemType::Potion { heal_amount: 0 }),
            ("Scroll of Identify", 50, ItemType::Scroll { teleport: false }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
        ],
        ShopType::Magic => vec![
            ("Scroll of Identify", 50, ItemType::Scroll { teleport: false }),
            ("Scroll of Phase Door", 20, ItemType::Scroll { teleport: true }),
            ("Scroll of Teleportation", 60, ItemType::Scroll { teleport: true }),
            ("Scroll of Word of Recall", 150, ItemType::Scroll { teleport: false }),
            ("Mage Spellbook [Beginner's Magick]", 50, ItemType::Scroll { teleport: false }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doors_only_placed_at_room_entrances() {
        let mut total_doors = 0;
        let mut total_secret_doors = 0;

        // Generate 100 random dungeon levels across different depths
        for depth in 1..=20 {
            for _ in 0..5 {
                let mut level = DungeonLevel::new(80, 24, depth, 50);
                level.generate_simple_floor();

                for y in 0..level.height {
                    for x in 0..level.width {
                        if let Some(tile) = level.get_tile(x, y) {
                            let is_door = matches!(
                                tile.tile_type,
                                TileType::DoorClosed { .. } | TileType::DoorOpen | TileType::SecretDoor
                            );

                            if is_door {
                                total_doors += 1;
                                if tile.tile_type == TileType::SecretDoor {
                                    total_secret_doors += 1;
                                }

                                // 1. Door must NEVER be inside a room interior
                                assert!(
                                    !level.is_in_room(x, y),
                                    "Door at ({}, {}) was placed inside a room interior!",
                                    x, y
                                );

                                // 2. Door must be recognized as a valid room entrance
                                assert!(
                                    level.is_room_entrance(x, y),
                                    "Door at ({}, {}) was NOT placed at a room entrance!",
                                    x, y
                                );

                                // 3. Door must have at least one orthogonal neighbor inside a room
                                let neighbors = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)];
                                let borders_room = neighbors.iter().any(|&(nx, ny)| level.is_in_room(nx, ny));
                                assert!(
                                    borders_room,
                                    "Door at ({}, {}) does not border any room!",
                                    x, y
                                );
                            }
                        }
                    }
                }
            }
        }

        // Verify that doors are actively generated and not 0
        assert!(total_doors > 50, "Expected doors to be generated, got {}", total_doors);
        assert!(total_secret_doors > 0, "Expected secret doors to be generated, got {}", total_secret_doors);
    }
}

