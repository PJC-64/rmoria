use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

pub mod shop;
pub mod tile;

use crate::entity::monster::Monster;
use crate::player::{Item, ItemType, Player};
pub use shop::{
    HaggleState, MAX_OWNERS, STORE_OWNERS, StoreOwner, calculate_buy_price, calculate_sell_price,
    get_item_base_value, handle_haggle_input, handle_shop_buy_input,
};
pub use tile::{Tile, TileType, TrapType};

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
    #[serde(default)]
    pub owner_index: usize,
    #[serde(default)]
    pub insults: u32,
    #[serde(default)]
    pub closed_until_turn: u64,
    #[serde(default)]
    pub inventory: Vec<Item>,
    #[serde(default)]
    pub good_purchases: u16,
    #[serde(default)]
    pub bad_purchases: u16,
}

impl ShopInfo {
    pub fn new(
        door_x: usize,
        door_y: usize,
        shop_type: ShopType,
        rng: &mut impl rand::Rng,
    ) -> Self {
        let store_offset = shop_type as usize;
        let owner_variant = rng.gen_range(0..3);
        let owner_index = store_offset + owner_variant * 6;
        let mut info = Self {
            door_x,
            door_y,
            shop_type,
            owner_index,
            insults: 0,
            closed_until_turn: 0,
            inventory: Vec::with_capacity(24),
            good_purchases: 0,
            bad_purchases: 0,
        };
        info.ensure_stocked(rng);
        info
    }

    pub fn owner(&self) -> &'static StoreOwner {
        &STORE_OWNERS[self.owner_index.min(MAX_OWNERS - 1)]
    }

    pub fn is_closed(&self, turn: u64) -> bool {
        turn < self.closed_until_turn
    }

    pub fn no_need_to_bargain(&self, min_price: u32) -> bool {
        if self.good_purchases == u16::MAX {
            return true;
        }
        let record = self.good_purchases as i32 - 3 * self.bad_purchases as i32 - 5;
        record > 0 && (record * record) > (min_price as i32 / 50)
    }

    pub fn update_bargaining_skills(&mut self, price: u32, min_price: u32) {
        if min_price < 10 {
            return;
        }
        if price == min_price {
            self.good_purchases = self.good_purchases.saturating_add(1);
        } else {
            self.bad_purchases = self.bad_purchases.saturating_add(1);
        }
    }

    pub fn ensure_stocked(&mut self, rng: &mut impl rand::Rng) {
        let max_cost = self.owner().max_cost;
        while self.inventory.len() < 20 {
            if let Some(item) = shop::create_store_item(self.shop_type, max_cost, rng) {
                shop::store_carry_item(&mut self.inventory, item);
            }
        }
    }

    pub fn maintain(&mut self, turn: u64, rng: &mut impl rand::Rng) {
        self.insults = 0;
        if self.closed_until_turn > 0 && turn >= self.closed_until_turn {
            self.closed_until_turn = 0;
        }

        // Canonical umoria storeMaintenance:
        if self.inventory.len() >= 20 {
            let mut turnaround = rng.gen_range(1..=4);
            if self.inventory.len() >= 22 {
                turnaround += self.inventory.len() - 22 + 1;
            }
            for _ in 0..turnaround {
                if !self.inventory.is_empty() {
                    let remove_idx = rng.gen_range(0..self.inventory.len());
                    if self.inventory[remove_idx].count > 1 {
                        self.inventory[remove_idx].count = rng
                            .gen_range(1..=self.inventory[remove_idx].count / 2)
                            .max(1);
                        if rng.gen_bool(0.5) {
                            self.inventory.remove(remove_idx);
                        }
                    } else {
                        self.inventory.remove(remove_idx);
                    }
                }
            }
        }

        if self.inventory.len() <= 22 {
            let max_cost = self.owner().max_cost;
            let turnaround = rng.gen_range(1..=4);
            for _ in 0..turnaround {
                if self.inventory.len() >= 24 {
                    break;
                }
                if let Some(item) = shop::create_store_item(self.shop_type, max_cost, rng) {
                    shop::store_carry_item(&mut self.inventory, item);
                }
            }
        }

        if self.insults >= self.owner().max_insults && rng.gen_bool(0.20) {
            let store_offset = self.shop_type as usize;
            let owner_variant = rng.gen_range(0..3);
            self.owner_index = store_offset + owner_variant * 6;
            self.insults = 0;
        }
    }
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

pub const MAX_HEIGHT: usize = 66;
pub const MAX_WIDTH: usize = 198;
pub const SCREEN_HEIGHT: usize = 22;
pub const SCREEN_WIDTH: usize = 66;
const QUART_HEIGHT: isize = (SCREEN_HEIGHT / 4) as isize;
const QUART_WIDTH: isize = (SCREEN_WIDTH / 4) as isize;

// Canonical Umoria config::dungeon constants
const DUN_RANDOM_DIR: i32 = 9;
const DUN_DIR_CHANGE: i32 = 70;
const DUN_TUNNELING: i32 = 15;
const DUN_ROOMS_MEAN: i32 = 32;
const DUN_ROOM_DOORS: i32 = 25;
const DUN_TUNNEL_DOORS: i32 = 15;
const DUN_STREAMER_DENSITY: i32 = 5;
const DUN_STREAMER_WIDTH: isize = 2;
const DUN_MAGMA_STREAMER: usize = 3;
const DUN_MAGMA_TREASURE: i32 = 90;
const DUN_QUARTZ_STREAMER: usize = 2;
const DUN_QUARTZ_TREASURE: i32 = 40;
const DUN_UNUSUAL_ROOMS: i32 = 300;
const LEVEL_OBJECTS_PER_ROOM: i32 = 7;
const LEVEL_OBJECTS_PER_CORRIDOR: i32 = 2;
const LEVEL_TOTAL_GOLD_AND_GEMS: i32 = 2;

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
    #[serde(default)]
    pub panel_row: usize,
    #[serde(default)]
    pub panel_col: usize,
    #[serde(skip)]
    pub vault_monsters: Vec<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GenFeature {
    NullWall,
    DarkFloor,
    LightFloor,
    CorrFloor,
    BlockedFloor,
    Tmp1Wall,
    Tmp2Wall,
    GraniteWall,
    MagmaWall,
    QuartzWall,
    BoundaryWall,
}

impl GenFeature {
    fn is_cave_floor(self) -> bool {
        matches!(
            self,
            Self::DarkFloor | Self::LightFloor | Self::CorrFloor | Self::BlockedFloor
        )
    }

    fn is_cave_wall(self) -> bool {
        matches!(
            self,
            Self::GraniteWall | Self::MagmaWall | Self::QuartzWall | Self::BoundaryWall
        )
    }

    fn is_room_floor(self) -> bool {
        matches!(self, Self::DarkFloor | Self::LightFloor)
    }

    fn is_corridor(self) -> bool {
        matches!(self, Self::CorrFloor | Self::BlockedFloor)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GenOverlay {
    None,
    OpenDoor,
    BrokenDoor,
    ClosedDoor,
    LockedDoor(u32),
    StuckDoor(u32),
    SecretDoor,
    Trap(TrapType),
    VeinGold,
    Object,
    Gold,
}

#[derive(Clone, Copy)]
struct GenCell {
    feature: GenFeature,
    perma_lit_room: bool,
    overlay: GenOverlay,
}

impl Default for GenCell {
    fn default() -> Self {
        Self {
            feature: GenFeature::NullWall,
            perma_lit_room: false,
            overlay: GenOverlay::None,
        }
    }
}

fn rnd<R: Rng>(max: i32, rng: &mut R) -> i32 {
    if max <= 1 { 1 } else { rng.gen_range(1..=max) }
}

fn rand_normal<R: Rng>(mean: i32, std: i32, rng: &mut R) -> i32 {
    let sum: i32 = (0..8).map(|_| rng.gen_range(1..=99)).sum();
    mean + (sum - 400) * std / 81
}

fn random_trap_type<R: Rng>(rng: &mut R) -> TrapType {
    TrapType::ALL[rng.gen_range(0..TrapType::ALL.len())]
}

struct CaveGen {
    width: usize,
    height: usize,
    depth: u32,
    cells: Vec<GenCell>,
    doors_tk: Vec<(usize, usize)>,
    rooms: Vec<RoomInfo>,
    vault_monsters: Vec<(usize, usize)>,
}

impl CaveGen {
    fn new(width: usize, height: usize, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
            cells: vec![GenCell::default(); width * height],
            doors_tk: Vec::new(),
            rooms: Vec::new(),
            vault_monsters: Vec::new(),
        }
    }

    fn in_bounds(&self, y: isize, x: isize) -> bool {
        y > 0 && y < (self.height as isize - 1) && x > 0 && x < (self.width as isize - 1)
    }

    fn valid_pos(&self, y: isize, x: isize) -> bool {
        y >= 0 && y < self.height as isize && x >= 0 && x < self.width as isize
    }

    fn cell(&self, y: usize, x: usize) -> &GenCell {
        &self.cells[y * self.width + x]
    }

    fn cell_mut(&mut self, y: usize, x: usize) -> &mut GenCell {
        &mut self.cells[y * self.width + x]
    }

    fn floor_tile_for_level<R: Rng>(&self, rng: &mut R) -> GenFeature {
        if (self.depth as i32) <= rnd(25, rng) {
            GenFeature::LightFloor
        } else {
            GenFeature::DarkFloor
        }
    }

    fn place_open_door(&mut self, y: usize, x: usize) {
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::OpenDoor;
        c.feature = GenFeature::CorrFloor;
    }

    fn place_broken_door(&mut self, y: usize, x: usize) {
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::BrokenDoor;
        c.feature = GenFeature::CorrFloor;
    }

    fn place_closed_door(&mut self, y: usize, x: usize) {
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::ClosedDoor;
        c.feature = GenFeature::BlockedFloor;
    }

    fn place_locked_door<R: Rng>(&mut self, y: usize, x: usize, rng: &mut R) {
        let lock = (rnd(10, rng) + 10) as u32;
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::LockedDoor(lock);
        c.feature = GenFeature::BlockedFloor;
    }

    fn place_stuck_door<R: Rng>(&mut self, y: usize, x: usize, rng: &mut R) {
        let jam = (rnd(10, rng) + 10) as u32;
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::StuckDoor(jam);
        c.feature = GenFeature::BlockedFloor;
    }

    fn place_secret_door(&mut self, y: usize, x: usize) {
        let c = self.cell_mut(y, x);
        c.overlay = GenOverlay::SecretDoor;
        c.feature = GenFeature::BlockedFloor;
    }

    fn place_door<R: Rng>(&mut self, y: usize, x: usize, rng: &mut R) {
        let door_type = rnd(3, rng);
        if door_type == 1 {
            if rnd(4, rng) == 1 {
                self.place_broken_door(y, x);
            } else {
                self.place_open_door(y, x);
            }
        } else if door_type == 2 {
            let sub = rnd(12, rng);
            if sub > 3 {
                self.place_closed_door(y, x);
            } else if sub == 3 {
                self.place_stuck_door(y, x, rng);
            } else {
                self.place_locked_door(y, x, rng);
            }
        } else {
            self.place_secret_door(y, x);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn place_random_secret_door<R: Rng>(
        &mut self,
        cy: isize,
        cx: isize,
        depth: isize,
        height: isize,
        left: isize,
        right: isize,
        rng: &mut R,
    ) {
        let (dy, dx) = match rnd(4, rng) {
            1 => (height - 1, cx),
            2 => (depth + 1, cx),
            3 => (cy, left - 1),
            _ => (cy, right + 1),
        };
        if self.in_bounds(dy, dx) {
            self.place_secret_door(dy as usize, dx as usize);
        }
    }

    fn place_vault(&mut self, cy: isize, cx: isize) {
        for y in (cy - 1)..=(cy + 1) {
            if self.in_bounds(y, cx - 1) {
                self.cell_mut(y as usize, (cx - 1) as usize).feature = GenFeature::Tmp1Wall;
            }
            if self.in_bounds(y, cx + 1) {
                self.cell_mut(y as usize, (cx + 1) as usize).feature = GenFeature::Tmp1Wall;
            }
        }
        if self.in_bounds(cy - 1, cx) {
            self.cell_mut((cy - 1) as usize, cx as usize).feature = GenFeature::Tmp1Wall;
        }
        if self.in_bounds(cy + 1, cx) {
            self.cell_mut((cy + 1) as usize, cx as usize).feature = GenFeature::Tmp1Wall;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn place_treasure_vault<R: Rng>(
        &mut self,
        cy: isize,
        cx: isize,
        depth: isize,
        height: isize,
        left: isize,
        right: isize,
        rng: &mut R,
    ) {
        self.place_random_secret_door(cy, cx, depth, height, left, right, rng);
        self.place_vault(cy, cx);

        let offset = rnd(4, rng) as isize;
        let (ly, lx) = if offset < 3 {
            (cy - 3 + (offset << 1), cx)
        } else {
            (cy, cx - 7 + (offset << 1))
        };
        if self.in_bounds(ly, lx) {
            self.place_locked_door(ly as usize, lx as usize, rng);
        }
    }

    fn place_inner_pillars<R: Rng>(&mut self, cy: isize, cx: isize, rng: &mut R) {
        for y in (cy - 1)..=(cy + 1) {
            for x in (cx - 1)..=(cx + 1) {
                if self.in_bounds(y, x) {
                    self.cell_mut(y as usize, x as usize).feature = GenFeature::Tmp1Wall;
                }
            }
        }

        if rnd(2, rng) != 1 {
            return;
        }

        let offset = rnd(2, rng) as isize;
        for y in (cy - 1)..=(cy + 1) {
            for x in (cx - 5 - offset)..=(cx - 3 - offset) {
                if self.in_bounds(y, x) {
                    self.cell_mut(y as usize, x as usize).feature = GenFeature::Tmp1Wall;
                }
            }
            for x in (cx + 3 + offset)..=(cx + 5 + offset) {
                if self.in_bounds(y, x) {
                    self.cell_mut(y as usize, x as usize).feature = GenFeature::Tmp1Wall;
                }
            }
        }
    }

    fn place_maze_inside_room(&mut self, depth: isize, height: isize, left: isize, right: isize) {
        for y in height..=depth {
            for x in left..=right {
                if self.in_bounds(y, x) && ((x + y) & 1) != 0 {
                    self.cell_mut(y as usize, x as usize).feature = GenFeature::Tmp1Wall;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn place_four_small_rooms<R: Rng>(
        &mut self,
        cy: isize,
        cx: isize,
        depth: isize,
        height: isize,
        left: isize,
        right: isize,
        rng: &mut R,
    ) {
        for y in height..=depth {
            if self.in_bounds(y, cx) {
                self.cell_mut(y as usize, cx as usize).feature = GenFeature::Tmp1Wall;
            }
        }
        for x in left..=right {
            if self.in_bounds(cy, x) {
                self.cell_mut(cy as usize, x as usize).feature = GenFeature::Tmp1Wall;
            }
        }

        if rnd(2, rng) == 1 {
            let offset = rnd(10, rng) as isize;
            for (dy, dx) in [
                (height - 1, cx - offset),
                (height - 1, cx + offset),
                (depth + 1, cx - offset),
                (depth + 1, cx + offset),
            ] {
                if self.in_bounds(dy, dx) {
                    self.place_secret_door(dy as usize, dx as usize);
                }
            }
        } else {
            let offset = rnd(3, rng) as isize;
            for (dy, dx) in [
                (cy + offset, left - 1),
                (cy - offset, left - 1),
                (cy + offset, right + 1),
                (cy - offset, right + 1),
            ] {
                if self.in_bounds(dy, dx) {
                    self.place_secret_door(dy as usize, dx as usize);
                }
            }
        }
    }

    fn place_vault_trap<R: Rng>(
        &mut self,
        cy: isize,
        cx: isize,
        disp_y: isize,
        disp_x: isize,
        number: i32,
        rng: &mut R,
    ) {
        for _ in 0..number {
            for _ in 0..=5 {
                let sy = cy - disp_y - 1 + rnd(2 * disp_y as i32 + 1, rng) as isize;
                let sx = cx - disp_x - 1 + rnd(2 * disp_x as i32 + 1, rng) as isize;
                if self.in_bounds(sy, sx) {
                    let c = self.cell_mut(sy as usize, sx as usize);
                    if c.feature != GenFeature::NullWall
                        && c.feature.is_cave_floor()
                        && c.overlay == GenOverlay::None
                    {
                        c.overlay = GenOverlay::Trap(random_trap_type(rng));
                        break;
                    }
                }
            }
        }
    }

    fn place_vault_monster(&mut self, cy: isize, cx: isize, number: i32) {
        if self.in_bounds(cy, cx) {
            for _ in 0..number {
                self.vault_monsters.push((cx as usize, cy as usize));
            }
        }
    }

    fn place_random_object_at(&mut self, cy: isize, cx: isize) {
        if self.in_bounds(cy, cx) {
            let c = self.cell_mut(cy as usize, cx as usize);
            if c.overlay == GenOverlay::None {
                c.overlay = GenOverlay::Object;
            }
        }
    }

    fn place_random_object_near<R: Rng>(&mut self, cy: isize, cx: isize, tries: i32, rng: &mut R) {
        for _ in 0..tries {
            for _ in 0..=10 {
                let ay = cy - 3 + rnd(5, rng) as isize;
                let ax = cx - 4 + rnd(7, rng) as isize;
                if self.in_bounds(ay, ax) {
                    let c = self.cell_mut(ay as usize, ax as usize);
                    if c.feature.is_cave_floor() && c.overlay == GenOverlay::None {
                        if rnd(100, rng) < 75 {
                            c.overlay = GenOverlay::Object;
                        } else {
                            c.overlay = GenOverlay::Gold;
                        }
                        break;
                    }
                }
            }
        }
    }

    fn build_room<R: Rng>(&mut self, cy: isize, cx: isize, rng: &mut R) {
        let floor = self.floor_tile_for_level(rng);
        let is_lit = floor == GenFeature::LightFloor;

        let height = (cy - rnd(4, rng) as isize).max(1);
        let depth = (cy + rnd(3, rng) as isize).min(self.height as isize - 2);
        let left = (cx - rnd(11, rng) as isize).max(1);
        let right = (cx + rnd(11, rng) as isize).min(self.width as isize - 2);

        for y in height..=depth {
            for x in left..=right {
                let c = self.cell_mut(y as usize, x as usize);
                c.feature = floor;
                c.perma_lit_room = true;
            }
        }

        for y in (height - 1)..=(depth + 1) {
            if self.valid_pos(y, left - 1) {
                let c = self.cell_mut(y as usize, (left - 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(y, right + 1) {
                let c = self.cell_mut(y as usize, (right + 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        for x in left..=right {
            if self.valid_pos(height - 1, x) {
                let c = self.cell_mut((height - 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(depth + 1, x) {
                let c = self.cell_mut((depth + 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        self.rooms.push(RoomInfo {
            x: left as usize,
            y: height as usize,
            w: (right - left + 1) as usize,
            h: (depth - height + 1) as usize,
            is_lit,
        });
    }

    fn build_room_overlapping_rectangles<R: Rng>(&mut self, cy: isize, cx: isize, rng: &mut R) {
        let floor = self.floor_tile_for_level(rng);
        let is_lit = floor == GenFeature::LightFloor;
        let limit = 1 + rnd(2, rng);

        for _ in 0..limit {
            let height = (cy - rnd(4, rng) as isize).max(1);
            let depth = (cy + rnd(3, rng) as isize).min(self.height as isize - 2);
            let left = (cx - rnd(11, rng) as isize).max(1);
            let right = (cx + rnd(11, rng) as isize).min(self.width as isize - 2);

            for y in height..=depth {
                for x in left..=right {
                    let c = self.cell_mut(y as usize, x as usize);
                    c.feature = floor;
                    c.perma_lit_room = true;
                }
            }

            for y in (height - 1)..=(depth + 1) {
                if self.valid_pos(y, left - 1) {
                    let c = self.cell_mut(y as usize, (left - 1) as usize);
                    if c.feature != floor {
                        c.feature = GenFeature::GraniteWall;
                        c.perma_lit_room = true;
                    }
                }
                if self.valid_pos(y, right + 1) {
                    let c = self.cell_mut(y as usize, (right + 1) as usize);
                    if c.feature != floor {
                        c.feature = GenFeature::GraniteWall;
                        c.perma_lit_room = true;
                    }
                }
            }

            for x in left..=right {
                if self.valid_pos(height - 1, x) {
                    let c = self.cell_mut((height - 1) as usize, x as usize);
                    if c.feature != floor {
                        c.feature = GenFeature::GraniteWall;
                        c.perma_lit_room = true;
                    }
                }
                if self.valid_pos(depth + 1, x) {
                    let c = self.cell_mut((depth + 1) as usize, x as usize);
                    if c.feature != floor {
                        c.feature = GenFeature::GraniteWall;
                        c.perma_lit_room = true;
                    }
                }
            }

            self.rooms.push(RoomInfo {
                x: left as usize,
                y: height as usize,
                w: (right - left + 1) as usize,
                h: (depth - height + 1) as usize,
                is_lit,
            });
        }
    }

    fn build_room_with_inner_rooms<R: Rng>(&mut self, cy: isize, cx: isize, rng: &mut R) {
        let floor = self.floor_tile_for_level(rng);
        let is_lit = floor == GenFeature::LightFloor;

        let mut height = (cy - 4).max(1);
        let mut depth = (cy + 4).min(self.height as isize - 2);
        let mut left = (cx - 11).max(1);
        let mut right = (cx + 11).min(self.width as isize - 2);

        for y in height..=depth {
            for x in left..=right {
                let c = self.cell_mut(y as usize, x as usize);
                c.feature = floor;
                c.perma_lit_room = true;
            }
        }

        for y in (height - 1)..=(depth + 1) {
            if self.valid_pos(y, left - 1) {
                let c = self.cell_mut(y as usize, (left - 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(y, right + 1) {
                let c = self.cell_mut(y as usize, (right + 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        for x in left..=right {
            if self.valid_pos(height - 1, x) {
                let c = self.cell_mut((height - 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(depth + 1, x) {
                let c = self.cell_mut((depth + 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        self.rooms.push(RoomInfo {
            x: left as usize,
            y: height as usize,
            w: (right - left + 1) as usize,
            h: (depth - height + 1) as usize,
            is_lit,
        });

        // The inner room
        height += 2;
        depth -= 2;
        left += 2;
        right -= 2;

        for y in (height - 1)..=(depth + 1) {
            if self.in_bounds(y, left - 1) {
                self.cell_mut(y as usize, (left - 1) as usize).feature = GenFeature::Tmp1Wall;
            }
            if self.in_bounds(y, right + 1) {
                self.cell_mut(y as usize, (right + 1) as usize).feature = GenFeature::Tmp1Wall;
            }
        }
        for x in left..=right {
            if self.in_bounds(height - 1, x) {
                self.cell_mut((height - 1) as usize, x as usize).feature = GenFeature::Tmp1Wall;
            }
            if self.in_bounds(depth + 1, x) {
                self.cell_mut((depth + 1) as usize, x as usize).feature = GenFeature::Tmp1Wall;
            }
        }

        match rnd(5, rng) {
            1 => {
                // Plain inner room
                self.place_random_secret_door(cy, cx, depth, height, left, right, rng);
                self.place_vault_monster(cy, cx, 1);
            }
            2 => {
                // TreasureVault
                self.place_treasure_vault(cy, cx, depth, height, left, right, rng);
                let num_mon = 2 + rnd(3, rng);
                self.place_vault_monster(cy, cx, num_mon);
                let num_traps = 2 + rnd(3, rng);
                self.place_vault_trap(cy, cx, 4, 10, num_traps, rng);
            }
            3 => {
                // Pillars
                self.place_random_secret_door(cy, cx, depth, height, left, right, rng);
                self.place_inner_pillars(cy, cx, rng);

                if rnd(3, rng) == 1 {
                    for x in (cx - 5)..=(cx + 5) {
                        if self.in_bounds(cy - 1, x) {
                            self.cell_mut((cy - 1) as usize, x as usize).feature =
                                GenFeature::Tmp1Wall;
                        }
                        if self.in_bounds(cy + 1, x) {
                            self.cell_mut((cy + 1) as usize, x as usize).feature =
                                GenFeature::Tmp1Wall;
                        }
                    }
                    if self.in_bounds(cy, cx - 5) {
                        self.cell_mut(cy as usize, (cx - 5) as usize).feature =
                            GenFeature::Tmp1Wall;
                    }
                    if self.in_bounds(cy, cx + 5) {
                        self.cell_mut(cy as usize, (cx + 5) as usize).feature =
                            GenFeature::Tmp1Wall;
                    }

                    let sd1_y = cy - 3 + ((rnd(2, rng) as isize) << 1);
                    let sd2_y = cy - 3 + ((rnd(2, rng) as isize) << 1);
                    if self.in_bounds(sd1_y, cx - 3) {
                        self.place_secret_door(sd1_y as usize, (cx - 3) as usize);
                    }
                    if self.in_bounds(sd2_y, cx + 3) {
                        self.place_secret_door(sd2_y as usize, (cx + 3) as usize);
                    }

                    if rnd(3, rng) == 1 {
                        self.place_random_object_at(cy, cx - 2);
                    }
                    if rnd(3, rng) == 1 {
                        self.place_random_object_at(cy, cx + 2);
                    }

                    let m1 = rnd(2, rng);
                    let m2 = rnd(2, rng);
                    self.place_vault_monster(cy, cx - 2, m1);
                    self.place_vault_monster(cy, cx + 2, m2);
                }
            }
            4 => {
                // Maze
                self.place_random_secret_door(cy, cx, depth, height, left, right, rng);
                self.place_maze_inside_room(depth, height, left, right);
                let m1 = rnd(3, rng);
                let m2 = rnd(3, rng);
                self.place_vault_monster(cy, cx - 5, m1);
                self.place_vault_monster(cy, cx + 5, m2);
                let t1 = rnd(3, rng);
                let t2 = rnd(3, rng);
                self.place_vault_trap(cy, cx - 3, 2, 8, t1, rng);
                self.place_vault_trap(cy, cx + 3, 2, 8, t2, rng);
                for _ in 0..3 {
                    self.place_random_object_near(cy, cx, 1, rng);
                }
            }
            _ => {
                // FourSmallRooms
                self.place_four_small_rooms(cy, cx, depth, height, left, right, rng);
                let tries = 2 + rnd(2, rng);
                self.place_random_object_near(cy, cx, tries, rng);
                let m1 = rnd(2, rng);
                let m2 = rnd(2, rng);
                let m3 = rnd(2, rng);
                let m4 = rnd(2, rng);
                self.place_vault_monster(cy + 2, cx - 4, m1);
                self.place_vault_monster(cy + 2, cx + 4, m2);
                self.place_vault_monster(cy - 2, cx - 4, m3);
                self.place_vault_monster(cy - 2, cx + 4, m4);
            }
        }
    }

    fn build_room_cross_shaped<R: Rng>(&mut self, cy: isize, cx: isize, rng: &mut R) {
        let floor = self.floor_tile_for_level(rng);
        let is_lit = floor == GenFeature::LightFloor;

        let mut random_offset = 2 + rnd(2, rng) as isize;
        let mut height = (cy - random_offset).max(1);
        let mut depth = (cy + random_offset).min(self.height as isize - 2);
        let mut left = (cx - 1).max(1);
        let mut right = (cx + 1).min(self.width as isize - 2);

        for y in height..=depth {
            for x in left..=right {
                let c = self.cell_mut(y as usize, x as usize);
                c.feature = floor;
                c.perma_lit_room = true;
            }
        }

        for y in (height - 1)..=(depth + 1) {
            if self.valid_pos(y, left - 1) {
                let c = self.cell_mut(y as usize, (left - 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(y, right + 1) {
                let c = self.cell_mut(y as usize, (right + 1) as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        for x in left..=right {
            if self.valid_pos(height - 1, x) {
                let c = self.cell_mut((height - 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
            if self.valid_pos(depth + 1, x) {
                let c = self.cell_mut((depth + 1) as usize, x as usize);
                c.feature = GenFeature::GraniteWall;
                c.perma_lit_room = true;
            }
        }

        self.rooms.push(RoomInfo {
            x: left as usize,
            y: height as usize,
            w: (right - left + 1) as usize,
            h: (depth - height + 1) as usize,
            is_lit,
        });

        random_offset = 2 + rnd(9, rng) as isize;
        height = (cy - 1).max(1);
        depth = (cy + 1).min(self.height as isize - 2);
        left = (cx - random_offset).max(1);
        right = (cx + random_offset).min(self.width as isize - 2);

        for y in height..=depth {
            for x in left..=right {
                let c = self.cell_mut(y as usize, x as usize);
                c.feature = floor;
                c.perma_lit_room = true;
            }
        }

        for y in (height - 1)..=(depth + 1) {
            if self.valid_pos(y, left - 1) {
                let c = self.cell_mut(y as usize, (left - 1) as usize);
                if c.feature != floor {
                    c.feature = GenFeature::GraniteWall;
                    c.perma_lit_room = true;
                }
            }
            if self.valid_pos(y, right + 1) {
                let c = self.cell_mut(y as usize, (right + 1) as usize);
                if c.feature != floor {
                    c.feature = GenFeature::GraniteWall;
                    c.perma_lit_room = true;
                }
            }
        }

        for x in left..=right {
            if self.valid_pos(height - 1, x) {
                let c = self.cell_mut((height - 1) as usize, x as usize);
                if c.feature != floor {
                    c.feature = GenFeature::GraniteWall;
                    c.perma_lit_room = true;
                }
            }
            if self.valid_pos(depth + 1, x) {
                let c = self.cell_mut((depth + 1) as usize, x as usize);
                if c.feature != floor {
                    c.feature = GenFeature::GraniteWall;
                    c.perma_lit_room = true;
                }
            }
        }

        self.rooms.push(RoomInfo {
            x: left as usize,
            y: height as usize,
            w: (right - left + 1) as usize,
            h: (depth - height + 1) as usize,
            is_lit,
        });

        match rnd(4, rng) {
            1 => {
                // Large middle pillar
                for y in (cy - 1)..=(cy + 1) {
                    for x in (cx - 1)..=(cx + 1) {
                        if self.in_bounds(y, x) {
                            self.cell_mut(y as usize, x as usize).feature = GenFeature::Tmp1Wall;
                        }
                    }
                }
            }
            2 => {
                // Inner treasure vault
                self.place_vault(cy, cx);
                let off = rnd(4, rng) as isize;
                let (sdy, sdx) = if off < 3 {
                    (cy - 3 + (off << 1), cx)
                } else {
                    (cy, cx - 7 + (off << 1))
                };
                if self.in_bounds(sdy, sdx) {
                    self.place_secret_door(sdy as usize, sdx as usize);
                }
                self.place_random_object_at(cy, cx);
                let num_mon = 2 + rnd(2, rng);
                self.place_vault_monster(cy, cx, num_mon);
                let num_traps = 1 + rnd(3, rng);
                self.place_vault_trap(cy, cx, 4, 4, num_traps, rng);
            }
            3 => {
                if rnd(3, rng) == 1 {
                    for (dy, dx) in [
                        (-1, -2),
                        (1, -2),
                        (-1, 2),
                        (1, 2),
                        (-2, -1),
                        (-2, 1),
                        (2, -1),
                        (2, 1),
                    ] {
                        if self.in_bounds(cy + dy, cx + dx) {
                            self.cell_mut((cy + dy) as usize, (cx + dx) as usize)
                                .feature = GenFeature::Tmp1Wall;
                        }
                    }
                    if rnd(3, rng) == 1 {
                        for (dy, dx) in [(0, -2), (0, 2), (-2, 0), (2, 0)] {
                            if self.in_bounds(cy + dy, cx + dx) {
                                self.place_secret_door((cy + dy) as usize, (cx + dx) as usize);
                            }
                        }
                    }
                } else if rnd(3, rng) == 1 {
                    for (dy, dx) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if self.in_bounds(cy + dy, cx + dx) {
                            self.cell_mut((cy + dy) as usize, (cx + dx) as usize)
                                .feature = GenFeature::Tmp1Wall;
                        }
                    }
                } else if rnd(3, rng) == 1 && self.in_bounds(cy, cx) {
                    self.cell_mut(cy as usize, cx as usize).feature = GenFeature::Tmp1Wall;
                }
            }
            _ => {}
        }
    }

    fn pick_correct_direction<R: Rng>(
        sy: isize,
        sx: isize,
        ey: isize,
        ex: isize,
        rng: &mut R,
    ) -> (isize, isize) {
        let mut dy = if sy < ey {
            1
        } else if sy == ey {
            0
        } else {
            -1
        };
        let mut dx = if sx < ex {
            1
        } else if sx == ex {
            0
        } else {
            -1
        };
        if dy != 0 && dx != 0 {
            if rnd(2, rng) == 1 {
                dy = 0;
            } else {
                dx = 0;
            }
        }
        (dy, dx)
    }

    fn chance_of_random_direction<R: Rng>(rng: &mut R) -> (isize, isize) {
        match rnd(4, rng) {
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, -1),
            _ => (0, 1),
        }
    }

    fn build_tunnel<R: Rng>(
        &mut self,
        mut sy: isize,
        mut sx: isize,
        ey: isize,
        ex: isize,
        rng: &mut R,
    ) {
        let mut tunnels_tk: Vec<(usize, usize)> = Vec::new();
        let mut walls_tk: Vec<(usize, usize)> = Vec::new();
        let mut door_flag = false;
        let mut stop_flag = false;
        let mut main_loop_count = 0;
        let start_row = sy;
        let start_col = sx;

        let (mut y_dir, mut x_dir) = Self::pick_correct_direction(sy, sx, ey, ex, rng);

        while (sy != ey || sx != ex) && !stop_flag {
            main_loop_count += 1;
            if main_loop_count > 2000 {
                break;
            }

            if rnd(100, rng) > DUN_DIR_CHANGE {
                if rnd(DUN_RANDOM_DIR, rng) == 1 {
                    let (ny, nx) = Self::chance_of_random_direction(rng);
                    y_dir = ny;
                    x_dir = nx;
                } else {
                    let (ny, nx) = Self::pick_correct_direction(sy, sx, ey, ex, rng);
                    y_dir = ny;
                    x_dir = nx;
                }
            }

            let mut tmp_row = sy + y_dir;
            let mut tmp_col = sx + x_dir;

            while !self.in_bounds(tmp_row, tmp_col) {
                if rnd(DUN_RANDOM_DIR, rng) == 1 {
                    let (ny, nx) = Self::chance_of_random_direction(rng);
                    y_dir = ny;
                    x_dir = nx;
                } else {
                    let (ny, nx) = Self::pick_correct_direction(sy, sx, ey, ex, rng);
                    y_dir = ny;
                    x_dir = nx;
                }
                tmp_row = sy + y_dir;
                tmp_col = sx + x_dir;
            }

            let feat = self.cell(tmp_row as usize, tmp_col as usize).feature;
            match feat {
                GenFeature::NullWall => {
                    sy = tmp_row;
                    sx = tmp_col;
                    if tunnels_tk.len() < 1000 {
                        tunnels_tk.push((sy as usize, sx as usize));
                    }
                    door_flag = false;
                }
                GenFeature::Tmp2Wall => {
                    // Do nothing
                }
                GenFeature::GraniteWall => {
                    sy = tmp_row;
                    sx = tmp_col;
                    if walls_tk.len() < 1000 {
                        walls_tk.push((sy as usize, sx as usize));
                    }
                    for y in (sy - 1)..=(sy + 1) {
                        for x in (sx - 1)..=(sx + 1) {
                            if self.in_bounds(y, x)
                                && self.cell(y as usize, x as usize).feature
                                    == GenFeature::GraniteWall
                            {
                                self.cell_mut(y as usize, x as usize).feature =
                                    GenFeature::Tmp2Wall;
                            }
                        }
                    }
                }
                GenFeature::CorrFloor | GenFeature::BlockedFloor => {
                    sy = tmp_row;
                    sx = tmp_col;
                    if !door_flag {
                        if self.doors_tk.len() < 100 {
                            self.doors_tk.push((sy as usize, sx as usize));
                        }
                        door_flag = true;
                    }
                    if rnd(100, rng) > DUN_TUNNELING {
                        let dr = (sy - start_row).abs();
                        let dc = (sx - start_col).abs();
                        if dr > 10 || dc > 10 {
                            stop_flag = true;
                        }
                    }
                }
                _ => {
                    sy = tmp_row;
                    sx = tmp_col;
                }
            }
        }

        for (ty, tx) in tunnels_tk {
            self.cell_mut(ty, tx).feature = GenFeature::CorrFloor;
        }

        for (wy, wx) in walls_tk {
            if self.cell(wy, wx).feature == GenFeature::Tmp2Wall {
                if rnd(100, rng) < DUN_ROOM_DOORS {
                    self.place_door(wy, wx, rng);
                } else {
                    self.cell_mut(wy, wx).feature = GenFeature::CorrFloor;
                }
            }
        }
    }

    fn fill_empty_tiles_with(&mut self, rock: GenFeature) {
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                let c = self.cell_mut(y, x);
                if matches!(
                    c.feature,
                    GenFeature::NullWall | GenFeature::Tmp1Wall | GenFeature::Tmp2Wall
                ) {
                    c.feature = rock;
                }
            }
        }
    }

    fn place_boundary_walls(&mut self) {
        for y in 0..self.height {
            self.cell_mut(y, 0).feature = GenFeature::BoundaryWall;
            self.cell_mut(y, self.width - 1).feature = GenFeature::BoundaryWall;
        }
        for x in 0..self.width {
            self.cell_mut(0, x).feature = GenFeature::BoundaryWall;
            self.cell_mut(self.height - 1, x).feature = GenFeature::BoundaryWall;
        }
    }

    fn place_streamer_rock<R: Rng>(
        &mut self,
        rock_type: GenFeature,
        chance_of_treasure: i32,
        rng: &mut R,
    ) {
        let mut cy = (self.height as isize / 2) + 11 - rnd(23, rng) as isize;
        let mut cx = (self.width as isize / 2) + 16 - rnd(33, rng) as isize;

        let mut dir = rnd(8, rng);
        if dir > 4 {
            dir += 1;
        }
        let (dy, dx) = match dir {
            1 => (1, -1),
            2 => (1, 0),
            3 => (1, 1),
            4 => (0, -1),
            6 => (0, 1),
            7 => (-1, -1),
            8 => (-1, 0),
            _ => (-1, 1),
        };

        let t1 = (2 * DUN_STREAMER_WIDTH + 1) as i32;
        let t2 = DUN_STREAMER_WIDTH + 1;

        loop {
            for _ in 0..DUN_STREAMER_DENSITY {
                let sy = cy + rnd(t1, rng) as isize - t2;
                let sx = cx + rnd(t1, rng) as isize - t2;
                if self.in_bounds(sy, sx) {
                    let c = self.cell_mut(sy as usize, sx as usize);
                    if c.feature == GenFeature::GraniteWall {
                        c.feature = rock_type;
                        if rnd(chance_of_treasure, rng) == 1 {
                            c.overlay = GenOverlay::VeinGold;
                        }
                    }
                }
            }
            let ny = cy + dy;
            let nx = cx + dx;
            if !self.valid_pos(ny, nx) {
                break;
            }
            cy = ny;
            cx = nx;
        }
    }

    fn corridor_walls_next_to(&self, cy: usize, cx: usize) -> usize {
        let mut count = 0;
        for y in (cy.saturating_sub(1))..=(cy + 1).min(self.height - 1) {
            for x in (cx.saturating_sub(1))..=(cx + 1).min(self.width - 1) {
                let c = self.cell(y, x);
                let is_door = matches!(
                    c.overlay,
                    GenOverlay::OpenDoor
                        | GenOverlay::BrokenDoor
                        | GenOverlay::ClosedDoor
                        | GenOverlay::LockedDoor(_)
                        | GenOverlay::StuckDoor(_)
                        | GenOverlay::SecretDoor
                );
                if c.feature == GenFeature::CorrFloor && !is_door {
                    count += 1;
                }
            }
        }
        count
    }

    fn is_next_to(&self, cy: usize, cx: usize) -> bool {
        if !self.in_bounds(cy as isize, cx as isize) {
            return false;
        }
        if self.corridor_walls_next_to(cy, cx) > 2 {
            let vertical = self.cell(cy - 1, cx).feature.is_cave_wall()
                && self.cell(cy + 1, cx).feature.is_cave_wall();
            let horizontal = self.cell(cy, cx - 1).feature.is_cave_wall()
                && self.cell(cy, cx + 1).feature.is_cave_wall();
            return vertical || horizontal;
        }
        false
    }

    fn place_door_if_next_to_two_walls<R: Rng>(&mut self, y: isize, x: isize, rng: &mut R) {
        if !self.in_bounds(y, x) {
            return;
        }
        let uy = y as usize;
        let ux = x as usize;
        if self.cell(uy, ux).feature == GenFeature::CorrFloor
            && rnd(100, rng) > DUN_TUNNEL_DOORS
            && self.is_next_to(uy, ux)
        {
            self.place_door(uy, ux, rng);
        }
    }
}

pub fn generate_random_floor_item<R: Rng>(depth: u32, rng: &mut R) -> Item {
    crate::entity::treasure_data::generate_canonical_floor_item(depth, rng)
}

fn generate_canonical_gold<R: Rng>(depth: u32, rng: &mut R) -> Item {
    let gold_names = [
        ("copper", 3),
        ("copper", 4),
        ("copper", 5),
        ("silver", 6),
        ("silver", 7),
        ("silver", 8),
        ("garnets", 9),
        ("garnets", 10),
        ("gold", 12),
        ("gold", 14),
        ("gold", 16),
        ("opals", 18),
        ("sapphires", 20),
        ("rubies", 24),
        ("diamonds", 28),
        ("emeralds", 32),
        ("mithril", 40),
        ("adamantite", 80),
    ];
    let mut gold_type_id = (((rnd(depth as i32 + 2, rng) + 2) / 2) - 1).max(0) as usize;
    if rnd(12, rng) == 1 {
        gold_type_id += rnd(depth as i32 + 1, rng) as usize;
    }
    if gold_type_id >= gold_names.len() {
        gold_type_id = gold_names.len() - 1;
    }
    let (_, base_cost) = gold_names[gold_type_id];
    let amount = (base_cost + 8 * rnd(base_cost, rng) + rnd(8, rng)) as u32;
    Item::new(
        &format!("Gold Pile [{} gp]", amount),
        1,
        1,
        ItemType::Food { nutrition: 0 },
    )
}

impl DungeonLevel {
    pub fn new(width: usize, height: usize, depth: u32, max_depth: u32) -> Self {
        let tiles = vec![Tile::new(TileType::Wall); width * height];
        Self {
            width,
            height,
            tiles,
            depth,
            shops: Vec::new(),
            max_depth,
            rooms: Vec::new(),
            items: Vec::new(),
            panel_row: 0,
            panel_col: 0,
            vault_monsters: Vec::new(),
        }
    }

    pub fn get_viewport_bounds(
        &mut self,
        player_x: usize,
        player_y: usize,
    ) -> (usize, usize, usize, usize) {
        if self.width <= SCREEN_WIDTH && self.height <= SCREEN_HEIGHT {
            self.panel_row = 0;
            self.panel_col = 0;
            return (0, self.height, 0, self.width);
        }

        let max_rows = ((self.height / SCREEN_HEIGHT) * 2).saturating_sub(2);
        let max_cols = ((self.width / SCREEN_WIDTH) * 2).saturating_sub(2);

        self.panel_row = self.panel_row.min(max_rows);
        self.panel_col = self.panel_col.min(max_cols);

        let top = self.panel_row * (SCREEN_HEIGHT / 2);
        let bottom = (top + SCREEN_HEIGHT).min(self.height).saturating_sub(1);
        let left = self.panel_col * (SCREEN_WIDTH / 2);
        let right = (left + SCREEN_WIDTH).min(self.width).saturating_sub(1);

        if player_y < top + 2 || player_y + 2 > bottom {
            let new_row = (player_y as isize - QUART_HEIGHT) / (SCREEN_HEIGHT as isize / 2);
            self.panel_row = new_row.clamp(0, max_rows as isize) as usize;
        }

        if player_x < left + 3 || player_x + 3 > right {
            let new_col = (player_x as isize - QUART_WIDTH) / (SCREEN_WIDTH as isize / 2);
            self.panel_col = new_col.clamp(0, max_cols as isize) as usize;
        }

        let final_top = self.panel_row * (SCREEN_HEIGHT / 2);
        let final_bottom = (final_top + SCREEN_HEIGHT).min(self.height);
        let final_left = self.panel_col * (SCREEN_WIDTH / 2);
        let final_right = (final_left + SCREEN_WIDTH).min(self.width);

        (final_top, final_bottom, final_left, final_right)
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

    pub fn set_tile(&mut self, x: usize, y: usize, tile_type: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x].tile_type = tile_type;
        }
    }

    pub fn is_in_room(&self, x: usize, y: usize) -> bool {
        self.rooms
            .iter()
            .any(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    }

    pub fn room_index_at(&self, x: usize, y: usize) -> Option<usize> {
        self.rooms
            .iter()
            .position(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
    }

    pub fn is_room_entrance(&self, x: usize, y: usize) -> bool {
        if self.is_in_room(x, y) {
            return false;
        }

        let is_wall = |tt: TileType| {
            matches!(
                tt,
                TileType::Wall | TileType::MagmaVein { .. } | TileType::QuartzVein { .. }
            )
        };
        let is_passage = |tt: TileType| !is_wall(tt);

        if let Some(tile) = self.get_tile(x, y) {
            if !is_passage(tile.tile_type) {
                return false;
            }
        } else {
            return false;
        }

        let w_l = self
            .get_tile(x.wrapping_sub(1), y)
            .map(|t| is_wall(t.tile_type))
            .unwrap_or(false);
        let w_r = self
            .get_tile(x + 1, y)
            .map(|t| is_wall(t.tile_type))
            .unwrap_or(false);
        let f_t = self
            .get_tile(x, y.wrapping_sub(1))
            .map(|t| is_passage(t.tile_type))
            .unwrap_or(false);
        let f_b = self
            .get_tile(x, y + 1)
            .map(|t| is_passage(t.tile_type))
            .unwrap_or(false);

        let w_t = self
            .get_tile(x, y.wrapping_sub(1))
            .map(|t| is_wall(t.tile_type))
            .unwrap_or(false);
        let w_b = self
            .get_tile(x, y + 1)
            .map(|t| is_wall(t.tile_type))
            .unwrap_or(false);
        let f_l = self
            .get_tile(x.wrapping_sub(1), y)
            .map(|t| is_passage(t.tile_type))
            .unwrap_or(false);
        let f_r = self
            .get_tile(x + 1, y)
            .map(|t| is_passage(t.tile_type))
            .unwrap_or(false);

        if w_l && w_r && f_t && f_b {
            let r_t = self.room_index_at(x, y.wrapping_sub(1));
            let r_b = self.room_index_at(x, y + 1);
            (r_t.is_some() || r_b.is_some()) && r_t != r_b
        } else if w_t && w_b && f_l && f_r {
            let r_l = self.room_index_at(x.wrapping_sub(1), y);
            let r_r = self.room_index_at(x + 1, y);
            (r_l.is_some() || r_r.is_some()) && r_l != r_r
        } else {
            false
        }
    }

    pub fn is_safe_for_obstacle(&self, x: usize, y: usize, entrances: &[(usize, usize)]) -> bool {
        if !self.is_in_room(x, y) || self.is_room_entrance(x, y) {
            return false;
        }
        !entrances.iter().any(|&(ex, ey)| {
            (ex as isize - x as isize).abs() + (ey as isize - y as isize).abs() <= 1
        })
    }

    pub fn stairs_are_reachable(&self) -> bool {
        let up_positions: Vec<(usize, usize)> = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.tile_type == TileType::StairsUp)
            .map(|(i, _)| (i % self.width, i / self.width))
            .collect();

        let down_positions: Vec<(usize, usize)> = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.tile_type == TileType::StairsDown)
            .map(|(i, _)| (i % self.width, i / self.width))
            .collect();

        if up_positions.is_empty() || down_positions.is_empty() {
            return true;
        }

        // Every up staircase must be able to reach at least one down staircase
        for &up in &up_positions {
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            visited.insert(up);
            queue.push_back(up);

            let mut reached_any_down = false;
            while let Some((cx, cy)) = queue.pop_front() {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < self.width
                        && (ny as usize) < self.height
                    {
                        let np = (nx as usize, ny as usize);
                        if visited.insert(np)
                            && let Some(t) = self.get_tile(np.0, np.1)
                        {
                            let passable = matches!(
                                t.tile_type,
                                TileType::Floor
                                    | TileType::DoorOpen
                                    | TileType::DoorClosed { .. }
                                    | TileType::StairsUp
                                    | TileType::StairsDown
                                    | TileType::Trap { .. }
                            );
                            if passable {
                                if t.tile_type == TileType::StairsDown {
                                    reached_any_down = true;
                                    break;
                                }
                                queue.push_back(np);
                            }
                        }
                    }
                }
                if reached_any_down {
                    break;
                }
            }

            if !reached_any_down {
                return false;
            }
        }

        true
    }

    pub fn find_random_floor_tile(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..4000 {
            let tx = rng.gen_range(1..(self.width - 1));
            let ty = rng.gen_range(1..(self.height - 1));
            if let Some(tile) = self.get_tile(tx, ty)
                && tile.tile_type == TileType::Floor
                && self.items.iter().all(|i| i.x != tx || i.y != ty)
            {
                return (tx, ty);
            }
        }
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                if let Some(tile) = self.get_tile(x, y)
                    && tile.tile_type == TileType::Floor
                {
                    return (x, y);
                }
            }
        }
        (self.width / 2, self.height / 2)
    }

    pub fn find_random_floor_tile_near(
        &self,
        center_x: usize,
        center_y: usize,
        max_dist: usize,
    ) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..500 {
            let dx = rng.gen_range(-(max_dist as isize)..=(max_dist as isize));
            let dy = rng.gen_range(-(max_dist as isize)..=(max_dist as isize));
            let tx = center_x as isize + dx;
            let ty = center_y as isize + dy;
            if tx > 0
                && ty > 0
                && (tx as usize) < self.width - 1
                && (ty as usize) < self.height - 1
                && let Some(tile) = self.get_tile(tx as usize, ty as usize)
                && tile.tile_type == TileType::Floor
            {
                return (tx as usize, ty as usize);
            }
        }
        self.find_random_floor_tile()
    }

    pub fn illuminate_area(&mut self, center_x: usize, center_y: usize, radius: usize) {
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                {
                    tile.visible = true;
                    tile.remembered = true;
                    tile.is_lit = true;
                }
            }
        }
    }

    pub fn map_area(&mut self, center_x: usize, center_y: usize, radius: usize) {
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                {
                    tile.remembered = true;
                }
            }
        }
    }

    pub fn detect_traps_near(&mut self, center_x: usize, center_y: usize, radius: usize) -> usize {
        let mut count = 0;
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                    && let TileType::Trap {
                        ref mut detected, ..
                    } = tile.tile_type
                {
                    *detected = true;
                    tile.remembered = true;
                    count += 1;
                }
            }
        }
        count
    }

    pub fn detect_doors_and_stairs_near(
        &mut self,
        center_x: usize,
        center_y: usize,
        radius: usize,
    ) -> usize {
        let mut count = 0;
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                {
                    match tile.tile_type {
                        TileType::SecretDoor => {
                            tile.tile_type = TileType::DoorClosed { spikes: 0 };
                            tile.remembered = true;
                            count += 1;
                        }
                        TileType::DoorClosed { .. }
                        | TileType::DoorOpen
                        | TileType::StairsUp
                        | TileType::StairsDown => {
                            tile.remembered = true;
                            count += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        count
    }

    pub fn detect_treasures_near(
        &mut self,
        center_x: usize,
        center_y: usize,
        radius: usize,
    ) -> usize {
        let item_coords: Vec<(usize, usize)> = self
            .items
            .iter()
            .map(|i| (i.x, i.y))
            .filter(|&(ix, iy)| {
                (ix as isize - center_x as isize)
                    .abs()
                    .max((iy as isize - center_y as isize).abs())
                    <= radius as isize
            })
            .collect();
        let mut count = item_coords.len();
        for (ix, iy) in item_coords {
            if let Some(tile) = self.get_tile_mut(ix, iy) {
                tile.remembered = true;
            }
        }
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                    && matches!(
                        tile.tile_type,
                        TileType::MagmaVein { gold: true } | TileType::QuartzVein { gold: true }
                    )
                {
                    tile.remembered = true;
                    count += 1;
                }
            }
        }
        count
    }

    pub fn destroy_adjacent_doors_and_traps(&mut self, center_x: usize, center_y: usize) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < self.width
                    && (ty as usize) < self.height
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                {
                    match tile.tile_type {
                        TileType::DoorClosed { .. } | TileType::SecretDoor => {
                            tile.tile_type = TileType::DoorOpen;
                            count += 1;
                        }
                        TileType::Trap { .. } => {
                            tile.tile_type = TileType::Floor;
                            count += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        count
    }

    pub fn place_glyph_of_warding(&mut self, x: usize, y: usize) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.tile_type = TileType::GlyphOfWarding;
        }
    }

    pub fn destroy_area(&mut self, center_x: usize, center_y: usize, radius: usize) {
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx > 0
                    && ty > 0
                    && (tx as usize) < self.width - 1
                    && (ty as usize) < self.height - 1
                    && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                {
                    tile.tile_type = TileType::Floor;
                }
            }
        }
        self.items.retain(|i| {
            let dist = (i.x as isize - center_x as isize)
                .abs()
                .max((i.y as isize - center_y as isize).abs());
            dist > radius as isize
        });
    }

    pub fn cause_earthquake<R: rand::Rng>(
        &mut self,
        center_x: usize,
        center_y: usize,
        radius: usize,
        rng: &mut R,
    ) {
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let tx = center_x as isize + dx;
                let ty = center_y as isize + dy;
                if tx > 0
                    && ty > 0
                    && (tx as usize) < self.width - 1
                    && (ty as usize) < self.height - 1
                {
                    if tx as usize == center_x && ty as usize == center_y {
                        continue;
                    }
                    if rng.gen_range(0..100) < 40
                        && let Some(tile) = self.get_tile_mut(tx as usize, ty as usize)
                    {
                        if tile.tile_type == TileType::Floor {
                            tile.tile_type = TileType::Rubble;
                        } else if matches!(
                            tile.tile_type,
                            TileType::Wall
                                | TileType::MagmaVein { .. }
                                | TileType::QuartzVein { .. }
                        ) {
                            tile.tile_type = TileType::Floor;
                        }
                    }
                }
            }
        }
    }

    fn count_walls_next_to(&self, x: usize, y: usize) -> i32 {
        let mut walls = 0;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;
            if let Some(t) = self.get_tile(nx, ny)
                && matches!(
                    t.tile_type,
                    TileType::Wall | TileType::MagmaVein { .. } | TileType::QuartzVein { .. }
                )
            {
                walls += 1;
            }
        }
        walls
    }

    fn place_stairs<R: Rng>(
        &mut self,
        stair_type: TileType,
        number: usize,
        mut walls: i32,
        rng: &mut R,
    ) {
        if self.width <= 14 || self.height <= 14 {
            return;
        }
        for _ in 0..number {
            let mut placed = false;
            let mut w_req = walls;
            while !placed {
                let mut j = 0;
                while !placed && j <= 30 {
                    let y1 = rnd(self.height as i32 - 14, rng) as usize;
                    let x1 = rnd(self.width as i32 - 14, rng) as usize;
                    let y2 = (y1 + 12).min(self.height - 1);
                    let x2 = (x1 + 12).min(self.width - 1);

                    let mut cy = y1;
                    while cy < y2 && !placed {
                        let mut cx = x1;
                        while cx < x2 && !placed {
                            let no_item = self.items.iter().all(|i| i.x != cx || i.y != cy);
                            if let Some(t) = self.get_tile(cx, cy)
                                && t.tile_type == TileType::Floor
                                && no_item
                                && self.count_walls_next_to(cx, cy) >= w_req
                            {
                                if let Some(tm) = self.get_tile_mut(cx, cy) {
                                    tm.tile_type = stair_type;
                                }
                                placed = true;
                            }
                            cx += 1;
                        }
                        cy += 1;
                    }
                    j += 1;
                }
                w_req -= 1;
                if w_req < -1 {
                    let (fx, fy) = self.find_random_floor_tile();
                    if let Some(tm) = self.get_tile_mut(fx, fy) {
                        tm.tile_type = stair_type;
                    }
                    placed = true;
                }
            }
            walls = walls.max(0);
        }
    }

    pub fn generate_simple_floor(&mut self) {
        let mut rng = rand::thread_rng();
        self.items.clear();
        self.rooms.clear();
        self.shops.clear();
        self.vault_monsters.clear();
        self.panel_row = 0;
        self.panel_col = 0;

        if self.depth == 0 {
            // --- CANONICAL UMORIA TOWN GENERATION (66 x 22) ---
            if self.width < SCREEN_WIDTH || self.height < SCREEN_HEIGHT {
                self.width = SCREEN_WIDTH;
                self.height = SCREEN_HEIGHT;
            }
            self.tiles = vec![Tile::new(TileType::Floor); self.width * self.height];
            for tile in self.tiles.iter_mut() {
                tile.is_lit = true;
            }

            let mut shop_types = [
                ShopType::General,
                ShopType::Armory,
                ShopType::Weaponsmith,
                ShopType::Temple,
                ShopType::Alchemy,
                ShopType::Magic,
            ];
            shop_types.shuffle(&mut rng);

            let mut idx = 0;
            for gy in 0..2 {
                for gx in 0..3 {
                    let shop_type = shop_types[idx];
                    idx += 1;

                    let yval = (gy * 10 + 5) as isize;
                    let xval = (gx * 16 + 16) as isize;
                    let height = (yval - rnd(3, &mut rng) as isize).max(1) as usize;
                    let depth =
                        (yval + rnd(4, &mut rng) as isize).min(self.height as isize - 2) as usize;
                    let left = (xval - rnd(6, &mut rng) as isize).max(1) as usize;
                    let right =
                        (xval + rnd(6, &mut rng) as isize).min(self.width as isize - 2) as usize;

                    for y in height..=depth {
                        for x in left..=right {
                            if let Some(tile) = self.get_tile_mut(x, y) {
                                tile.tile_type = TileType::Wall;
                            }
                        }
                    }

                    let tmp = rnd(4, &mut rng);
                    let (door_x, door_y) = if tmp < 3 {
                        let dy =
                            rnd((depth - height).max(1) as i32, &mut rng) as usize + height - 1;
                        let dx = if tmp == 1 { left } else { right };
                        (dx, dy)
                    } else {
                        let dx = rnd((right - left).max(1) as i32, &mut rng) as usize + left - 1;
                        let dy = if tmp == 3 { depth } else { height };
                        (dx, dy)
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
                    self.shops
                        .push(ShopInfo::new(door_x, door_y, shop_type, &mut rng));
                }
            }

            // Boundary walls
            for x in 0..self.width {
                if let Some(t) = self.get_tile_mut(x, 0) {
                    t.tile_type = TileType::Wall;
                }
                if let Some(t) = self.get_tile_mut(x, self.height - 1) {
                    t.tile_type = TileType::Wall;
                }
            }
            for y in 0..self.height {
                if let Some(t) = self.get_tile_mut(0, y) {
                    t.tile_type = TileType::Wall;
                }
                if let Some(t) = self.get_tile_mut(self.width - 1, y) {
                    t.tile_type = TileType::Wall;
                }
            }

            // Place 1 down staircase (`dungeonPlaceStairs(2, 1, 0)`)
            self.place_stairs(TileType::StairsDown, 1, 0, &mut rng);
        } else if self.width < SCREEN_WIDTH || self.height < SCREEN_HEIGHT {
            // Small test grids (e.g. 40x20 in unit tests): carve a simple open room
            self.tiles = vec![Tile::new(TileType::Wall); self.width * self.height];
            let is_lit = (self.depth as i32) <= rnd(25, &mut rng);
            for y in 1..(self.height - 1) {
                for x in 1..(self.width - 1) {
                    if let Some(t) = self.get_tile_mut(x, y) {
                        t.tile_type = TileType::Floor;
                        t.is_lit = is_lit;
                    }
                }
            }
            self.rooms.push(RoomInfo {
                x: 1,
                y: 1,
                w: self.width - 2,
                h: self.height - 2,
                is_lit,
            });
            if let Some(t) = self.get_tile_mut(2, 2) {
                t.tile_type = TileType::StairsUp;
            }
            if let Some(t) = self.get_tile_mut(self.width - 3, self.height - 3) {
                t.tile_type = TileType::StairsDown;
            }
        } else {
            // --- CANONICAL UMORIA DUNGEON GENERATION (198 x 66) ---
            self.width = MAX_WIDTH;
            self.height = MAX_HEIGHT;
            self.tiles = vec![Tile::new(TileType::Wall); self.width * self.height];

            let mut cave_gen = CaveGen::new(self.width, self.height, self.depth);

            let row_rooms = 2 * (self.height / SCREEN_HEIGHT);
            let col_rooms = 2 * (self.width / SCREEN_WIDTH);

            let mut room_map = vec![vec![false; col_rooms]; row_rooms];
            let random_room_count = rand_normal(DUN_ROOMS_MEAN, 2, &mut rng).max(2);
            for _ in 0..random_room_count {
                let r = (rnd(row_rooms as i32, &mut rng) - 1) as usize;
                let c = (rnd(col_rooms as i32, &mut rng) - 1) as usize;
                room_map[r][c] = true;
            }

            // Ensure at least 2 rooms are selected
            let selected_count: usize = room_map
                .iter()
                .map(|r| r.iter().filter(|&&b| b).count())
                .sum();
            if selected_count < 2 {
                room_map[0][0] = true;
                room_map[row_rooms - 1][col_rooms - 1] = true;
            }

            let mut locations: Vec<(isize, isize)> = Vec::new();
            for (row, row_cells) in room_map.iter().enumerate() {
                for (col, &has_room) in row_cells.iter().enumerate() {
                    if has_room {
                        let cy = (row * (SCREEN_HEIGHT >> 1)) as isize + QUART_HEIGHT;
                        let cx = (col * (SCREEN_WIDTH >> 1)) as isize + QUART_WIDTH;
                        if (self.depth as i32) > rnd(DUN_UNUSUAL_ROOMS, &mut rng) {
                            match rnd(3, &mut rng) {
                                1 => cave_gen.build_room_overlapping_rectangles(cy, cx, &mut rng),
                                2 => cave_gen.build_room_with_inner_rooms(cy, cx, &mut rng),
                                _ => cave_gen.build_room_cross_shaped(cy, cx, &mut rng),
                            }
                        } else {
                            cave_gen.build_room(cy, cx, &mut rng);
                        }
                        locations.push((cy, cx));
                    }
                }
            }

            // Shuffle room locations (matching Umoria lines 1039-1049)
            let loc_len = locations.len();
            for _ in 0..loc_len {
                let pick1 = (rnd(loc_len as i32, &mut rng) - 1) as usize;
                let pick2 = (rnd(loc_len as i32, &mut rng) - 1) as usize;
                locations.swap(pick1, pick2);
            }

            // Build tunnels connecting rooms in a loop (matching Umoria lines 1054-1059)
            for i in 0..loc_len {
                let (sy, sx) = locations[(i + 1) % loc_len];
                let (ey, ex) = locations[i];
                cave_gen.build_tunnel(sy, sx, ey, ex, &mut rng);
            }

            // Fill remaining empty / temporary walls with granite
            cave_gen.fill_empty_tiles_with(GenFeature::GraniteWall);

            // Magma and Quartz streamers
            for _ in 0..DUN_MAGMA_STREAMER {
                cave_gen.place_streamer_rock(GenFeature::MagmaWall, DUN_MAGMA_TREASURE, &mut rng);
            }
            for _ in 0..DUN_QUARTZ_STREAMER {
                cave_gen.place_streamer_rock(GenFeature::QuartzWall, DUN_QUARTZ_TREASURE, &mut rng);
            }
            cave_gen.place_boundary_walls();

            // Place intersection doors
            let doors_tk = cave_gen.doors_tk.clone();
            for (dy, dx) in doors_tk {
                let y = dy as isize;
                let x = dx as isize;
                cave_gen.place_door_if_next_to_two_walls(y, x - 1, &mut rng);
                cave_gen.place_door_if_next_to_two_walls(y, x + 1, &mut rng);
                cave_gen.place_door_if_next_to_two_walls(y - 1, x, &mut rng);
                cave_gen.place_door_if_next_to_two_walls(y + 1, x, &mut rng);
            }

            self.rooms = cave_gen.rooms.clone();
            self.vault_monsters = cave_gen.vault_monsters.clone();

            // Materialize tiles and vault objects/traps from CaveGen
            for y in 0..self.height {
                for x in 0..self.width {
                    let c = cave_gen.cell(y, x);
                    let is_lit = c.feature == GenFeature::LightFloor
                        || (c.perma_lit_room
                            && self.rooms.iter().any(|r| {
                                r.is_lit
                                    && x >= r.x.saturating_sub(1)
                                    && x <= r.x + r.w
                                    && y >= r.y.saturating_sub(1)
                                    && y <= r.y + r.h
                            }));

                    let tile_type = match c.overlay {
                        GenOverlay::OpenDoor | GenOverlay::BrokenDoor => TileType::DoorOpen,
                        GenOverlay::ClosedDoor
                        | GenOverlay::LockedDoor(_)
                        | GenOverlay::StuckDoor(_) => TileType::DoorClosed { spikes: 0 },
                        GenOverlay::SecretDoor => TileType::SecretDoor,
                        GenOverlay::Trap(trap_type) => TileType::Trap {
                            detected: false,
                            trap_type,
                        },
                        GenOverlay::VeinGold => match c.feature {
                            GenFeature::MagmaWall => TileType::MagmaVein { gold: true },
                            GenFeature::QuartzWall => TileType::QuartzVein { gold: true },
                            _ => TileType::Wall,
                        },
                        GenOverlay::Object | GenOverlay::Gold | GenOverlay::None => {
                            match c.feature {
                                GenFeature::DarkFloor
                                | GenFeature::LightFloor
                                | GenFeature::CorrFloor
                                | GenFeature::BlockedFloor => TileType::Floor,
                                GenFeature::MagmaWall => TileType::MagmaVein { gold: false },
                                GenFeature::QuartzWall => TileType::QuartzVein { gold: false },
                                _ => TileType::Wall,
                            }
                        }
                    };

                    if let Some(tile) = self.get_tile_mut(x, y) {
                        tile.tile_type = tile_type;
                        tile.is_lit = is_lit;
                    }

                    if c.overlay == GenOverlay::Object && tile_type == TileType::Floor {
                        let item = generate_random_floor_item(self.depth, &mut rng);
                        self.items.push(FloorItem { x, y, item });
                    } else if c.overlay == GenOverlay::Gold && tile_type == TileType::Floor {
                        let item = generate_canonical_gold(self.depth, &mut rng);
                        self.items.push(FloorItem { x, y, item });
                    }
                }
            }

            let alloc_level = (self.depth as i32 / 3).clamp(2, 10);

            // Place stairs: 3-4 down stairs, 1-2 up stairs (matching Umoria lines 1086-1087)
            if self.depth < self.max_depth {
                let num_down = (rnd(2, &mut rng) + 2) as usize;
                self.place_stairs(TileType::StairsDown, num_down, 3, &mut rng);
            }
            let num_up = rnd(2, &mut rng) as usize;
            self.place_stairs(TileType::StairsUp, num_up, 3, &mut rng);

            // Allocate and place objects (matching Umoria lines 1096-1100):
            // 1. Corridor rubble: rnd(alloc_level)
            let num_rubble = rnd(alloc_level, &mut rng).max(0) as usize;
            for _ in 0..num_rubble {
                for _ in 0..500 {
                    let rx = rng.gen_range(1..(self.width - 1));
                    let ry = rng.gen_range(1..(self.height - 1));
                    let no_item = self.items.iter().all(|i| i.x != rx || i.y != ry);
                    if cave_gen.cell(ry, rx).feature.is_corridor()
                        && no_item
                        && let Some(t) = self.get_tile_mut(rx, ry)
                        && t.tile_type == TileType::Floor
                    {
                        t.tile_type = TileType::Rubble;
                        break;
                    }
                }
            }

            // 2. Room objects: rand_normal(LEVEL_OBJECTS_PER_ROOM, 3)
            let num_room_objs = rand_normal(LEVEL_OBJECTS_PER_ROOM, 3, &mut rng).max(0) as usize;
            for _ in 0..num_room_objs {
                for _ in 0..500 {
                    let rx = rng.gen_range(1..(self.width - 1));
                    let ry = rng.gen_range(1..(self.height - 1));
                    if cave_gen.cell(ry, rx).feature.is_room_floor()
                        && let Some(t) = self.get_tile(rx, ry)
                        && t.tile_type == TileType::Floor
                        && self.items.iter().all(|i| i.x != rx || i.y != ry)
                    {
                        let item = generate_random_floor_item(self.depth, &mut rng);
                        self.items.push(FloorItem { x: rx, y: ry, item });
                        break;
                    }
                }
            }

            // 3. Floor objects: rand_normal(LEVEL_OBJECTS_PER_CORRIDOR, 3)
            let num_floor_objs =
                rand_normal(LEVEL_OBJECTS_PER_CORRIDOR, 3, &mut rng).max(0) as usize;
            for _ in 0..num_floor_objs {
                let (ix, iy) = self.find_random_floor_tile();
                if let Some(t) = self.get_tile(ix, iy)
                    && t.tile_type == TileType::Floor
                {
                    let item = generate_random_floor_item(self.depth, &mut rng);
                    self.items.push(FloorItem { x: ix, y: iy, item });
                }
            }

            // 4. Gold and gems: rand_normal(LEVEL_TOTAL_GOLD_AND_GEMS, 3)
            let num_gold = rand_normal(LEVEL_TOTAL_GOLD_AND_GEMS, 3, &mut rng).max(0) as usize;
            for _ in 0..num_gold {
                let (gx, gy) = self.find_random_floor_tile();
                if let Some(t) = self.get_tile(gx, gy)
                    && t.tile_type == TileType::Floor
                {
                    let item = generate_canonical_gold(self.depth, &mut rng);
                    self.items.push(FloorItem { x: gx, y: gy, item });
                }
            }

            // 5. Floor traps: rnd(alloc_level)
            let num_traps = rnd(alloc_level, &mut rng).max(0) as usize;
            for _ in 0..num_traps {
                let (tx, ty) = self.find_random_floor_tile();
                if let Some(t) = self.get_tile_mut(tx, ty)
                    && t.tile_type == TileType::Floor
                {
                    t.tile_type = TileType::Trap {
                        detected: false,
                        trap_type: random_trap_type(&mut rng),
                    };
                }
            }

            // Ensure all rooms and stairs are reachable via normal doors/floors
            self.ensure_level_connectivity();
        }
    }

    fn ensure_level_connectivity(&mut self) {
        // 1. Ensure every room has at least one non-secret entrance
        for r_idx in 0..self.rooms.len() {
            let room = self.rooms[r_idx].clone();
            let mut has_open_or_closed = false;
            let mut secret_doors = Vec::new();
            for y in (room.y.saturating_sub(1))..=(room.y + room.h).min(self.height - 1) {
                for x in (room.x.saturating_sub(1))..=(room.x + room.w).min(self.width - 1) {
                    let is_perimeter = x == room.x.saturating_sub(1)
                        || x == room.x + room.w
                        || y == room.y.saturating_sub(1)
                        || y == room.y + room.h;
                    if is_perimeter && let Some(t) = self.get_tile(x, y) {
                        if matches!(
                            t.tile_type,
                            TileType::Floor | TileType::DoorOpen | TileType::DoorClosed { .. }
                        ) {
                            has_open_or_closed = true;
                        } else if t.tile_type == TileType::SecretDoor {
                            secret_doors.push((x, y));
                        }
                    }
                }
            }
            if !has_open_or_closed
                && let Some(&(sx, sy)) = secret_doors.first()
                && let Some(t) = self.get_tile_mut(sx, sy)
            {
                t.tile_type = TileType::DoorClosed { spikes: 0 };
            }
        }

        // 2. If any up stairs cannot reach down stairs due to secret doors or rubble,
        // convert blocking secret doors to closed doors and rubble to floor.
        if !self.stairs_are_reachable() {
            for tile in self.tiles.iter_mut() {
                if tile.tile_type == TileType::SecretDoor {
                    tile.tile_type = TileType::DoorClosed { spikes: 0 };
                }
            }
        }
        if !self.stairs_are_reachable() {
            for tile in self.tiles.iter_mut() {
                if matches!(tile.tile_type, TileType::Rubble | TileType::Chest { .. }) {
                    tile.tile_type = TileType::Floor;
                }
            }
        }

        // 3. If a tunnel stopped early and left a stair component disconnected, carve a connecting corridor.
        if !self.stairs_are_reachable() {
            let up_positions: Vec<(usize, usize)> = self
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.tile_type == TileType::StairsUp)
                .map(|(i, _)| (i % self.width, i / self.width))
                .collect();
            let down_positions: Vec<(usize, usize)> = self
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.tile_type == TileType::StairsDown)
                .map(|(i, _)| (i % self.width, i / self.width))
                .collect();

            if let Some(&(dx, dy)) = down_positions.first() {
                for &(ux, uy) in &up_positions {
                    let start_x = ux.min(dx);
                    let end_x = ux.max(dx);
                    for x in start_x..=end_x {
                        if let Some(t) = self.get_tile_mut(x, uy)
                            && matches!(
                                t.tile_type,
                                TileType::Wall
                                    | TileType::MagmaVein { .. }
                                    | TileType::QuartzVein { .. }
                                    | TileType::SecretDoor
                                    | TileType::Rubble
                            )
                        {
                            t.tile_type = TileType::Floor;
                        }
                    }
                    let start_y = uy.min(dy);
                    let end_y = uy.max(dy);
                    for y in start_y..=end_y {
                        if let Some(t) = self.get_tile_mut(dx, y)
                            && matches!(
                                t.tile_type,
                                TileType::Wall
                                    | TileType::MagmaVein { .. }
                                    | TileType::QuartzVein { .. }
                                    | TileType::SecretDoor
                                    | TileType::Rubble
                            )
                        {
                            t.tile_type = TileType::Floor;
                        }
                    }
                }
            }
        }
    }

    pub fn has_los_step(
        &self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
    ) -> Option<(usize, usize)> {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = (y2 as isize - y1 as isize).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut cx = x1 as isize;
        let mut cy = y1 as isize;
        let mut prev_x = cx;
        let mut prev_y = cy;

        loop {
            if cx == x2 as isize && cy == y2 as isize {
                return Some((prev_x as usize, prev_y as usize));
            }
            if (cx != x1 as isize || cy != y1 as isize)
                && let Some(tile) = self.get_tile(cx as usize, cy as usize)
                && !tile.is_passable()
            {
                return None;
            }
            let e2 = 2 * err;
            let mut stepped_x = false;
            let mut stepped_y = false;
            prev_x = cx;
            prev_y = cy;
            if e2 > -dy {
                err -= dy;
                cx += sx;
                stepped_x = true;
            }
            if e2 < dx {
                err += dx;
                cy += sy;
                stepped_y = true;
            }
            // Prevent rays from slipping diagonally through between two impassable corner blocks
            if stepped_x && stepped_y {
                let c1_blocked = self
                    .get_tile(cx as usize, prev_y as usize)
                    .map(|t| !t.is_passable())
                    .unwrap_or(true);
                let c2_blocked = self
                    .get_tile(prev_x as usize, cy as usize)
                    .map(|t| !t.is_passable())
                    .unwrap_or(true);
                if c1_blocked && c2_blocked {
                    return None;
                }
            }
        }
    }

    pub fn has_los(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        self.has_los_step(x1, y1, x2, y2).is_some()
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
                let player_inside = player.x >= room.x
                    && player.x < room.x + room.w
                    && player.y >= room.y
                    && player.y < room.y + room.h;
                if player_inside {
                    lit_rooms_to_reveal.push(room.clone());
                }
            }
        }

        // Also include any overlapping lit rooms (e.g. Type 1 overlapping rects or Type 3 cross arms)
        let mut added = true;
        while added {
            added = false;
            for room in &self.rooms {
                if room.is_lit
                    && !lit_rooms_to_reveal
                        .iter()
                        .any(|r| r.x == room.x && r.y == room.y && r.w == room.w && r.h == room.h)
                    && lit_rooms_to_reveal.iter().any(|r| {
                        room.x <= r.x + r.w
                            && room.x + room.w >= r.x
                            && room.y <= r.y + r.h
                            && room.y + room.h >= r.y
                    })
                {
                    lit_rooms_to_reveal.push(room.clone());
                    added = true;
                }
            }
        }

        let radius = player.get_light_radius();
        let max_dist = 8usize;

        // Step 1: Reveal lit rooms when player is inside (matching Umoria dungeonLightRoom)
        for room in &lit_rooms_to_reveal {
            let y_min = room.y.saturating_sub(1);
            let y_max = (room.y + room.h).min(self.height - 1);
            let x_min = room.x.saturating_sub(1);
            let x_max = (room.x + room.w).min(self.width - 1);
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    if let Some(tile) = self.get_tile_mut(x, y) {
                        tile.visible = true;
                        tile.remembered = true;
                    }
                }
            }
        }

        // Step 2: Line of sight checks within max_dist of player
        let py_min = player.y.saturating_sub(max_dist);
        let py_max = (player.y + max_dist).min(self.height - 1);
        let px_min = player.x.saturating_sub(max_dist);
        let px_max = (player.x + max_dist).min(self.width - 1);

        for y in py_min..=py_max {
            for x in px_min..=px_max {
                if let Some(tile) = self.get_tile(x, y)
                    && tile.visible
                {
                    continue;
                }

                let dx = (x as isize - player.x as isize).abs();
                let dy = (y as isize - player.y as isize).abs();
                let dist = dx.max(dy) as usize;

                if dist <= max_dist && self.has_los(player.x, player.y, x, y) {
                    let is_in_lit_room = self.rooms.iter().any(|r| {
                        r.is_lit && x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
                    });
                    let is_tile_lit = self.get_tile(x, y).map(|t| t.is_lit).unwrap_or(false);

                    if (dist <= radius || is_tile_lit || is_in_lit_room)
                        && let Some(tile) = self.get_tile_mut(x, y)
                    {
                        tile.visible = true;
                        tile.remembered = true;
                    }
                }
            }
        }

        // Step 3: Walls adjacent to visible, illuminated passable tiles
        let mut illuminated_walls = Vec::new();
        for y in py_min..=py_max {
            for x in px_min..=px_max {
                if let Some(tile) = self.get_tile(x, y)
                    && !tile.visible
                    && !tile.is_passable()
                {
                    let mut touches_lit_visible = false;
                    for (ndx, ndy) in &[
                        (0, 1),
                        (0, -1),
                        (1, 0),
                        (-1, 0),
                        (1, 1),
                        (1, -1),
                        (-1, 1),
                        (-1, -1),
                    ] {
                        let nx = (x as isize + ndx) as usize;
                        let ny = (y as isize + ndy) as usize;
                        if let Some(adj) = self.get_tile(nx, ny)
                            && adj.is_passable()
                            && adj.visible
                            && adj.is_lit
                        {
                            touches_lit_visible = true;
                            break;
                        }
                    }
                    if touches_lit_visible {
                        illuminated_walls.push((x, y));
                    }
                }
            }
        }

        for (wx, wy) in illuminated_walls {
            if let Some(tile) = self.get_tile_mut(wx, wy) {
                tile.visible = true;
                tile.remembered = true;
            }
        }
    }

    pub fn generate_monsters(&self, player_has_killed_balrog: bool) -> Vec<Monster> {
        let depth = self.depth;
        let max_depth = self.max_depth;
        let mut rng = rand::thread_rng();

        if depth == 0 {
            let mut mons = Vec::new();
            let count = rng.gen_range(4..=8);
            for _ in 0..count {
                let (tx, ty) = self.find_random_floor_tile();
                let c_idx = rng.gen_range(0..8);
                mons.push(Monster::from_creature_id(c_idx, tx, ty, false, &mut rng));
            }
            return mons;
        }

        let mut mons = Vec::new();

        // Spawn vault monsters from unusual rooms (matching dungeonPlaceVaultMonster)
        for &(vx, vy) in &self.vault_monsters {
            let (mx, my) = self.find_random_floor_tile_near(vx, vy, 3);
            if mons.iter().all(|m: &Monster| m.x != mx || m.y != my) {
                let c_idx = crate::entity::monster_data::get_monster_for_level(
                    depth.saturating_add(2),
                    &mut rng,
                );
                mons.push(Monster::from_creature_id(c_idx, mx, my, true, &mut rng));
            }
        }

        if depth == max_depth && !player_has_killed_balrog {
            let (bx, by) = self.find_random_floor_tile();
            mons.push(Monster::from_creature_id(278, bx, by, false, &mut rng));
        }

        let alloc_level = (depth / 3).clamp(2, 10);
        let count = rng.gen_range(1..=8) + 14 + alloc_level;
        for _ in 0..count {
            let (mx, my) = self.find_random_floor_tile();
            let c_idx = crate::entity::monster_data::get_monster_for_level(depth, &mut rng);
            let sleeping = true;
            mons.push(Monster::from_creature_id(c_idx, mx, my, sleeping, &mut rng));
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
                status_msg.push_str(&format!(
                    " The chest sprays poison gas! You take {} poison damage.",
                    dmg
                ));
            }
            _ => {
                status_msg.push_str(" The chest summons a monster!");
                let adj_tiles = &[
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ];
                let mut spawned = false;
                for &(dx, dy) in adj_tiles {
                    let sx = (px as isize + dx) as usize;
                    let sy = (py as isize + dy) as usize;
                    if let Some(t) = self.get_tile(sx, sy)
                        && t.is_passable()
                        && monsters.iter().all(|m| m.x != sx || m.y != sy)
                    {
                        let c_idx =
                            crate::entity::monster_data::get_monster_for_level(self.depth, rng);
                        monsters.push(Monster::from_creature_id(c_idx, sx, sy, false, rng));
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

pub fn get_shop_items(shop: ShopType) -> Vec<(String, u32, ItemType)> {
    crate::entity::treasure_data::generate_canonical_shop_items(shop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Class, Player, Race};

    #[test]
    fn test_umoria_doors_and_dimensions() {
        let mut total_doors = 0;
        let mut total_secret_doors = 0;
        let mut total_open_doors = 0;
        let mut total_closed_doors = 0;

        for depth in 1..=20 {
            let mut level = DungeonLevel::new(MAX_WIDTH, MAX_HEIGHT, depth, 50);
            level.generate_simple_floor();

            assert_eq!(level.width, MAX_WIDTH);
            assert_eq!(level.height, MAX_HEIGHT);

            for y in 0..level.height {
                for x in 0..level.width {
                    if let Some(tile) = level.get_tile(x, y) {
                        match tile.tile_type {
                            TileType::DoorClosed { .. } => {
                                total_doors += 1;
                                total_closed_doors += 1;
                            }
                            TileType::DoorOpen => {
                                total_doors += 1;
                                total_open_doors += 1;
                            }
                            TileType::SecretDoor => {
                                total_doors += 1;
                                total_secret_doors += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        assert!(
            total_doors > 50,
            "Expected doors to be generated, got {}",
            total_doors
        );
        assert!(total_open_doors > 0, "Expected open doors to be generated");
        assert!(
            total_closed_doors > 0,
            "Expected closed doors to be generated"
        );
        assert!(
            total_secret_doors > 0,
            "Expected secret doors to be generated"
        );
    }

    #[test]
    fn test_secret_door_and_corners_block_los() {
        let mut level = DungeonLevel::new(10, 10, 1, 50);
        for tile in level.tiles.iter_mut() {
            tile.tile_type = TileType::Floor;
        }

        // 1. Direct SecretDoor between player (1, 1) and target (3, 1)
        level.get_tile_mut(2, 1).unwrap().tile_type = TileType::SecretDoor;
        assert!(
            !level.has_los(1, 1, 3, 1),
            "SecretDoor must block line of sight!"
        );

        // 2. Open door allows LOS
        level.get_tile_mut(2, 1).unwrap().tile_type = TileType::DoorOpen;
        assert!(
            level.has_los(1, 1, 3, 1),
            "DoorOpen should allow line of sight"
        );

        // 3. Diagonal corner pinch between two walls
        level.get_tile_mut(2, 1).unwrap().tile_type = TileType::Wall;
        level.get_tile_mut(1, 2).unwrap().tile_type = TileType::Wall;
        assert!(
            !level.has_los(1, 1, 2, 2),
            "Diagonal wall corner pinch must block line of sight!"
        );
    }

    #[test]
    fn test_all_rooms_have_accessible_entrance() {
        for depth in 1..=10 {
            let mut level = DungeonLevel::new(MAX_WIDTH, MAX_HEIGHT, depth, 50);
            level.generate_simple_floor();

            for r_idx in 0..level.rooms.len() {
                let room = &level.rooms[r_idx];
                let mut has_accessible_entrance = false;
                for y in (room.y.saturating_sub(1))..=(room.y + room.h) {
                    for x in (room.x.saturating_sub(1))..=(room.x + room.w) {
                        let is_perimeter = x == room.x.saturating_sub(1)
                            || x == room.x + room.w
                            || y == room.y.saturating_sub(1)
                            || y == room.y + room.h;
                        if is_perimeter
                            && let Some(tile) = level.get_tile(x, y)
                            && matches!(
                                tile.tile_type,
                                TileType::Floor | TileType::DoorOpen | TileType::DoorClosed { .. }
                            )
                        {
                            has_accessible_entrance = true;
                            break;
                        }
                    }
                    if has_accessible_entrance {
                        break;
                    }
                }
                assert!(
                    has_accessible_entrance,
                    "Room {} on depth {} has no accessible entrance!",
                    r_idx, depth
                );
            }
        }
    }

    #[test]
    fn test_canonical_umoria_lighting_and_viewport() {
        // Level 1: depth <= rnd(25) means 100% of rooms are lit, and corridors are dark
        let mut l1_room_lit = 0;
        let mut l1_room_total = 0;

        for _ in 0..10 {
            let mut level = DungeonLevel::new(MAX_WIDTH, MAX_HEIGHT, 1, 50);
            level.generate_simple_floor();

            for r in &level.rooms {
                l1_room_total += 1;
                if r.is_lit {
                    l1_room_lit += 1;
                }
            }
        }
        assert_eq!(
            l1_room_lit, l1_room_total,
            "On level 1, all rooms must be lit (1 <= rnd(25))"
        );

        // Level 13: depth <= rnd(25) means ~13/25 = ~52% of rooms are lit
        let mut l13_room_lit = 0;
        let mut l13_room_total = 0;
        for _ in 0..30 {
            let mut level = DungeonLevel::new(MAX_WIDTH, MAX_HEIGHT, 13, 50);
            level.generate_simple_floor();
            for r in &level.rooms {
                l13_room_total += 1;
                if r.is_lit {
                    l13_room_lit += 1;
                }
            }
        }
        let l13_ratio = l13_room_lit as f64 / l13_room_total as f64;
        assert!(
            (0.38..=0.66).contains(&l13_ratio),
            "Expected ~52% lit rooms on level 13, got {:.2}",
            l13_ratio
        );

        // Level 26: depth <= rnd(25) means 0% of rooms are lit
        let mut level26 = DungeonLevel::new(MAX_WIDTH, MAX_HEIGHT, 26, 50);
        level26.generate_simple_floor();
        assert!(
            level26.rooms.iter().all(|r| !r.is_lit),
            "On level 26+, all rooms must be dark"
        );

        // Verify viewport panel scrolling over 198x66 dungeon
        let (top, bottom, left, right) = level26.get_viewport_bounds(180, 60);
        assert_eq!(bottom - top, SCREEN_HEIGHT);
        assert_eq!(right - left, SCREEN_WIDTH);
        assert!(180 >= left && 180 < right);
        assert!(60 >= top && 60 < bottom);
    }

    #[test]
    fn test_corridor_lighting_fov() {
        let mut level = DungeonLevel::new(20, 10, 1, 50);
        // Build a horizontal corridor at y = 5 from x = 1 to x = 15
        for x in 1..=15 {
            let t = level.get_tile_mut(x, 5).unwrap();
            t.tile_type = TileType::Floor;
            t.is_lit = true;
        }

        let player = Player::new("TestLit", Race::Human, Class::Warrior, 2, 5);
        // Default player radius is 1 (no torch)
        assert_eq!(player.get_light_radius(), 1);

        level.update_fov(&player);

        // Lit corridor tiles in LOS up to distance 8 should be visible
        for x in 2..=10 {
            let t = level.get_tile(x, 5).unwrap();
            assert!(
                t.visible,
                "Tile at ({}, 5) in lit corridor should be visible in LOS",
                x
            );
        }
        // Walls adjacent to lit corridor floor should also be visible in LOS
        for x in 2..=10 {
            let top_wall = level.get_tile(x, 4).unwrap();
            let bot_wall = level.get_tile(x, 6).unwrap();
            assert!(
                top_wall.visible,
                "Top wall at ({}, 4) bordering lit corridor should be visible in LOS",
                x
            );
            assert!(
                bot_wall.visible,
                "Bottom wall at ({}, 6) bordering lit corridor should be visible in LOS",
                x
            );
        }
        // Beyond distance 8 (e.g. x = 11, dist = 9) should NOT be visible
        let distant_tile = level.get_tile(11, 5).unwrap();
        assert!(
            !distant_tile.visible,
            "Tile at distance 9 should not be visible beyond max_dist 8"
        );

        // Now darken the corridor
        for x in 1..=15 {
            level.get_tile_mut(x, 5).unwrap().is_lit = false;
        }

        level.update_fov(&player);

        // At dist <= 1, visible from player's personal light
        assert!(level.get_tile(2, 5).unwrap().visible);
        assert!(level.get_tile(3, 5).unwrap().visible);

        // At dist >= 2 in dark corridor, should NOT be visible
        for x in 4..=10 {
            let t = level.get_tile(x, 5).unwrap();
            assert!(
                !t.visible,
                "Tile at ({}, 5) in dark corridor should NOT be visible beyond light radius",
                x
            );
        }
        // Adjacent walls at dist >= 2 in dark corridor should NOT be visible
        for x in 4..=10 {
            let top_wall = level.get_tile(x, 4).unwrap();
            assert!(
                !top_wall.visible,
                "Top wall at ({}, 4) in dark corridor should NOT be visible beyond light radius",
                x
            );
        }
    }
}
