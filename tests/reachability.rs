use rmoria::dungeon::{DungeonLevel, TileType};
use std::collections::{HashSet, VecDeque};

#[test]
fn test_stairs_reachability_and_distribution() {
    let total_trials = 200;

    for depth in [1, 2, 5, 10] {
        let mut unreachable_count = 0;

        for _ in 0..total_trials {
            let mut level = DungeonLevel::new(66, 22, depth, 40);
            level.generate_simple_floor();

            let up_pos = level
                .tiles
                .iter()
                .enumerate()
                .find(|(_, t)| t.tile_type == TileType::StairsUp)
                .map(|(i, _)| (i % level.width, i / level.width))
                .expect("Level must have an up staircase");

            let down_positions: Vec<(usize, usize)> = level
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.tile_type == TileType::StairsDown)
                .map(|(i, _)| (i % level.width, i / level.width))
                .collect();

            assert!(
                down_positions.len() >= 3,
                "Expected 3-4 down staircases on depth {}, got {}",
                depth,
                down_positions.len()
            );

            // BFS from up_pos to find if ANY down staircase is reachable via normal passable tiles
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(up_pos);
            visited.insert(up_pos);

            let mut reached_any_down = false;
            while let Some((cx, cy)) = queue.pop_front() {
                if down_positions.contains(&(cx, cy)) {
                    reached_any_down = true;
                    break;
                }
                for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < level.width
                        && (ny as usize) < level.height
                    {
                        let np = (nx as usize, ny as usize);
                        if !visited.contains(&np)
                            && let Some(t) = level.get_tile(np.0, np.1)
                        {
                            let walkable = matches!(
                                t.tile_type,
                                TileType::Floor
                                    | TileType::DoorOpen
                                    | TileType::DoorClosed { .. }
                                    | TileType::StairsUp
                                    | TileType::StairsDown
                                    | TileType::Trap { .. }
                            );
                            if walkable {
                                visited.insert(np);
                                queue.push_back(np);
                            }
                        }
                    }
                }
            }

            if !reached_any_down {
                unreachable_count += 1;
            }
        }

        assert_eq!(
            unreachable_count, 0,
            "Found {} unreachable levels on depth {} out of {} trials!",
            unreachable_count, depth, total_trials
        );
    }
}
