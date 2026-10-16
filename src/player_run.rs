use crate::dungeon::{DungeonLevel, TileType};
use crate::entity::monster::Monster;
use crate::input::Direction;

pub const CYCLE: [u8; 17] = [1, 2, 3, 6, 9, 8, 7, 4, 1, 2, 3, 6, 9, 8, 7, 4, 1];
pub const CHOME: [usize; 10] = [0, 8, 9, 10, 7, 0, 11, 6, 5, 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunState {
    pub direction: Direction,
    pub prev_dir: Direction,
    pub open_area: bool,
    pub break_left: bool,
    pub break_right: bool,
    pub steps: u32,
    pub is_running: bool,
    pub cut_corners: bool,
    pub examine_corners: bool,
}

pub fn is_dungeon_wall(level: &DungeonLevel, x: usize, y: usize) -> bool {
    if let Some(tile) = level.get_tile(x, y) {
        matches!(
            tile.tile_type,
            TileType::Wall
                | TileType::SecretDoor
                | TileType::MagmaVein { .. }
                | TileType::QuartzVein { .. }
                | TileType::Rubble
        )
    } else {
        true
    }
}

pub fn move_coord(
    x: usize,
    y: usize,
    dir: Direction,
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    let (dx, dy) = dir.to_dx_dy();
    let nx = x as isize + dx;
    let ny = y as isize + dy;
    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
        Some((nx as usize, ny as usize))
    } else {
        None
    }
}

pub fn init_run(
    direction: Direction,
    px: usize,
    py: usize,
    level: &DungeonLevel,
    is_blind: bool,
) -> RunState {
    let mut state = RunState {
        direction,
        prev_dir: direction,
        open_area: true,
        break_left: false,
        break_right: false,
        steps: 0,
        is_running: true,
        cut_corners: true,
        examine_corners: true,
    };

    let dir_val = direction as usize;
    if dir_val == 5 || dir_val == 0 || dir_val > 9 {
        state.is_running = false;
        return state;
    }

    let next = match move_coord(px, py, direction, level.width, level.height) {
        Some(pos) => pos,
        None => {
            state.is_running = false;
            return state;
        }
    };

    if is_dungeon_wall(level, next.0, next.1) {
        state.is_running = false;
        return state;
    }

    if is_blind {
        return state;
    }

    let cycle_index = CHOME[dir_val];
    let left_dir = Direction::from_u8(CYCLE[cycle_index + 1]).unwrap();
    let right_dir = Direction::from_u8(CYCLE[cycle_index - 1]).unwrap();

    let short_left = move_coord(px, py, left_dir, level.width, level.height)
        .map(|(x, y)| is_dungeon_wall(level, x, y))
        .unwrap_or(true);
    let deep_left = move_coord(next.0, next.1, left_dir, level.width, level.height)
        .map(|(x, y)| is_dungeon_wall(level, x, y))
        .unwrap_or(true);
    state.break_left = short_left || deep_left;

    let short_right = move_coord(px, py, right_dir, level.width, level.height)
        .map(|(x, y)| is_dungeon_wall(level, x, y))
        .unwrap_or(true);
    let deep_right = move_coord(next.0, next.1, right_dir, level.width, level.height)
        .map(|(x, y)| is_dungeon_wall(level, x, y))
        .unwrap_or(true);
    state.break_right = short_right || deep_right;

    if state.break_left && state.break_right {
        state.open_area = false;

        if (dir_val & 1) != 0 {
            if deep_left && !deep_right {
                state.prev_dir = right_dir;
            } else if deep_right && !deep_left {
                state.prev_dir = left_dir;
            }
        } else if let Some(ahead2) =
            move_coord(next.0, next.1, direction, level.width, level.height)
            && is_dungeon_wall(level, ahead2.0, ahead2.1)
        {
            if short_left && !short_right {
                state.prev_dir = Direction::from_u8(CYCLE[cycle_index - 2]).unwrap();
            } else if short_right && !short_left {
                state.prev_dir = Direction::from_u8(CYCLE[cycle_index + 2]).unwrap();
            }
        }
    } else {
        state.open_area = true;
    }

    state
}

pub fn update_run_after_step(
    state: &mut RunState,
    px: usize,
    py: usize,
    level: &DungeonLevel,
    monsters: &[Monster],
    is_blind: bool,
) -> bool {
    state.steps += 1;
    if state.steps >= 100 {
        state.is_running = false;
        return false;
    }

    if is_blind {
        return state.is_running;
    }

    if level.items.iter().any(|i| i.x == px && i.y == py) {
        state.is_running = false;
        return false;
    }

    let dir_val = state.prev_dir as usize;
    if dir_val == 0 || dir_val > 9 {
        state.is_running = false;
        return false;
    }
    let max = (dir_val & 1) + 1;
    let cycle_idx = CHOME[dir_val];

    let mut check_dir = 0;
    let mut dir_a = 0;
    let mut dir_b = 0;

    for i in -(max as isize)..=(max as isize) {
        let new_dir_num = CYCLE[(cycle_idx as isize + i) as usize];
        let new_dir = match Direction::from_u8(new_dir_num) {
            Some(d) => d,
            None => continue,
        };
        let spot = match move_coord(px, py, new_dir, level.width, level.height) {
            Some(s) => s,
            None => continue,
        };

        if monsters.iter().any(|m| m.x == spot.0 && m.y == spot.1)
            && level
                .get_tile(spot.0, spot.1)
                .map(|t| t.visible)
                .unwrap_or(false)
        {
            state.is_running = false;
            return false;
        }

        if level.items.iter().any(|i| i.x == spot.0 && i.y == spot.1) {
            state.is_running = false;
            return false;
        }
        if let Some(tile) = level.get_tile(spot.0, spot.1) {
            if matches!(
                tile.tile_type,
                TileType::DoorOpen
                    | TileType::DoorClosed { .. }
                    | TileType::Trap { .. }
                    | TileType::Chest { .. }
            ) {
                state.is_running = false;
                return false;
            }

            let is_open = !is_dungeon_wall(level, spot.0, spot.1);
            if is_open {
                if state.open_area {
                    if (i < 0 && state.break_right) || (i > 0 && state.break_left) {
                        state.is_running = false;
                        return false;
                    }
                } else if dir_a == 0 {
                    dir_a = new_dir_num;
                } else if dir_b != 0 {
                    state.is_running = false;
                    return false;
                } else {
                    let prev_cycle_dir = CYCLE[(cycle_idx as isize + i - 1) as usize];
                    if dir_a != prev_cycle_dir {
                        state.is_running = false;
                        return false;
                    }
                    if (new_dir_num & 1) == 1 {
                        check_dir = CYCLE[(cycle_idx as isize + i - 2) as usize];
                        dir_b = new_dir_num;
                    } else {
                        check_dir = CYCLE[(cycle_idx as isize + i + 1) as usize];
                        dir_b = dir_a;
                        dir_a = new_dir_num;
                    }
                }
            } else if state.open_area {
                if i < 0 {
                    if state.break_left {
                        state.is_running = false;
                        return false;
                    }
                    state.break_right = true;
                } else if i > 0 {
                    if state.break_right {
                        state.is_running = false;
                        return false;
                    }
                    state.break_left = true;
                }
            }
        }
    }

    if state.open_area {
        return state.is_running;
    }

    if dir_b == 0 || (state.examine_corners && !state.cut_corners) {
        if dir_a != 0 {
            state.direction = Direction::from_u8(dir_a).unwrap();
        }
        if dir_b == 0 {
            state.prev_dir = Direction::from_u8(dir_a).unwrap_or(state.direction);
        } else {
            state.prev_dir = Direction::from_u8(dir_b).unwrap_or(state.direction);
        }
        return state.is_running;
    }

    let dir_a_dir = Direction::from_u8(dir_a).unwrap();
    let loc_a = move_coord(px, py, dir_a_dir, level.width, level.height);
    let check_d = Direction::from_u8(check_dir).unwrap();

    let wall_a = loc_a
        .and_then(|(x, y)| move_coord(x, y, dir_a_dir, level.width, level.height))
        .map(|(x2, y2)| is_dungeon_wall(level, x2, y2))
        .unwrap_or(true);
    let wall_chk = loc_a
        .and_then(|(x, y)| move_coord(x, y, check_d, level.width, level.height))
        .map(|(x2, y2)| is_dungeon_wall(level, x2, y2))
        .unwrap_or(true);

    if !wall_a || !wall_chk {
        state.is_running = false;
    } else if state.cut_corners {
        let b = Direction::from_u8(dir_b).unwrap();
        state.direction = b;
        state.prev_dir = b;
    } else {
        let a = Direction::from_u8(dir_a).unwrap();
        let b = Direction::from_u8(dir_b).unwrap();
        state.direction = a;
        state.prev_dir = b;
    }

    state.is_running
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corridor_running_turns_corner() {
        let mut level = DungeonLevel::new(20, 20, 1, 50);
        for y in 0..20 {
            for x in 0..20 {
                level.set_tile(x, y, TileType::Wall);
            }
        }
        // Corridor: from (2, 2) south to (2, 5), then turns east to (6, 5)
        for y in 2..=5 {
            level.set_tile(2, y, TileType::Floor);
        }
        for x in 3..=6 {
            level.set_tile(x, 5, TileType::Floor);
        }

        // Test with cut_corners = false (deliberately go full corner)
        let mut run_no_cut = init_run(Direction::South, 2, 2, &level, false);
        run_no_cut.cut_corners = false;
        assert!(!run_no_cut.open_area);
        assert_eq!(run_no_cut.direction, Direction::South);

        let monsters = Vec::new();
        // Step to (2, 3)
        assert!(update_run_after_step(
            &mut run_no_cut,
            2,
            3,
            &level,
            &monsters,
            false
        ));
        assert_eq!(run_no_cut.direction, Direction::South);

        // Step to (2, 4) - without cutting corners, remains South
        assert!(update_run_after_step(
            &mut run_no_cut,
            2,
            4,
            &level,
            &monsters,
            false
        ));
        assert_eq!(run_no_cut.direction, Direction::South);

        // Step to corner (2, 5) - turns East
        assert!(update_run_after_step(
            &mut run_no_cut,
            2,
            5,
            &level,
            &monsters,
            false
        ));
        assert_eq!(run_no_cut.direction, Direction::East);

        // Test with default cut_corners = true (canonical umoria default)
        let mut run_cut = init_run(Direction::South, 2, 2, &level, false);
        assert!(run_cut.cut_corners);
        assert!(update_run_after_step(
            &mut run_cut,
            2,
            3,
            &level,
            &monsters,
            false
        ));
        assert_eq!(run_cut.direction, Direction::South);
        // At (2, 4), detects enclosed corner and cuts diagonally SouthEast
        assert!(update_run_after_step(
            &mut run_cut,
            2,
            4,
            &level,
            &monsters,
            false
        ));
        assert_eq!(run_cut.direction, Direction::SouthEast);
    }
}
