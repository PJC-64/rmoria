pub mod dice;
pub mod dungeon;
pub mod entity;
pub mod flavor;
pub mod input;
pub mod magic;
pub mod player;
pub mod player_run;
pub mod rendering;
mod save;
pub mod scores;

pub use rendering::draw_map;

use crossterm::{
    cursor::{Hide, Show},
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock};

static GUI_ACTIVE: AtomicBool = AtomicBool::new(false);
static CHEAT_ACTIVE: AtomicBool = AtomicBool::new(false);
static GUI_INPUT_RX: OnceLock<Mutex<Receiver<crossterm::event::Event>>> = OnceLock::new();
static PEEKED_EVENT: Mutex<Option<crossterm::event::Event>> = Mutex::new(None);

#[derive(Clone)]
pub struct GuiState {
    pub screen: Vec<String>,
    pub status_msg: String,
    pub player_json: serde_json::Value,
}

static GUI_STATE_TX: OnceLock<Mutex<Sender<GuiState>>> = OnceLock::new();
static GUI_SETUP_RX: OnceLock<Mutex<Receiver<serde_json::Value>>> = OnceLock::new();
static GUI_SETUP_TX: OnceLock<Mutex<Sender<serde_json::Value>>> = OnceLock::new();

pub fn init_gui_mode() -> (
    Sender<crossterm::event::Event>,
    Receiver<GuiState>,
    Sender<serde_json::Value>,
) {
    GUI_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);

    let (input_tx, input_rx) = channel();
    let (state_tx, state_rx) = channel();
    let (setup_tx, setup_rx) = channel();

    let _ = GUI_INPUT_RX.set(Mutex::new(input_rx));
    let _ = GUI_STATE_TX.set(Mutex::new(state_tx));
    let _ = GUI_SETUP_RX.set(Mutex::new(setup_rx));
    let _ = GUI_SETUP_TX.set(Mutex::new(setup_tx.clone()));

    (input_tx, state_rx, setup_tx)
}

pub fn activate_cheat_global() {
    CHEAT_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn get_save_path() -> std::path::PathBuf {
    let mut path = if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        std::path::PathBuf::from(userprofile)
    } else {
        std::path::PathBuf::from(".")
    };

    path.push(".config");
    path.push("rmoria");

    // Ensure the directories exist
    let _ = std::fs::create_dir_all(&path);

    path.push("save.json");
    path
}

pub fn get_scores_path() -> std::path::PathBuf {
    let mut path = if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        std::path::PathBuf::from(userprofile)
    } else {
        std::path::PathBuf::from(".")
    };

    path.push(".config");
    path.push("rmoria");

    // Ensure the directories exist
    let _ = std::fs::create_dir_all(&path);

    path.push("scores.json");
    path
}

mod event {
    #[allow(unused_imports)]
    pub use crossterm::event::{Event, KeyCode, KeyModifiers};
    pub fn poll(dur: std::time::Duration) -> Result<bool, std::io::Error> {
        if crate::GUI_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
            if crate::PEEKED_EVENT.lock().unwrap().is_some() {
                return Ok(true);
            }
            if let Some(rx_mutex) = crate::GUI_INPUT_RX.get() {
                let rx = rx_mutex.lock().unwrap();
                if dur.as_millis() == 0 {
                    if let Ok(event) = rx.try_recv() {
                        *crate::PEEKED_EVENT.lock().unwrap() = Some(event);
                        return Ok(true);
                    }
                } else {
                    if let Ok(event) = rx.recv_timeout(dur) {
                        *crate::PEEKED_EVENT.lock().unwrap() = Some(event);
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            crossterm::event::poll(dur)
        }
    }

    pub fn read() -> Result<crossterm::event::Event, std::io::Error> {
        if crate::GUI_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(event) = crate::PEEKED_EVENT.lock().unwrap().take() {
                return Ok(event);
            }
            if let Some(rx_mutex) = crate::GUI_INPUT_RX.get() {
                let rx = rx_mutex.lock().unwrap();
                if let Ok(event) = rx.recv() {
                    return Ok(event);
                }
            }
            Err(std::io::Error::other("GUI Input channel disconnected"))
        } else {
            crossterm::event::read()
        }
    }
}
use dice::Dice;
use dungeon::{
    DungeonLevel, FloorItem, HaggleState, ShopType, TileType, generate_random_floor_item,
    handle_haggle_input, tile::TrapType,
};
use entity::monster::Monster;
use input::{Action, Direction, InputMapper, KeybindingMode};
use player::{Class, InscribeState, Item, ItemType, Player, Race};
use rand::Rng;
use save::GameState;
use scores::{HighScoreEntry, record_high_score};
use std::fs;
use std::io::{self};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMode {
    Dungeon,
    InventoryList,
    EquipmentList,
    WearMenu,
    TakeOffMenu,
    QuaffMenu,
    ReadMenu,
    Shop,
    ShopSellMenu,
    BrowseBookMenu,
    CastSpellMenu,
    PrayMenu,
    StudyMenu,
    SelectSpellDirection,
    CharacterStatsMenu,
    SelectDisarmDirection,
    BarterBuyMenu,
    BarterSellMenu,
    SelectOpenDirection,
    SelectCloseDirection,
    DropMenu,
    HelpMenu,
    AimWandMenu,
    SelectWandDirection,
    UseStaffMenu,
    SelectJamDirection,
    EatMenu,
    SelectRestCount,
    SelectBashDirection,
    SelectTunnelDirection,
    SelectLookDirection,
    InscribeMenu,
    Inscribing,
    IdentifyMenu,
    SelectThrowItem,
    SelectThrowDirection,
    GameOver,
    CombatLog,
    MessageHistory,
    IdentifySymbol,
    OptionsMenu,
    HighScores,
    ReducedMap,
    LocateMap,
    RoyalCoronation,
    Tombstone,
}

fn test_being_hit<R: rand::Rng>(
    base_to_hit: i32,
    level: i32,
    plus_to_hit: i32,
    armor_class: i32,
    class_adj: i32,
    rng: &mut R,
) -> bool {
    let hit_chance = base_to_hit + plus_to_hit * 3 + level * class_adj;
    let die = rng.gen_range(1..=20);
    die != 1 && (die == 20 || (hit_chance > 0 && rng.gen_range(1..=hit_chance) > armor_class))
}

fn trigger_trap_effect<R: rand::Rng>(
    trap_type: TrapType,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    rng: &mut R,
    tx: usize,
    ty: usize,
) -> String {
    let mut msg = String::new();
    match trap_type {
        TrapType::OpenPit => {
            msg.push_str("You fell into a pit!");
            if player.flags.free_fall {
                msg.push_str(" You gently float down.");
            } else {
                let dmg = Dice::new(2, 6).roll(rng) as i32;
                player.hp -= dmg;
                msg.push_str(&format!(" You take {} damage.", dmg));
            }
        }
        TrapType::ArrowPit => {
            let hit = test_being_hit(125, 0, 0, player.calculate_ac(), 0, rng);
            if hit {
                let dmg = Dice::new(1, 8).roll(rng) as i32;
                player.hp -= dmg;
                msg.push_str(&format!("An arrow hits you for {} damage!", dmg));
            } else {
                msg.push_str("An arrow barely misses you.");
            }
        }
        TrapType::CoveredPit => {
            msg.push_str("You fell into a covered pit.");
            if player.flags.free_fall {
                msg.push_str(" You gently float down.");
            } else {
                let dmg = Dice::new(2, 6).roll(rng) as i32;
                player.hp -= dmg;
                msg.push_str(&format!(" You take {} damage.", dmg));
            }
            if let Some(t) = level.get_tile_mut(tx, ty) {
                t.tile_type = TileType::Trap {
                    detected: true,
                    trap_type: TrapType::OpenPit,
                };
            }
        }
        TrapType::TrapDoor => {
            msg.push_str("You fell through a trap door!");
            if player.flags.free_fall {
                msg.push_str(" You gently float down.");
            } else {
                let dmg = Dice::new(2, 8).roll(rng) as i32;
                player.hp -= dmg;
                msg.push_str(&format!(" You take {} damage.", dmg));
            }
            level.depth += 1;
            level.generate_simple_floor();
            let (px, py) = level.find_random_floor_tile();
            player.x = px;
            player.y = py;
        }
        TrapType::SleepingGas => {
            msg.push_str("A strange white mist surrounds you!");
            if player.flags.free_action {
                msg.push_str(" You are unaffected.");
            } else {
                player.flags.paralysis += rng.gen_range(1..=10) + 4;
                msg.push_str(" You fall asleep.");
            }
        }
        TrapType::HiddenObject => {
            msg.push_str("Hmmm, there was something under this rock.");
            let item =
                crate::entity::treasure_data::generate_canonical_floor_item(level.depth, rng);
            level.items.push(FloorItem { x: tx, y: ty, item });
        }
        TrapType::DartOfStr => {
            let hit = test_being_hit(125, 0, 0, player.calculate_ac(), 0, rng);
            if hit {
                let dmg = Dice::new(1, 4).roll(rng) as i32;
                player.hp -= dmg;
                if player.drain_stat(0, rng) {
                    msg.push_str(&format!(
                        "A small dart weakens you! You take {} damage.",
                        dmg
                    ));
                } else {
                    msg.push_str(&format!("A small dart hits you for {} damage.", dmg));
                }
            } else {
                msg.push_str("A small dart barely misses you.");
            }
        }
        TrapType::Teleport => {
            msg.push_str("You hit a teleport trap!");
            let dest = level.find_random_floor_tile();
            player.move_to(dest.0, dest.1);
        }
        TrapType::Rockfall => {
            let dmg = Dice::new(2, 6).roll(rng) as i32;
            player.hp -= dmg;
            msg.push_str(&format!("You are hit by falling rock for {} damage.", dmg));
            if let Some(t) = level.get_tile_mut(tx, ty) {
                t.tile_type = TileType::Rubble;
            }
        }
        TrapType::CorrodingGas => {
            msg.push_str("A strange red gas surrounds you.");
            let dmg = Dice::new(1, 4).roll(rng) as i32;
            player.hp -= dmg;
            msg.push_str(&format!(" You take {} corrosion damage.", dmg));
        }
        TrapType::SummonMonster => {
            msg.push_str("You are enveloped in a strange rune! Monsters appear!");
            let num = 2 + rng.gen_range(1..=3);
            for _ in 0..num {
                let (sx, sy) = level.find_random_floor_tile_near(tx, ty, 3);
                let c_idx = crate::entity::monster_data::get_monster_for_level(level.depth, rng);
                monsters.push(Monster::from_creature_id(c_idx, sx, sy, false, rng));
            }
        }
        TrapType::FireTrap => {
            msg.push_str("You are enveloped in flames!");
            let mut dmg = Dice::new(4, 6).roll(rng) as i32;
            if player.flags.resistant_to_fire || player.flags.heat_resistance > 0 {
                dmg /= 3;
                msg.push_str(" Your fire resistance protects you!");
            }
            player.hp -= dmg;
            msg.push_str(&format!(" You take {} fire damage.", dmg));
        }
        TrapType::AcidTrap => {
            msg.push_str("You are splashed with acid!");
            let mut dmg = Dice::new(4, 6).roll(rng) as i32;
            if player.flags.resistant_to_acid {
                dmg /= 3;
                msg.push_str(" Your acid resistance protects you!");
            }
            player.hp -= dmg;
            msg.push_str(&format!(" You take {} acid damage.", dmg));
        }
        TrapType::PoisonGas => {
            msg.push_str("A pungent green gas surrounds you!");
            let dmg = Dice::new(2, 6).roll(rng) as i32;
            player.flags.poisoned += rng.gen_range(10..=30);
            player.hp -= dmg;
            msg.push_str(&format!(" You take {} poison damage.", dmg));
        }
        TrapType::BlindGas => {
            msg.push_str("A black gas surrounds you! You are blinded!");
            player.flags.blind += rng.gen_range(1..=50) + 50;
        }
        TrapType::ConfuseGas => {
            msg.push_str("A gas of scintillating colors surrounds you! You are confused!");
            player.flags.confused += rng.gen_range(1..=15) + 15;
        }
        TrapType::SlowDart => {
            let hit = test_being_hit(125, 0, 0, player.calculate_ac(), 0, rng);
            if hit {
                let dmg = Dice::new(1, 8).roll(rng) as i32;
                player.hp -= dmg;
                if player.flags.free_action {
                    msg.push_str(&format!(
                        "A small dart hits you for {} damage! You are unaffected.",
                        dmg
                    ));
                } else {
                    player.flags.slow += rng.gen_range(10..=30);
                    msg.push_str(&format!(
                        "A small dart hits you for {} damage! You feel slowed.",
                        dmg
                    ));
                }
            } else {
                msg.push_str("A small dart barely misses you.");
            }
        }
        TrapType::DartOfCon => {
            let hit = test_being_hit(125, 0, 0, player.calculate_ac(), 0, rng);
            if hit {
                let dmg = Dice::new(1, 8).roll(rng) as i32;
                player.hp -= dmg;
                if player.drain_stat(4, rng) {
                    msg.push_str(&format!(
                        "A small dart saps your health! You take {} damage.",
                        dmg
                    ));
                } else {
                    msg.push_str(&format!("A small dart hits you for {} damage.", dmg));
                }
            } else {
                msg.push_str("A small dart barely misses you.");
            }
        }
    }
    msg
}

#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
fn process_end_of_turn(
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    status_msg: &mut String,
    screen_mode: &mut ScreenMode,
    save_path: &std::path::Path,
    active_shop: ShopType,
    active_haggle: Option<&HaggleState>,
    is_resting: bool,
    turn: u64,
    max_depth: u32,
    level_cache: &mut std::collections::HashMap<u32, (DungeonLevel, Vec<Monster>)>,
) -> Result<(), std::io::Error> {
    let mut rng = rand::thread_rng();

    // Light source fuel consumption
    if let Some(light_item) = player
        .equipment
        .iter_mut()
        .find(|i| matches!(i.item_type, ItemType::Light { .. }))
        && let ItemType::Light { ref mut fuel } = light_item.item_type
        && *fuel > 0
    {
        *fuel -= 1;
        if *fuel == 50 {
            if light_item.name.contains("Torch") {
                status_msg.push_str(" Your torch is growing faint.");
            } else {
                status_msg.push_str(" Your light is growing faint.");
            }
        } else if *fuel == 0 {
            status_msg.push_str(" Your light has gone out!");
        }
    }

    // Condition ticks (poison, confusion, blindness, fear, speed, heroism, blessed, word of recall)
    let cond_result = player.tick_conditions(turn);
    if cond_result.died_from_poison {
        if save_path.exists() {
            let _ = std::fs::remove_file(save_path);
        }
        player.died_from = "poison".to_string();
        *screen_mode = if player.balrog_killed || player.total_winner {
            ScreenMode::RoyalCoronation
        } else {
            ScreenMode::Tombstone
        };
        *status_msg = format!("{} You have died from poison!", status_msg);
        draw_map(
            level,
            player,
            monsters,
            status_msg,
            *screen_mode,
            active_shop,
            active_haggle,
            None,
        )?;
        return Ok(());
    }
    for msg in cond_result.messages {
        status_msg.push(' ');
        status_msg.push_str(&msg);
    }
    if cond_result.recall_triggered {
        let new_depth = if level.depth > 0 {
            status_msg.push_str(" You feel yourself yanked upwards!");
            0
        } else {
            let target = player.max_depth_reached.max(1);
            status_msg.push_str(" You feel yourself yanked downwards!");
            target
        };

        if level.depth == 0 {
            level_cache.insert(0, (level.clone(), monsters.clone()));
        }
        level_cache.retain(|&k, _| k == 0);
        if let Some((cached_level, cached_monsters)) = level_cache.remove(&new_depth) {
            *level = cached_level;
            *monsters = cached_monsters;
        } else {
            let (w, h) = if new_depth == 0 {
                (dungeon::SCREEN_WIDTH, dungeon::SCREEN_HEIGHT)
            } else {
                (dungeon::MAX_WIDTH, dungeon::MAX_HEIGHT)
            };
            *level = DungeonLevel::new(w, h, new_depth, max_depth);
            level.generate_simple_floor();
            *monsters = level.generate_monsters(player.balrog_killed);
        }

        let (px, py) = level.find_random_floor_tile();
        player.move_to(px, py);
        level.update_fov(player);
    }

    // Food consumption, natural health/mana regeneration, and starvation
    let (hunger_msg, starved) = player.tick_digestion_and_regen(is_resting);
    if starved {
        if save_path.exists() {
            let _ = std::fs::remove_file(save_path);
        }
        player.died_from = "starvation".to_string();
        *screen_mode = if player.balrog_killed || player.total_winner {
            ScreenMode::RoyalCoronation
        } else {
            ScreenMode::Tombstone
        };
        *status_msg = format!("{} You have starved to death!", status_msg);
        draw_map(
            level,
            player,
            monsters,
            status_msg,
            *screen_mode,
            active_shop,
            active_haggle,
            None,
        )?;
        return Ok(());
    } else if let Some(msg) = hunger_msg
        && !status_msg.contains(msg.trim())
    {
        status_msg.push_str(msg);
    }

    // Track max depth visited
    player.max_depth_reached = player.max_depth_reached.max(level.depth);

    let player_speed_mod = player.speed_modifier();
    let mut new_monsters: Vec<Monster> = Vec::new();

    for m_idx in 0..monsters.len() {
        if monsters[m_idx].hp <= 0 {
            continue;
        }

        // 1. Wake up check
        if monsters[m_idx].asleep > 0 {
            let dist = (monsters[m_idx].x as isize - player.x as isize)
                .abs()
                .max((monsters[m_idx].y as isize - player.y as isize).abs());
            let aoe = monsters[m_idx].creature_def().area_affect_radius as isize;
            if monsters[m_idx].was_attacked {
                monsters[m_idx].asleep = 0;
            } else if dist <= aoe {
                monsters[m_idx].asleep = monsters[m_idx].asleep.saturating_sub(1);
            }
            if monsters[m_idx].asleep > 0 {
                continue;
            }
        }

        // 2. Stunned check
        if monsters[m_idx].stunned > 0 {
            monsters[m_idx].stunned -= 1;
            continue;
        }

        let moves = monsters[m_idx].movement_rate(player_speed_mod, turn, is_resting);
        for _ in 0..moves {
            if monsters[m_idx].hp <= 0 || player.hp <= 0 {
                break;
            }

            let dx = (player.x as isize - monsters[m_idx].x as isize).abs();
            let dy = (player.y as isize - monsters[m_idx].y as isize).abs();
            // Collect occupied positions dynamically so multi-step moves collide accurately
            let occupied_positions: Vec<(usize, usize)> = monsters
                .iter()
                .enumerate()
                .filter(|(idx, m)| *idx != m_idx && m.hp > 0)
                .map(|(_, m)| (m.x, m.y))
                .chain(new_monsters.iter().map(|m| (m.x, m.y)))
                .collect();

            // 3. Try casting spell or breath weapon
            let mut did_attack = crate::entity::monster::try_monster_cast_spell(
                &mut monsters[m_idx],
                player,
                level,
                &mut new_monsters,
                &occupied_positions,
                status_msg,
                &mut rng,
            );
            if player.hp <= 0 {
                player.hp = 0;
                if save_path.exists() {
                    let _ = std::fs::remove_file(save_path);
                }
                player.died_from = monsters[m_idx].name.clone();
                *screen_mode = if player.balrog_killed || player.total_winner {
                    ScreenMode::RoyalCoronation
                } else {
                    ScreenMode::Tombstone
                };
                status_msg.push_str(" You have died!");
                break;
            }

            // 4. Melee attack if adjacent and hasn't cast spell
            if !did_attack && dx <= 1 && dy <= 1 {
                crate::entity::monster::execute_monster_melee_attacks(
                    &monsters[m_idx],
                    player,
                    status_msg,
                    &mut rng,
                );
                player.searching = false;
                did_attack = true;

                if player.hp <= 0 {
                    player.hp = 0;
                    if save_path.exists() {
                        let _ = std::fs::remove_file(save_path);
                    }
                    player.died_from = monsters[m_idx].name.clone();
                    *screen_mode = if player.balrog_killed || player.total_winner {
                        ScreenMode::RoyalCoronation
                    } else {
                        ScreenMode::Tombstone
                    };
                    status_msg.push_str(" You have died!");
                    break;
                }
            }

            // 5. Monster reproduction check (e.g. lice, worms multiplying)
            if !did_attack
                && let Some(child) = crate::entity::monster::try_monster_multiply(
                    &monsters[m_idx],
                    level,
                    &occupied_positions,
                    player.x,
                    player.y,
                    &mut rng,
                )
            {
                new_monsters.push(child);
            }

            // 6. Movement AI
            if !did_attack
                && let Some((mx, my)) = monsters[m_idx].update_ai(
                    player.x,
                    player.y,
                    level,
                    level.depth,
                    player.killed_town_npcs,
                    player.town_npc_attack_threshold,
                )
            {
                let occupied_by_player = mx == player.x && my == player.y;
                let occupied_by_monster = occupied_positions
                    .iter()
                    .any(|&(ox, oy)| ox == mx && oy == my);

                if !occupied_by_player && !occupied_by_monster {
                    let is_door = if let Some(t) = level.get_tile(mx, my) {
                        matches!(t.tile_type, TileType::DoorClosed { .. })
                    } else {
                        false
                    };

                    if is_door {
                        let mut broke_spike = false;
                        let mut opened = false;
                        if let Some(t) = level.get_tile_mut(mx, my)
                            && let TileType::DoorClosed { ref mut spikes } = t.tile_type
                        {
                            if *spikes > 0 {
                                if rng.gen_bool(0.30) {
                                    *spikes -= 1;
                                    broke_spike = true;
                                }
                            } else {
                                t.tile_type = TileType::DoorOpen;
                                opened = true;
                            }
                        }

                        let dist = (((monsters[m_idx].x as isize - player.x as isize).pow(2)
                            + (monsters[m_idx].y as isize - player.y as isize).pow(2))
                            as f64)
                            .sqrt();
                        if dist <= 10.0 {
                            if broke_spike {
                                status_msg.push_str(" You hear a door spike snap!");
                            } else if opened {
                                status_msg.push_str(&format!(
                                    " The {} opens a door.",
                                    monsters[m_idx].name
                                ));
                            } else {
                                status_msg.push_str(" You hear a door being bashed!");
                            }
                        }

                        if opened {
                            monsters[m_idx].x = mx;
                            monsters[m_idx].y = my;
                        }
                    } else {
                        monsters[m_idx].x = mx;
                        monsters[m_idx].y = my;
                    }
                }
            }
        }

        if player.hp <= 0 {
            break;
        }
    }

    if !new_monsters.is_empty() {
        monsters.append(&mut new_monsters);
    }

    // Periodic town store maintenance every 1000 turns on town level (depth == 0)
    if turn > 0 && turn.is_multiple_of(1000) && level.depth == 0 {
        for shop in &mut level.shops {
            shop.maintain(turn, &mut rng);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
fn execute_player_run(
    direction: Direction,
    player: &mut Player,
    level: &mut DungeonLevel,
    monsters: &mut Vec<Monster>,
    status_msg: &mut String,
    screen_mode: &mut ScreenMode,
    save_path: &std::path::Path,
    active_shop: ShopType,
    active_haggle: Option<&HaggleState>,
    game_turn: &mut u64,
    max_depth: u32,
    level_cache: &mut std::collections::HashMap<u32, (DungeonLevel, Vec<Monster>)>,
) -> Result<bool, std::io::Error> {
    if player.flags.paralysis > 0 {
        *status_msg = "You are paralyzed!".to_string();
        return Ok(false);
    }
    if player.flags.confused > 0 {
        *status_msg = "You are confused!".to_string();
        return Ok(false);
    }
    if player.flags.blind > 0 {
        *status_msg = "You can't see to run!".to_string();
        return Ok(false);
    }

    let mut run_state = crate::player_run::init_run(direction, player.x, player.y, level, false);
    if !run_state.is_running {
        *status_msg = "You cannot run in that direction.".to_string();
        return Ok(false);
    }

    let mut ran_any_step = false;
    let mut rng = rand::thread_rng();

    while run_state.is_running && run_state.steps < 100 {
        let (dx, dy) = run_state.direction.to_dx_dy();
        let next_x = (player.x as isize + dx).clamp(0, level.width as isize - 1) as usize;
        let next_y = (player.y as isize + dy).clamp(0, level.height as isize - 1) as usize;

        if let Some(m_idx) = monsters
            .iter()
            .position(|m| m.x == next_x && m.y == next_y && m.hp > 0)
        {
            if player.flags.afraid > 0 {
                *status_msg = "You are too afraid!".to_string();
            } else {
                let damage =
                    player.roll_melee_damage_against(&mut rng, Some(monsters[m_idx].creature_id));
                if damage == 0 {
                    *status_msg = format!("You miss {}.", monsters[m_idx].name);
                } else {
                    *status_msg =
                        format!("You hit {} for {} damage!", monsters[m_idx].name, damage);
                }
                monsters[m_idx].was_attacked = true;
                let monster_name = monsters[m_idx].name.clone();
                let exp_reward = monsters[m_idx].experience_reward;
                let m_lvl = monsters[m_idx].level;
                let is_win = monsters[m_idx].is_win_creature() || monster_name == "The Balrog";

                if monsters[m_idx].take_damage(damage) {
                    if level.depth == 0 {
                        player.killed_town_npcs += 1;
                    }
                    status_msg.push_str(&format!(" You killed {}!", monster_name));
                    let old_exp = player.exp;
                    let old_lvl = player.level;
                    let leveled_up = player.gain_kill_experience(exp_reward, m_lvl);
                    let gained = player.exp.saturating_sub(old_exp);
                    if leveled_up || player.level > old_lvl {
                        status_msg.push_str(&format!(
                            " Congratulations! You reached level {}.",
                            player.level
                        ));
                    } else if gained > 0 {
                        status_msg.push_str(&format!(" Gained {} EXP.", gained));
                    }
                    if is_win {
                        player.balrog_killed = true;
                        player.total_winner = true;
                    }
                    monsters.remove(m_idx);
                }
            }
            run_state.is_running = false;
            ran_any_step = true;
        } else {
            let is_passable = level
                .get_tile(next_x, next_y)
                .map(|t| t.is_passable())
                .unwrap_or(false);
            if !is_passable {
                break;
            }

            let mut trigger_trap = None;
            if let Some(tile) = level.get_tile(next_x, next_y)
                && let TileType::Trap { trap_type, .. } = tile.tile_type
            {
                trigger_trap = Some(trap_type);
            }

            player.move_to(next_x, next_y);
            run_state.steps += 1;
            ran_any_step = true;

            if let Some(tt) = trigger_trap {
                let (px, py) = (player.x, player.y);
                let trap_msg = trigger_trap_effect(tt, player, level, monsters, &mut rng, px, py);
                *status_msg = trap_msg;
                if tt != TrapType::CoveredPit
                    && tt != TrapType::Rockfall
                    && let Some(t) = level.get_tile_mut(px, py)
                {
                    t.tile_type = TileType::Floor;
                }
                run_state.is_running = false;
            }

            let item_idx = level
                .items
                .iter()
                .position(|i| i.x == player.x && i.y == player.y);
            if let Some(idx) = item_idx {
                let is_gold = level.items[idx].item.name.contains("Gold Pile");
                if is_gold {
                    let floor_item = level.items.remove(idx);
                    let gold_amount = floor_item
                        .item
                        .name
                        .split('[')
                        .nth(1)
                        .and_then(|s| s.split(' ').next())
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(20);
                    player.gold += gold_amount;
                    *status_msg = format!("You picked up {} gold pieces.", gold_amount);
                } else if player.auto_pickup {
                    if !player.can_carry_item(&level.items[idx].item) {
                        *status_msg =
                            format!("You can't carry {}.", level.items[idx].item.display_name());
                        run_state.is_running = false;
                    } else {
                        let floor_item = level.items.remove(idx);
                        let mut item = floor_item.item;
                        if !item.identified {
                            player.flavors.assign_flavor(&mut item, &mut rng);
                            if player.flavors.is_identified(&item.name) {
                                item.identified = true;
                            }
                        }
                        let dname = item.display_name();
                        *status_msg = format!("You picked up {}.", dname);
                        player.add_item_to_inventory(item);
                        run_state.is_running = false;
                    }
                } else {
                    *status_msg = format!("You see {}.", level.items[idx].item.display_name());
                    run_state.is_running = false;
                }
            }
        }

        *game_turn += 1;
        process_end_of_turn(
            player,
            level,
            monsters,
            status_msg,
            screen_mode,
            save_path,
            active_shop,
            active_haggle,
            false,
            *game_turn,
            max_depth,
            level_cache,
        )?;

        if *screen_mode != ScreenMode::Dungeon {
            break;
        }

        if run_state.is_running {
            let still_running = crate::player_run::update_run_after_step(
                &mut run_state,
                player.x,
                player.y,
                level,
                monsters,
                player.flags.blind > 0,
            );
            if !still_running {
                run_state.is_running = false;
            }
        }

        draw_map(
            level,
            player,
            monsters,
            status_msg,
            *screen_mode,
            active_shop,
            active_haggle,
            None,
        )?;

        if event::poll(std::time::Duration::from_millis(0))? {
            let _ = event::read()?;
            *status_msg = "Run stopped by keypress.".to_string();
            break;
        }
    }

    Ok(ran_any_step)
}

#[allow(deprecated)]
pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = get_save_path();
    let mut rng = rand::thread_rng();

    let args: Vec<String> = std::env::args().collect();
    let is_gui = args.iter().any(|arg| arg == "--gui")
        || GUI_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);

    if is_gui {
        GUI_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
        let (input_tx, input_rx) = channel();
        let (state_tx, state_rx) = channel();
        let (setup_tx, setup_rx) = channel();

        let _ = GUI_INPUT_RX.set(Mutex::new(input_rx));
        let _ = GUI_STATE_TX.set(Mutex::new(state_tx));
        let _ = GUI_SETUP_RX.set(Mutex::new(setup_rx));
        let _ = GUI_SETUP_TX.set(Mutex::new(setup_tx));

        let save_path_clone = save_path.clone();
        std::thread::spawn(move || {
            let server = match std::net::TcpListener::bind("127.0.0.1:8080") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to bind WebSocket server on 127.0.0.1:8080: {}", e);
                    return;
                }
            };
            println!("Rmoria WebSocket Server running at ws://127.0.0.1:8080");

            for stream in server.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
                let mut ws = match tungstenite::accept(stream) {
                    Ok(w) => w,
                    Err(_) => continue,
                };

                let has_save = save_path_clone.exists();
                let init_msg = serde_json::json!({
                    "type": "init",
                    "has_save": has_save
                });
                let _ = ws.write_message(tungstenite::Message::Text(init_msg.to_string()));

                loop {
                    // Check state updates
                    while let Ok(gui_state) = state_rx.try_recv() {
                        let update_msg = serde_json::json!({
                            "type": "update",
                            "screen": gui_state.screen,
                            "status_msg": gui_state.status_msg,
                            "player": gui_state.player_json
                        });
                        if ws
                            .write_message(tungstenite::Message::Text(update_msg.to_string()))
                            .is_err()
                        {
                            break;
                        }
                    }

                    // Read incoming message
                    let msg = match ws.read_message() {
                        Ok(m) => m,
                        Err(e) => {
                            if e.to_string().contains("WouldBlock")
                                || e.to_string().contains("TimedOut")
                            {
                                continue;
                            }
                            break;
                        }
                    };

                    if let tungstenite::Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
                    {
                        if val["type"] == "load"
                            || val["type"] == "create_character"
                            || val["type"] == "roll_character"
                            || val["type"] == "accept_character"
                        {
                            if let Some(tx_mutex) = GUI_SETUP_TX.get() {
                                let tx = tx_mutex.lock().unwrap();
                                let _ = tx.send(val);
                            }
                        } else if val["type"] == "activate_cheat" {
                            activate_cheat_global();
                            let dummy =
                                crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                    crossterm::event::KeyCode::Char(' '),
                                    crossterm::event::KeyModifiers::empty(),
                                ));
                            let _ = input_tx.send(dummy);
                        } else if val["type"] == "key"
                            && let Some(key_str) = val["key"].as_str()
                        {
                            let event = match key_str {
                                "\x1b" => {
                                    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                        crossterm::event::KeyCode::Esc,
                                        crossterm::event::KeyModifiers::empty(),
                                    ))
                                }
                                "\n" => {
                                    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                        crossterm::event::KeyCode::Enter,
                                        crossterm::event::KeyModifiers::empty(),
                                    ))
                                }
                                "\u{0008}" => {
                                    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                        crossterm::event::KeyCode::Backspace,
                                        crossterm::event::KeyModifiers::empty(),
                                    ))
                                }
                                s if s.len() == 1 => {
                                    let c = s.chars().next().unwrap();
                                    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                        crossterm::event::KeyCode::Char(c),
                                        crossterm::event::KeyModifiers::empty(),
                                    ))
                                }
                                _ => continue,
                            };
                            let _ = input_tx.send(event);
                        }
                    }
                }
            }
        });
    }

    let mut stdout = io::stdout();

    if !is_gui {
        let (cols, rows) = crossterm::terminal::size()?;
        if cols < 80 || rows < 24 {
            return Err(format!(
                "Terminal size is too small (current: {}x{}). Please resize your terminal to at least 80x24.",
                cols, rows
            ).into());
        }
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
    }

    struct TerminalGuard {
        active: bool,
    }
    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            if self.active {
                let _ = disable_raw_mode();
                let mut stdout = io::stdout();
                let _ = execute!(stdout, Show, LeaveAlternateScreen);
            }
        }
    }
    let mut _terminal_guard = TerminalGuard { active: !is_gui };

    'outer_loop: loop {
        let mut restart_game = false;
        let (mut player, mut level, mut monsters, mut status_msg, max_depth, mut level_cache) =
            if is_gui {
                if let Some(rx_mutex) = GUI_SETUP_RX.get() {
                    let rx = rx_mutex.lock().unwrap();
                    let state_tx = crate::GUI_STATE_TX.get().unwrap().lock().unwrap().clone();
                    println!("Waiting for client initialization choice...");
                    let mut val = rx.recv()?;
                    loop {
                        if val["type"] == "load" {
                            let state = GameState::load_from_file(&save_path)?;
                            break (
                                state.player,
                                state.level,
                                state.monsters,
                                "Save game loaded successfully!".to_string(),
                                state.max_depth,
                                state.level_cache,
                            );
                        } else if val["type"] == "roll_character" {
                            let race_str = val["race"].as_str().unwrap_or("Human");
                            let class_str = val["class"].as_str().unwrap_or("Warrior");
                            let race = match race_str {
                                "Human" => Race::Human,
                                "HalfElf" => Race::HalfElf,
                                "Elf" => Race::Elf,
                                "Halfling" => Race::Halfling,
                                "Gnome" => Race::Gnome,
                                "Dwarf" => Race::Dwarf,
                                "HalfOrc" => Race::HalfOrc,
                                "HalfTroll" => Race::HalfTroll,
                                _ => Race::Human,
                            };
                            let class = match class_str {
                                "Warrior" => Class::Warrior,
                                "Mage" => Class::Mage,
                                "Priest" => Class::Priest,
                                "Rogue" => Class::Rogue,
                                "Ranger" => Class::Ranger,
                                "Paladin" => Class::Paladin,
                                _ => Class::Warrior,
                            };

                            let s_str = rng.gen_range(8..=18);
                            let s_int = rng.gen_range(8..=18);
                            let s_wis = rng.gen_range(8..=18);
                            let s_dex = rng.gen_range(8..=18);
                            let s_con = rng.gen_range(8..=18);
                            let s_chr = rng.gen_range(8..=18);
                            let stats = player::Attributes::new(
                                s_str as i16,
                                s_int as i16,
                                s_wis as i16,
                                s_dex as i16,
                                s_con as i16,
                                s_chr as i16,
                            );

                            let mut temp_player = Player::new("Candidate", race, class, 30, 10);
                            temp_player.stats = stats;
                            temp_player.apply_race_and_class_modifiers();
                            temp_player.update_max_hp_and_mana();

                            let history = player::generate_history(race);

                            let roll_result = serde_json::json!({
                                "stats": {
                                    "str": temp_player.stats.strength,
                                    "int": temp_player.stats.intelligence,
                                    "wis": temp_player.stats.wisdom,
                                    "dex": temp_player.stats.dexterity,
                                    "con": temp_player.stats.constitution,
                                    "chr": temp_player.stats.charisma,
                                    "max_hp": temp_player.max_hp,
                                    "max_mana": temp_player.max_mana
                                },
                                "history": history
                            });

                            let state_msg = GuiState {
                                screen: vec![],
                                status_msg: "rolled_character".to_string(),
                                player_json: roll_result,
                            };
                            let _ = state_tx.send(state_msg);

                            // Wait for next client message
                            val = rx.recv()?;
                        } else if val["type"] == "accept_character" {
                            let name = val["name"].as_str().unwrap_or("Warrior").to_string();
                            let race_str = val["race"].as_str().unwrap_or("Human");
                            let class_str = val["class"].as_str().unwrap_or("Warrior");
                            let race = match race_str {
                                "Human" => Race::Human,
                                "HalfElf" => Race::HalfElf,
                                "Elf" => Race::Elf,
                                "Halfling" => Race::Halfling,
                                "Gnome" => Race::Gnome,
                                "Dwarf" => Race::Dwarf,
                                "HalfOrc" => Race::HalfOrc,
                                "HalfTroll" => Race::HalfTroll,
                                _ => Race::Human,
                            };
                            let class = match class_str {
                                "Warrior" => Class::Warrior,
                                "Mage" => Class::Mage,
                                "Priest" => Class::Priest,
                                "Rogue" => Class::Rogue,
                                "Ranger" => Class::Ranger,
                                "Paladin" => Class::Paladin,
                                _ => Class::Warrior,
                            };

                            let stats_val = &val["stats"];
                            let s_str = stats_val["str"].as_i64().unwrap_or(10) as i16;
                            let s_int = stats_val["int"].as_i64().unwrap_or(10) as i16;
                            let s_wis = stats_val["wis"].as_i64().unwrap_or(10) as i16;
                            let s_dex = stats_val["dex"].as_i64().unwrap_or(10) as i16;
                            let s_con = stats_val["con"].as_i64().unwrap_or(10) as i16;
                            let s_chr = stats_val["chr"].as_i64().unwrap_or(10) as i16;
                            let stats =
                                player::Attributes::new(s_str, s_int, s_wis, s_dex, s_con, s_chr);

                            let mut final_player = Player::new(&name, race, class, 30, 10);
                            final_player.stats = stats;

                            let history = val["history"].as_str().unwrap_or("").to_string();
                            final_player.history = history;
                            final_player.update_max_hp_and_mana();
                            final_player.hp = final_player.max_hp;
                            final_player.mana = final_player.max_mana;

                            let rolled_max = rng.gen_range(8..=15);
                            let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                            lvl.generate_simple_floor();

                            let start_pos = lvl.find_random_floor_tile();
                            final_player.move_to(start_pos.0, start_pos.1);
                            let mons = lvl.generate_monsters(false);

                            break (
                                final_player,
                                lvl,
                                mons,
                                "Welcome to rmoria! Find town shops or descend stairs (>)"
                                    .to_string(),
                                rolled_max,
                                std::collections::HashMap::new(),
                            );
                        } else {
                            val = rx.recv()?;
                        }
                    }
                } else {
                    return Err("GUI Setup channel not initialized".into());
                }
            } else if save_path.exists() {
                match GameState::load_from_file(&save_path) {
                    Ok(state) => (
                        state.player,
                        state.level,
                        state.monsters,
                        "Save game loaded successfully!".to_string(),
                        state.max_depth,
                        state.level_cache,
                    ),
                    Err(e) => {
                        let rolled_max = rng.gen_range(8..=15);
                        let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                        lvl.generate_simple_floor();
                        let p = match player::run_character_creation(&mut stdout, &mut rng, None) {
                            Ok(p) => p,
                            Err(e) if e.to_string().contains("cancelled") => {
                                return Ok(());
                            }
                            Err(e) => return Err(e),
                        };
                        if !is_gui {
                            let _ = execute!(
                                stdout,
                                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                            );
                        }
                        let start_pos = lvl.find_random_floor_tile();
                        let mut player = p;
                        player.move_to(start_pos.0, start_pos.1);
                        let mons = lvl.generate_monsters(false);
                        (
                            player,
                            lvl,
                            mons,
                            format!("Failed to load save: {}. Started new game.", e),
                            rolled_max,
                            std::collections::HashMap::new(),
                        )
                    }
                }
            } else {
                let rolled_max = rng.gen_range(8..=15);
                let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                lvl.generate_simple_floor();
                let p = match player::run_character_creation(&mut stdout, &mut rng, None) {
                    Ok(p) => p,
                    Err(e) if e.to_string().contains("cancelled") => {
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                };
                if !is_gui {
                    let _ = execute!(
                        stdout,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    );
                }
                let start_pos = lvl.find_random_floor_tile();
                let mut player = p;
                player.move_to(start_pos.0, start_pos.1);
                let mons = lvl.generate_monsters(false);
                (
                    player,
                    lvl,
                    mons,
                    "Welcome to rmoria! Find town shops or descend stairs (>)".to_string(),
                    rolled_max,
                    std::collections::HashMap::new(),
                )
            };

        let mut screen_mode = ScreenMode::Dungeon;
        let mut active_shop = ShopType::General;
        let mut selected_spell_idx: usize = 0;
        let mut selected_spell_is_prayer: bool = false;
        let mut selected_wand_inv_idx: usize = 0;
        let mut selected_throw_inv_idx: usize = 0;
        let mut active_haggle: Option<HaggleState> = None;
        let mut active_inscribe: Option<InscribeState> = None;
        let mut resting_turns: Option<i32> = None;
        let mut resting_until_healed = false;
        let mut rest_input_buffer = String::new();
        let mut game_turn: u64 = 0;

        draw_map(
            &mut level,
            &player,
            &monsters,
            &status_msg,
            screen_mode,
            active_shop,
            active_haggle.as_ref(),
            active_inscribe.as_ref(),
        )?;

        let mut mapper = InputMapper::new(player.keybinding_mode);

        'game_loop: loop {
            if CHEAT_ACTIVE.swap(false, std::sync::atomic::Ordering::Relaxed) {
                player.is_wizard = true;
                player.max_hp = 9999;
                player.hp = 9999;
                player.max_mana = 9999;
                player.mana = 9999;
                player.food = 15000;
                player.level = 50;
                player.stats.strength = 99;
                player.stats.intelligence = 99;
                player.stats.wisdom = 99;
                player.stats.dexterity = 99;
                player.stats.constitution = 99;
                player.stats.charisma = 99;

                let books = vec![
                    crate::magic::create_book(0, false),
                    crate::magic::create_book(1, false),
                    crate::magic::create_book(2, false),
                    crate::magic::create_book(3, false),
                    crate::magic::create_book(0, true),
                    crate::magic::create_book(1, true),
                    crate::magic::create_book(2, true),
                    crate::magic::create_book(3, true),
                ];
                for book in books {
                    if !player.inventory.iter().any(|i| i.name == book.name) {
                        player.inventory.push(book);
                    }
                }
                player.spells_learnt = 0x7FFFFFFF;
                player.update_max_hp_and_mana();
                player.mana = player.max_mana;

                status_msg = "TEST MODE ENABLED: ALL SPELLBOOKS & IMMORTALITY GRANTED!".to_string();
                let _ = execute!(
                    io::stdout(),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                );
                draw_map(
                    &mut level,
                    &player,
                    &monsters,
                    &status_msg,
                    screen_mode,
                    active_shop,
                    active_haggle.as_ref(),
                    active_inscribe.as_ref(),
                )?;
            }

            if player.is_wizard {
                player.hp = player.max_hp;
                player.mana = player.max_mana;
                player.food = 15000;
            }

            if screen_mode != ScreenMode::GameOver
                && screen_mode != ScreenMode::Tombstone
                && screen_mode != ScreenMode::RoyalCoronation
                && player.flags.paralysis > 0
            {
                resting_turns = None;
                resting_until_healed = false;
                status_msg = "You are paralyzed!".to_string();
                game_turn += 1;
                process_end_of_turn(
                    &mut player,
                    &mut level,
                    &mut monsters,
                    &mut status_msg,
                    &mut screen_mode,
                    &save_path,
                    active_shop,
                    active_haggle.as_ref(),
                    false,
                    game_turn,
                    max_depth,
                    &mut level_cache,
                )?;
                draw_map(
                    &mut level,
                    &player,
                    &monsters,
                    &status_msg,
                    screen_mode,
                    active_shop,
                    active_haggle.as_ref(),
                    active_inscribe.as_ref(),
                )?;
                std::thread::sleep(std::time::Duration::from_millis(150));
                continue 'game_loop;
            }

            let is_resting = resting_turns.is_some() || resting_until_healed;
            let poll_dur = if is_resting {
                std::time::Duration::from_millis(0)
            } else {
                std::time::Duration::from_millis(100)
            };

            if event::poll(poll_dur)? {
                match event::read()? {
                    Event::Resize(_, _) => {
                        let _ = execute!(
                            io::stdout(),
                            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                        );
                        draw_map(
                            &mut level,
                            &player,
                            &monsters,
                            &status_msg,
                            screen_mode,
                            active_shop,
                            active_haggle.as_ref(),
                            active_inscribe.as_ref(),
                        )?;
                    }
                    Event::Key(key_event) => {
                        if is_resting {
                            resting_turns = None;
                            resting_until_healed = false;
                            status_msg = "Rest interrupted by keypress.".to_string();
                            draw_map(
                                &mut level,
                                &player,
                                &monsters,
                                &status_msg,
                                screen_mode,
                                active_shop,
                                active_haggle.as_ref(),
                                active_inscribe.as_ref(),
                            )?;
                            continue 'game_loop;
                        }

                        if key_event.code == KeyCode::Char('c')
                            && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            break 'game_loop;
                        }

                        let mut player_acted = false;

                        if screen_mode == ScreenMode::SelectRestCount {
                            match key_event.code {
                                KeyCode::Char(c) => {
                                    if c.is_ascii_digit() || c == '*' {
                                        rest_input_buffer.push(c);
                                        status_msg = format!(
                                            "Rest how long? (number of turns, or * for until healed): {}",
                                            rest_input_buffer
                                        );
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                    }
                                }
                                KeyCode::Backspace => {
                                    rest_input_buffer.pop();
                                    status_msg = format!(
                                        "Rest how long? (number of turns, or * for until healed): {}",
                                        rest_input_buffer
                                    );
                                    draw_map(
                                        &mut level,
                                        &player,
                                        &monsters,
                                        &status_msg,
                                        screen_mode,
                                        active_shop,
                                        active_haggle.as_ref(),
                                        active_inscribe.as_ref(),
                                    )?;
                                }
                                KeyCode::Enter => {
                                    if rest_input_buffer == "*" {
                                        resting_until_healed = true;
                                        resting_turns = None;
                                        status_msg = "Resting...".to_string();
                                        screen_mode = ScreenMode::Dungeon;
                                    } else if let Ok(n) = rest_input_buffer.parse::<i32>()
                                        && n > 0
                                    {
                                        resting_turns = Some(n);
                                        resting_until_healed = false;
                                        status_msg = "Resting...".to_string();
                                        screen_mode = ScreenMode::Dungeon;
                                    }
                                }
                                KeyCode::Esc => {
                                    status_msg = "Cancelled.".to_string();
                                    screen_mode = ScreenMode::Dungeon;
                                    draw_map(
                                        &mut level,
                                        &player,
                                        &monsters,
                                        &status_msg,
                                        screen_mode,
                                        active_shop,
                                        active_haggle.as_ref(),
                                        active_inscribe.as_ref(),
                                    )?;
                                }
                                _ => {}
                            }
                        } else if screen_mode == ScreenMode::Inscribing {
                            if let Some(ref mut state) = active_inscribe {
                                match key_event.code {
                                    KeyCode::Char(c) => {
                                        if state.input.len() < 30 {
                                            state.input.push(c);
                                            status_msg = format!(
                                                "Inscribing... type characters and press ENTER: {}",
                                                state.input
                                            );
                                            draw_map(
                                                &mut level,
                                                &player,
                                                &monsters,
                                                &status_msg,
                                                screen_mode,
                                                active_shop,
                                                active_haggle.as_ref(),
                                                active_inscribe.as_ref(),
                                            )?;
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        state.input.pop();
                                        status_msg = format!(
                                            "Inscribing... type characters and press ENTER: {}",
                                            state.input
                                        );
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                    }
                                    KeyCode::Enter => {
                                        let item_idx = state.item_index;
                                        let is_eq = state.is_equipment;
                                        let ins_text = state.input.trim().to_string();
                                        let ins_val = if ins_text.is_empty() {
                                            None
                                        } else {
                                            Some(ins_text)
                                        };

                                        if is_eq {
                                            if item_idx < player.equipment.len() {
                                                player.equipment[item_idx].inscription =
                                                    ins_val.clone();
                                            }
                                        } else {
                                            if item_idx < player.inventory.len() {
                                                player.inventory[item_idx].inscription =
                                                    ins_val.clone();
                                            }
                                        }

                                        status_msg = if let Some(txt) = ins_val {
                                            format!("Inscribed item with: {}", txt)
                                        } else {
                                            "Cleared inscription.".to_string()
                                        };

                                        active_inscribe = None;
                                        screen_mode = ScreenMode::Dungeon;
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                    }
                                    KeyCode::Esc => {
                                        status_msg = "Cancelled.".to_string();
                                        active_inscribe = None;
                                        screen_mode = ScreenMode::Dungeon;
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                    }
                                    _ => {}
                                }
                            }
                        } else if screen_mode == ScreenMode::Dungeon {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                match key_event.code {
                                    KeyCode::Char('p') => {
                                        screen_mode = ScreenMode::MessageHistory;
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('w') => {
                                        player.is_wizard = !player.is_wizard;
                                        status_msg = if player.is_wizard {
                                            "Wizard mode enabled!".to_string()
                                        } else {
                                            "Wizard mode disabled.".to_string()
                                        };
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('x') => {
                                        if player.is_wizard {
                                            status_msg = "TEST MODE ACTIVE: Character cannot be saved. Quitting...".to_string();
                                            draw_map(
                                                &mut level,
                                                &player,
                                                &monsters,
                                                &status_msg,
                                                screen_mode,
                                                active_shop,
                                                active_haggle.as_ref(),
                                                active_inscribe.as_ref(),
                                            )?;
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                1500,
                                            ));
                                            break 'game_loop;
                                        }
                                        status_msg = "Saving game and quitting...".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;

                                        level_cache.retain(|&k, _| k == 0);
                                        let state = GameState::new(
                                            player.clone(),
                                            level.clone(),
                                            monsters.clone(),
                                            max_depth,
                                            level_cache.clone(),
                                        );
                                        match state.save_to_file(&save_path) {
                                            Ok(()) => {
                                                std::thread::sleep(std::time::Duration::from_millis(
                                                    800,
                                                ));
                                                break 'game_loop;
                                            }
                                            Err(e) => {
                                                status_msg = format!("Save failed: {}. Game NOT exited.", e);
                                                draw_map(
                                                    &mut level,
                                                    &player,
                                                    &monsters,
                                                    &status_msg,
                                                    screen_mode,
                                                    active_shop,
                                                    active_haggle.as_ref(),
                                                    active_inscribe.as_ref(),
                                                )?;
                                                std::thread::sleep(std::time::Duration::from_secs(2));
                                            }
                                        }
                                    }
                                    KeyCode::Char('a') if player.is_wizard => {
                                        player.flags.poisoned = 0;
                                        player.flags.blind = 0;
                                        player.flags.confused = 0;
                                        player.flags.afraid = 0;
                                        player.flags.paralysis = 0;
                                        player.flags.image = 0;
                                        player.food = 15000;
                                        player.hp = player.max_hp;
                                        player.mana = player.max_mana;
                                        status_msg = "Wizard: Cured all afflictions and restored HP/Mana/Food.".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('e') if player.is_wizard => {
                                        player.stats.strength = 118;
                                        player.stats.intelligence = 118;
                                        player.stats.wisdom = 118;
                                        player.stats.dexterity = 118;
                                        player.stats.constitution = 118;
                                        player.stats.charisma = 118;
                                        player.max_hp = 999;
                                        player.hp = 999;
                                        player.max_mana = 999;
                                        player.mana = 999;
                                        player.gold += 50000;
                                        status_msg = "Wizard: Stats set to 18/100, HP/Mana to 999, +50,000 gold.".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('f') if player.is_wizard => {
                                        let count = monsters
                                            .iter()
                                            .filter(|m| m.name != "The Balrog")
                                            .count();
                                        monsters.retain(|m| m.name == "The Balrog");
                                        status_msg = format!(
                                            "Wizard: Mass genocide eliminated {} monsters!",
                                            count
                                        );
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('g') if player.is_wizard => {
                                        for _ in 0..5 {
                                            let item = generate_random_floor_item(
                                                level.depth.max(30),
                                                &mut rng,
                                            );
                                            level.items.push(FloorItem {
                                                x: player.x,
                                                y: player.y,
                                                item,
                                            });
                                        }
                                        status_msg =
                                            "Wizard: Generated 5 magical items at player tile."
                                                .to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('d') if player.is_wizard => {
                                        let new_depth = (level.depth + 5).min(50);
                                        level = DungeonLevel::new(
                                            level.width,
                                            level.height,
                                            new_depth,
                                            max_depth,
                                        );
                                        level.generate_simple_floor();
                                        monsters = level.generate_monsters(player.balrog_killed);
                                        let (px, py) = level.find_random_floor_tile();
                                        player.move_to(px, py);
                                        level.update_fov(&player);
                                        status_msg = format!(
                                            "Wizard: Jumped to depth {} ({} ft).",
                                            new_depth,
                                            new_depth * 50
                                        );
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('i') if player.is_wizard => {
                                        for item in player
                                            .inventory
                                            .iter_mut()
                                            .chain(player.equipment.iter_mut())
                                        {
                                            item.identified = true;
                                            player.flavors.identify(&item.name);
                                        }
                                        status_msg =
                                            "Wizard: Identified all inventory and equipment items."
                                                .to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('t') if player.is_wizard => {
                                        let (nx, ny) = level.find_random_floor_tile();
                                        player.move_to(nx, ny);
                                        level.update_fov(&player);
                                        status_msg =
                                            "Wizard: Teleported to a random location.".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    _ => {}
                                }
                            }

                            if player.is_wizard {
                                match key_event.code {
                                    KeyCode::Char('*') => {
                                        for tile in level.tiles.iter_mut() {
                                            tile.is_lit = true;
                                            tile.visible = true;
                                            tile.remembered = true;
                                        }
                                        status_msg =
                                            "Wizard: Illuminated entire dungeon level.".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char(':') => {
                                        let (top, bottom, left, right) =
                                            level.get_viewport_bounds(player.x, player.y);
                                        for y in top..bottom {
                                            for x in left..right {
                                                if let Some(tile) = level.get_tile_mut(x, y) {
                                                    tile.remembered = true;
                                                }
                                            }
                                        }
                                        status_msg = "Wizard: Mapped current panel.".to_string();
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('+') => {
                                        player.add_experience(10000);
                                        status_msg = format!(
                                            "Wizard: Gained 10,000 EXP! (Total: {})",
                                            player.exp
                                        );
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    KeyCode::Char('&') => {
                                        let cid = rng.gen_range(
                                            1..crate::entity::monster_data::CREATURES_LIST.len(),
                                        );
                                        let mut summoned = Monster::from_creature_id(
                                            cid, player.x, player.y, false, &mut rng,
                                        );
                                        for dy in -1..=1 {
                                            for dx in -1..=1 {
                                                let tx = (player.x as isize + dx)
                                                    .clamp(0, level.width as isize - 1)
                                                    as usize;
                                                let ty = (player.y as isize + dy)
                                                    .clamp(0, level.height as isize - 1)
                                                    as usize;
                                                if level
                                                    .get_tile(tx, ty)
                                                    .map(|t| t.is_passable())
                                                    .unwrap_or(false)
                                                    && !monsters
                                                        .iter()
                                                        .any(|m| m.x == tx && m.y == ty)
                                                {
                                                    summoned.x = tx;
                                                    summoned.y = ty;
                                                    break;
                                                }
                                            }
                                        }
                                        status_msg = format!("Wizard: Summoned {}!", summoned.name);
                                        monsters.push(summoned);
                                        draw_map(
                                            &mut level,
                                            &player,
                                            &monsters,
                                            &status_msg,
                                            screen_mode,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            active_inscribe.as_ref(),
                                        )?;
                                        continue 'game_loop;
                                    }
                                    _ => {}
                                }
                            }

                            if key_event.code == KeyCode::Char('=') {
                                screen_mode = ScreenMode::OptionsMenu;
                                draw_map(
                                    &mut level,
                                    &player,
                                    &monsters,
                                    &status_msg,
                                    screen_mode,
                                    active_shop,
                                    active_haggle.as_ref(),
                                    active_inscribe.as_ref(),
                                )?;
                                continue 'game_loop;
                            }
                            if key_event.code == KeyCode::Char('/') {
                                player.identified_symbol = None;
                                screen_mode = ScreenMode::IdentifySymbol;
                                draw_map(
                                    &mut level,
                                    &player,
                                    &monsters,
                                    &status_msg,
                                    screen_mode,
                                    active_shop,
                                    active_haggle.as_ref(),
                                    active_inscribe.as_ref(),
                                )?;
                                continue 'game_loop;
                            }
                            if key_event.code == KeyCode::Char('V') {
                                player.high_score_page = 0;
                                screen_mode = ScreenMode::HighScores;
                                draw_map(
                                    &mut level,
                                    &player,
                                    &monsters,
                                    &status_msg,
                                    screen_mode,
                                    active_shop,
                                    active_haggle.as_ref(),
                                    active_inscribe.as_ref(),
                                )?;
                                continue 'game_loop;
                            }
                            if let KeyCode::Char(c) = key_event.code {
                                let action = mapper.map_key(c);
                                match action {
                                    Action::Move(mut direction)
                                    | Action::MoveWithoutPickup(mut direction) => {
                                        if player.flags.confused > 0
                                            && direction != Direction::Rest
                                            && rng.gen_bool(0.75)
                                        {
                                            let random_dirs = [
                                                Direction::NorthWest,
                                                Direction::North,
                                                Direction::NorthEast,
                                                Direction::West,
                                                Direction::East,
                                                Direction::SouthWest,
                                                Direction::South,
                                                Direction::SouthEast,
                                            ];
                                            direction =
                                                random_dirs[rng.gen_range(0..random_dirs.len())];
                                        }

                                        let (dx, dy) = match direction {
                                            Direction::NorthWest => (-1, -1),
                                            Direction::North => (0, -1),
                                            Direction::NorthEast => (1, -1),
                                            Direction::West => (-1, 0),
                                            Direction::Rest => (0, 0),
                                            Direction::East => (1, 0),
                                            Direction::SouthWest => (-1, 1),
                                            Direction::South => (0, 1),
                                            Direction::SouthEast => (1, 1),
                                        };

                                        let next_x = (player.x as isize + dx) as usize;
                                        let next_y = (player.y as isize + dy) as usize;

                                        let mut entered_shop = false;
                                        if level.depth == 0
                                            && let Some(shop_info) = level
                                                .shops
                                                .iter()
                                                .find(|s| s.door_x == next_x && s.door_y == next_y)
                                        {
                                            if shop_info.is_closed(game_turn) {
                                                status_msg = "The door is locked! The shopkeeper had enough of your insults. Come back later.".to_string();
                                            } else {
                                                screen_mode = ScreenMode::Shop;
                                                active_shop = shop_info.shop_type;
                                                status_msg = format!(
                                                    "Welcome to {}!",
                                                    shop_info.owner().name.trim()
                                                );
                                                entered_shop = true;
                                            }
                                        }

                                        if entered_shop {
                                            // Shop entered
                                        } else if let Some(m_idx) = monsters
                                            .iter()
                                            .position(|m| m.x == next_x && m.y == next_y)
                                        {
                                            let is_m_visible = level
                                                .get_tile(next_x, next_y)
                                                .map(|t| t.visible)
                                                .unwrap_or(false);

                                            if is_m_visible {
                                                if player.flags.afraid > 0 {
                                                    status_msg = "You are too afraid!".to_string();
                                                    player_acted = true;
                                                } else {
                                                    let damage = player.roll_melee_damage_against(
                                                        &mut rng,
                                                        Some(monsters[m_idx].creature_id),
                                                    );
                                                    if damage == 0 {
                                                        status_msg = format!(
                                                            "You miss {}.",
                                                            monsters[m_idx].name
                                                        );
                                                    } else {
                                                        status_msg = format!(
                                                            "You hit {} for {} damage!",
                                                            monsters[m_idx].name, damage
                                                        );
                                                    }

                                                    monsters[m_idx].was_attacked = true;
                                                    let monster_name = monsters[m_idx].name.clone();
                                                    let exp_reward =
                                                        monsters[m_idx].experience_reward;
                                                    let m_lvl = monsters[m_idx].level;
                                                    let is_win = monsters[m_idx].is_win_creature()
                                                        || monster_name == "The Balrog";

                                                    if monsters[m_idx].take_damage(damage) {
                                                        if level.depth == 0 {
                                                            player.killed_town_npcs += 1;
                                                        }
                                                        status_msg.push_str(&format!(
                                                            " You killed {}!",
                                                            monster_name
                                                        ));

                                                        let old_exp = player.exp;
                                                        let old_lvl = player.level;
                                                        let leveled_up = player
                                                            .gain_kill_experience(
                                                                exp_reward, m_lvl,
                                                            );
                                                        let gained =
                                                            player.exp.saturating_sub(old_exp);
                                                        if leveled_up || player.level > old_lvl {
                                                            status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                                        } else if gained > 0 {
                                                            status_msg.push_str(&format!(
                                                                " Gained {} EXP.",
                                                                gained
                                                            ));
                                                        } else if exp_reward > 0 {
                                                            status_msg.push_str(
                                                                " Gained fractional EXP.",
                                                            );
                                                        } else {
                                                            status_msg.push_str(
                                                                " Gained 0 EXP (Town monster).",
                                                            );
                                                        }

                                                        if is_win {
                                                            player.balrog_killed = true;
                                                            player.total_winner = true;
                                                        }

                                                        let is_boss = monster_name == "The Balrog"
                                                            || monster_name.contains("Lich");
                                                        if is_boss || rng.gen_bool(0.30) {
                                                            let drop_item =
                                                                if monster_name == "The Balrog" {
                                                                    Item::new(
                                                                        "Broadsword of Slaying",
                                                                        1,
                                                                        150,
                                                                        ItemType::Weapon {
                                                                            damage: Dice::new(3, 6),
                                                                        },
                                                                    )
                                                                } else {
                                                                    generate_random_floor_item(
                                                                        level.depth,
                                                                        &mut rng,
                                                                    )
                                                                };
                                                            level.items.push(FloorItem {
                                                                x: next_x,
                                                                y: next_y,
                                                                item: drop_item.clone(),
                                                            });
                                                            status_msg.push_str(&format!(
                                                                " It dropped a {}!",
                                                                drop_item.name
                                                            ));
                                                        }

                                                        if monster_name == "The Balrog" {
                                                            player.balrog_killed = true;
                                                            player.total_winner = true;
                                                            status_msg.push_str(" CONGRATULATIONS! YOU HAVE SLAIN THE BALROG! You are a Total Winner! Use 'Q' when you wish to retire.");
                                                        }
                                                        monsters.remove(m_idx);
                                                    }
                                                    player_acted = true;
                                                }
                                            } else {
                                                if let Some(tile) = level.get_tile(next_x, next_y)
                                                    && tile.is_passable()
                                                {
                                                    player.move_to(next_x, next_y);
                                                    status_msg = "You step into the darkness and bump into a monster!".to_string();
                                                    player_acted = true;
                                                }
                                            }
                                        } else {
                                            if let Some(tile) = level.get_tile(next_x, next_y) {
                                                if tile.is_passable()
                                                    || direction == Direction::Rest
                                                {
                                                    let mut trigger_trap = None;
                                                    if let TileType::Trap { trap_type, .. } =
                                                        tile.tile_type
                                                    {
                                                        trigger_trap = Some(trap_type);
                                                    }

                                                    player.move_to(next_x, next_y);

                                                    if direction == Direction::Rest {
                                                        if player.food < 0 {
                                                            status_msg =
                                                                "You are starving and cannot rest."
                                                                    .to_string();
                                                        } else if player.food < 300 {
                                                            status_msg = "You are too faint to rest effectively.".to_string();
                                                        } else {
                                                            player.hp =
                                                                (player.hp + 1).min(player.max_hp);
                                                            player.mana = (player.mana + 1)
                                                                .min(player.max_mana);
                                                            status_msg =
                                                                "You rest and recover health/mana."
                                                                    .to_string();
                                                        }
                                                    } else {
                                                        status_msg = format!(
                                                            "Moved to ({}, {})",
                                                            next_x, next_y
                                                        );
                                                    }

                                                    if let Some(tt) = trigger_trap {
                                                        let (px, py) = (player.x, player.y);
                                                        let trap_msg = trigger_trap_effect(
                                                            tt,
                                                            &mut player,
                                                            &mut level,
                                                            &mut monsters,
                                                            &mut rng,
                                                            px,
                                                            py,
                                                        );
                                                        status_msg = trap_msg;
                                                        if tt != TrapType::CoveredPit
                                                            && tt != TrapType::Rockfall
                                                            && let Some(t) =
                                                                level.get_tile_mut(px, py)
                                                        {
                                                            t.tile_type = TileType::Floor;
                                                        }
                                                        if player.hp <= 0 {
                                                            player.hp = 0;
                                                            if save_path.exists() {
                                                                let _ = fs::remove_file(&save_path);
                                                            }
                                                            player.died_from = "a trap".to_string();
                                                            screen_mode = if player.balrog_killed
                                                                || player.total_winner
                                                            {
                                                                ScreenMode::RoyalCoronation
                                                            } else {
                                                                ScreenMode::Tombstone
                                                            };
                                                            status_msg.push_str(" You have died!");
                                                        }
                                                    }

                                                    let item_idx =
                                                        level.items.iter().position(|i| {
                                                            i.x == player.x && i.y == player.y
                                                        });
                                                    if let Some(idx) = item_idx {
                                                        let is_gold = level.items[idx]
                                                            .item
                                                            .name
                                                            .contains("Gold Pile");
                                                        let should_pickup =
                                                            matches!(action, Action::Move(_))
                                                                && player.auto_pickup;
                                                        if is_gold {
                                                            let floor_item =
                                                                level.items.remove(idx);
                                                            let item = floor_item.item;
                                                            let gold_amount = item
                                                                .name
                                                                .split('[')
                                                                .nth(1)
                                                                .and_then(|s| s.split(' ').next())
                                                                .and_then(|s| s.parse::<u32>().ok())
                                                                .unwrap_or(20);
                                                            player.gold += gold_amount;
                                                            status_msg = format!(
                                                                "You picked up {} gold pieces.",
                                                                gold_amount
                                                            );
                                                        } else if !should_pickup {
                                                            status_msg = format!(
                                                                "You see a {}.",
                                                                level.items[idx]
                                                                    .item
                                                                    .display_name()
                                                            );
                                                        } else if !player
                                                            .can_carry_item(&level.items[idx].item)
                                                        {
                                                            status_msg = format!(
                                                                "You can't carry {}.",
                                                                level.items[idx]
                                                                    .item
                                                                    .display_name()
                                                            );
                                                        } else {
                                                            let floor_item =
                                                                level.items.remove(idx);
                                                            let mut item = floor_item.item;
                                                            if !item.identified {
                                                                player.flavors.assign_flavor(
                                                                    &mut item, &mut rng,
                                                                );
                                                                if player
                                                                    .flavors
                                                                    .is_identified(&item.name)
                                                                {
                                                                    item.identified = true;
                                                                }
                                                            }
                                                            let dname = item.display_name();
                                                            status_msg = format!(
                                                                "You picked up a {}.",
                                                                dname
                                                            );
                                                            player.add_item_to_inventory(item);
                                                        }
                                                    }

                                                    player_acted = true;
                                                } else {
                                                    match tile.tile_type {
                                                        TileType::SecretDoor => {
                                                            let is_rogue =
                                                                player.class == Class::Rogue;
                                                            let is_elf = matches!(
                                                                player.race,
                                                                Race::Elf | Race::HalfElf
                                                            );
                                                            let door_chance = if is_rogue {
                                                                0.60
                                                            } else if is_elf {
                                                                0.40
                                                            } else {
                                                                0.20
                                                            };
                                                            if rng.gen_bool(door_chance) {
                                                                if let Some(t) = level
                                                                    .get_tile_mut(next_x, next_y)
                                                                {
                                                                    t.tile_type =
                                                                        TileType::DoorClosed {
                                                                            spikes: 0,
                                                                        };
                                                                }
                                                                status_msg = "You feel a seam in the wall and find a secret door!".to_string();
                                                            } else {
                                                                status_msg =
                                                                    "Ouch! You bumped into a wall."
                                                                        .to_string();
                                                            }
                                                            player_acted = true;
                                                        }
                                                        TileType::DoorClosed { spikes } => {
                                                            if spikes > 0 {
                                                                status_msg = format!(
                                                                    "The door is spiked shut! ({} spikes)",
                                                                    spikes
                                                                );
                                                            } else {
                                                                if let Some(t) = level
                                                                    .get_tile_mut(next_x, next_y)
                                                                {
                                                                    t.tile_type =
                                                                        TileType::DoorOpen;
                                                                }
                                                                status_msg = "You open the door."
                                                                    .to_string();
                                                                player_acted = true;
                                                            }
                                                        }
                                                        TileType::Chest { trapped: _, locked } => {
                                                            if locked {
                                                                status_msg = "There is a chest here, but it is locked! (Use 'D' to disarm or pick the lock, or 'o' to force open)".to_string();
                                                            } else {
                                                                level.open_chest(
                                                                    next_x,
                                                                    next_y,
                                                                    &mut player,
                                                                    &mut rng,
                                                                    &mut status_msg,
                                                                    &mut monsters,
                                                                );
                                                                player_acted = true;
                                                            }
                                                        }
                                                        TileType::Rubble => {
                                                            let ability = player.digging_ability();
                                                            if ability == 0 {
                                                                status_msg = "There is rubble blocking your way. (Equip a shovel or pickaxe to dig)".to_string();
                                                            } else {
                                                                let chance = rng.gen_range(0..180);
                                                                if ability > chance {
                                                                    if let Some(t) = level
                                                                        .get_tile_mut(
                                                                            next_x, next_y,
                                                                        )
                                                                    {
                                                                        t.tile_type =
                                                                            TileType::Floor;
                                                                    }
                                                                    status_msg = "You have removed the rubble.".to_string();
                                                                    if rng.gen_bool(0.10) {
                                                                        let item = generate_random_floor_item(level.depth, &mut rng);
                                                                        status_msg = format!(
                                                                            "You removed the rubble and found a {} under it!",
                                                                            item.name
                                                                        );
                                                                        level.items.push(
                                                                            FloorItem {
                                                                                x: next_x,
                                                                                y: next_y,
                                                                                item,
                                                                            },
                                                                        );
                                                                    }
                                                                } else {
                                                                    status_msg =
                                                                        "You dig in the rubble."
                                                                            .to_string();
                                                                }
                                                                player_acted = true;
                                                            }
                                                        }
                                                        TileType::MagmaVein { .. } => {
                                                            status_msg = "There is a magma vein in the granite wall. (Use 'T' to tunnel)".to_string();
                                                        }
                                                        TileType::QuartzVein { .. } => {
                                                            status_msg = "There is a quartz vein in the granite wall. (Use 'T' to tunnel)".to_string();
                                                        }
                                                        _ => {
                                                            status_msg =
                                                                "Ouch! You bumped into a wall."
                                                                    .to_string();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Action::GoUpStairs => {
                                        if let Some(tile) = level.get_tile(player.x, player.y) {
                                            if tile.tile_type == TileType::StairsUp
                                                && level.depth > 0
                                            {
                                                let new_depth = level.depth - 1;
                                                if level.depth == 0 {
                                                    level_cache.insert(
                                                        0,
                                                        (level.clone(), monsters.clone()),
                                                    );
                                                }
                                                level_cache.retain(|&k, _| k == 0);
                                                if let Some((cached_level, cached_monsters)) =
                                                    level_cache.remove(&new_depth)
                                                {
                                                    level = cached_level;
                                                    monsters = cached_monsters;
                                                } else {
                                                    level = DungeonLevel::new(
                                                        66, 22, new_depth, max_depth,
                                                    );
                                                    level.generate_simple_floor();
                                                    monsters = level
                                                        .generate_monsters(player.balrog_killed);
                                                }

                                                let sd_pos = level
                                                    .tiles
                                                    .iter()
                                                    .enumerate()
                                                    .find(|(_, t)| {
                                                        t.tile_type == TileType::StairsDown
                                                    })
                                                    .map(|(i, _)| {
                                                        (i % level.width, i / level.width)
                                                    })
                                                    .unwrap_or((40, 15));
                                                player.move_to(sd_pos.0, sd_pos.1);

                                                status_msg = format!(
                                                    "You climb up to depth {} ({} feet).",
                                                    new_depth,
                                                    new_depth * 50
                                                );
                                                player_acted = true;
                                            } else {
                                                status_msg =
                                                    "You see no up stairs (<) here.".to_string();
                                            }
                                        }
                                    }
                                    Action::GoDownStairs => {
                                        if let Some(tile) = level.get_tile(player.x, player.y) {
                                            if tile.tile_type == TileType::StairsDown
                                                && level.depth < max_depth
                                            {
                                                let new_depth = level.depth + 1;
                                                if level.depth == 0 {
                                                    level_cache.insert(
                                                        0,
                                                        (level.clone(), monsters.clone()),
                                                    );
                                                }
                                                level_cache.retain(|&k, _| k == 0);
                                                if let Some((cached_level, cached_monsters)) =
                                                    level_cache.remove(&new_depth)
                                                {
                                                    level = cached_level;
                                                    monsters = cached_monsters;
                                                } else {
                                                    level = DungeonLevel::new(
                                                        66, 22, new_depth, max_depth,
                                                    );
                                                    level.generate_simple_floor();
                                                    monsters = level
                                                        .generate_monsters(player.balrog_killed);
                                                }

                                                let su_pos = level
                                                    .tiles
                                                    .iter()
                                                    .enumerate()
                                                    .find(|(_, t)| {
                                                        t.tile_type == TileType::StairsUp
                                                    })
                                                    .map(|(i, _)| {
                                                        (i % level.width, i / level.width)
                                                    })
                                                    .unwrap_or((15, 15));
                                                player.move_to(su_pos.0, su_pos.1);

                                                status_msg = format!(
                                                    "You climb down to depth {} ({} feet).",
                                                    new_depth,
                                                    new_depth * 50
                                                );
                                                player_acted = true;
                                            } else {
                                                status_msg =
                                                    "You see no down stairs (>) here.".to_string();
                                            }
                                        }
                                    }
                                    Action::InventoryList => {
                                        screen_mode = ScreenMode::InventoryList;
                                        status_msg = "Inspecting inventory.".to_string();
                                    }
                                    Action::EquipmentList => {
                                        screen_mode = ScreenMode::EquipmentList;
                                        status_msg = "Inspecting equipment.".to_string();
                                    }
                                    Action::WearWield => {
                                        screen_mode = ScreenMode::WearMenu;
                                        status_msg = "Equip item from inventory.".to_string();
                                    }
                                    Action::TakeOff => {
                                        screen_mode = ScreenMode::TakeOffMenu;
                                        status_msg = "Select item to unequip.".to_string();
                                    }
                                    Action::MultiBlow => {
                                        let weapon = player.equipped_weapon();
                                        let blows = player.calculate_blows(&weapon);
                                        let dmg = player.roll_melee_damage_against(&mut rng, None);
                                        player.message_history.push(format!(
                                            "You strike {} times for {} total damage!",
                                            blows, dmg
                                        ));
                                        screen_mode = ScreenMode::CombatLog;
                                    }
                                    Action::Fire | Action::Throw => {
                                        if player.inventory.is_empty() {
                                            status_msg =
                                                "You have nothing in your inventory to throw/fire."
                                                    .to_string();
                                        } else {
                                            screen_mode = ScreenMode::SelectThrowItem;
                                            status_msg = "Fire/Throw which item? Select a letter [a-z] or [ESC] to cancel...".to_string();
                                        }
                                    }
                                    Action::Eat => {
                                        let has_food = player
                                            .inventory
                                            .iter()
                                            .any(|i| matches!(i.item_type, ItemType::Food { .. }));
                                        if has_food {
                                            screen_mode = ScreenMode::EatMenu;
                                            status_msg = "Select food to eat.".to_string();
                                        } else {
                                            status_msg = "You do not carry any food!".to_string();
                                        }
                                    }
                                    Action::Rest => {
                                        if player.food < 0 {
                                            status_msg =
                                                "You are starving and cannot rest!".to_string();
                                        } else if player.food < 300 {
                                            status_msg = "You are too faint from hunger to rest!"
                                                .to_string();
                                        } else {
                                            player.searching = false;
                                            screen_mode = ScreenMode::SelectRestCount;
                                            rest_input_buffer.clear();
                                            status_msg = "Rest how long? (number of turns, or * for until healed): ".to_string();
                                        }
                                    }
                                    Action::Bash => {
                                        screen_mode = ScreenMode::SelectBashDirection;
                                        status_msg = "Bash: choose direction...".to_string();
                                    }
                                    Action::Tunnel => {
                                        screen_mode = ScreenMode::SelectTunnelDirection;
                                        status_msg = "Tunnel: choose direction...".to_string();
                                    }
                                    Action::RefillLight => {
                                        let has_refillable_light =
                                            player.equipment.iter().any(|i| {
                                                matches!(i.item_type, ItemType::Light { .. })
                                                    && (i.name.contains("Lantern")
                                                        || i.name.contains("Lamp"))
                                            });
                                        if !has_refillable_light {
                                            let is_wielding_torch =
                                                player.equipment.iter().any(|i| {
                                                    matches!(i.item_type, ItemType::Light { .. })
                                                        && i.name.contains("Torch")
                                                });
                                            if is_wielding_torch {
                                                status_msg =
                                                    "You cannot refill a torch!".to_string();
                                            } else {
                                                status_msg =
                                                    "You are not wielding a lamp or lantern."
                                                        .to_string();
                                            }
                                        } else {
                                            let oil_position = player
                                                .inventory
                                                .iter()
                                                .position(|i| i.name.contains("Flask of Oil"));
                                            match oil_position {
                                                None => {
                                                    status_msg = "You have no oil.".to_string();
                                                }
                                                Some(idx) => {
                                                    // We have oil and a refillable light source equipped
                                                    if let Some(light_item) =
                                                        player.equipment.iter_mut().find(|i| {
                                                            matches!(
                                                                i.item_type,
                                                                ItemType::Light { .. }
                                                            ) && (i.name.contains("Lantern")
                                                                || i.name.contains("Lamp"))
                                                        })
                                                        && let ItemType::Light { ref mut fuel } =
                                                            light_item.item_type
                                                    {
                                                        if *fuel >= 7500 {
                                                            *fuel = 7500;
                                                            status_msg = "Your lamp overflows, spilling oil on the ground.".to_string();
                                                        } else {
                                                            *fuel = (*fuel + 7500).min(7500);
                                                            status_msg =
                                                                "You refill your lamp.".to_string();
                                                        }
                                                    }
                                                    // Consume 1 flask of oil
                                                    player.inventory[idx].count -= 1;
                                                    if player.inventory[idx].count == 0 {
                                                        player.inventory.remove(idx);
                                                    }
                                                    player_acted = true;
                                                }
                                            }
                                        }
                                    }
                                    Action::Quaff => {
                                        screen_mode = ScreenMode::QuaffMenu;
                                        status_msg = "Select potion to quaff.".to_string();
                                    }
                                    Action::ReadScroll => {
                                        if player.flags.blind > 0 {
                                            status_msg =
                                                "You can't see to read the scroll.".to_string();
                                        } else if player.flags.confused > 0 {
                                            status_msg = "You are too confused to read a scroll."
                                                .to_string();
                                        } else {
                                            screen_mode = ScreenMode::ReadMenu;
                                            status_msg = "Select scroll to read.".to_string();
                                        }
                                    }
                                    Action::BrowseBook => {
                                        if player.flags.blind > 0 {
                                            status_msg =
                                                "You can't see to read your book!".to_string();
                                        } else if player.flags.confused > 0 {
                                            status_msg = "You are too confused.".to_string();
                                        } else {
                                            let is_prayer = player.is_priest_caster();
                                            let has_book =
                                                !player.carried_books(is_prayer).is_empty()
                                                    || player.is_wizard;
                                            if has_book {
                                                screen_mode = ScreenMode::BrowseBookMenu;
                                                status_msg = "Browsing spells/prayers (press any key to exit).".to_string();
                                            } else {
                                                status_msg = "You do not have any books to read."
                                                    .to_string();
                                            }
                                        }
                                    }
                                    Action::CastSpell => {
                                        if player.flags.blind > 0 {
                                            status_msg = "You can't see to read your spell book!"
                                                .to_string();
                                        } else if player.flags.confused > 0 {
                                            status_msg = "You are too confused.".to_string();
                                        } else if !player.is_mage_caster() {
                                            status_msg = "You can't cast spells!".to_string();
                                        } else if player.carried_books(false).is_empty()
                                            && !player.is_wizard
                                        {
                                            status_msg =
                                                "But you are not carrying any spell-books!"
                                                    .to_string();
                                        } else {
                                            let known = player.known_spells(false);
                                            if known.is_empty() {
                                                status_msg =
                                                    "You don't know any spells in your books."
                                                        .to_string();
                                            } else {
                                                screen_mode = ScreenMode::CastSpellMenu;
                                                status_msg =
                                                    "Cast which spell? (ESC to cancel)".to_string();
                                            }
                                        }
                                    }
                                    Action::Pray => {
                                        if player.flags.blind > 0 {
                                            status_msg =
                                                "You can't see to read your prayer!".to_string();
                                        } else if player.flags.confused > 0 {
                                            status_msg = "You are too confused.".to_string();
                                        } else if !player.is_priest_caster() {
                                            status_msg = "Pray hard enough and your prayers may be answered.".to_string();
                                        } else if player.carried_books(true).is_empty()
                                            && !player.is_wizard
                                        {
                                            status_msg =
                                                "You are not carrying any Holy Books!".to_string();
                                        } else {
                                            let known = player.known_spells(true);
                                            if known.is_empty() {
                                                status_msg =
                                                    "You don't know any prayers in your books."
                                                        .to_string();
                                            } else {
                                                screen_mode = ScreenMode::PrayMenu;
                                                status_msg = "Recite which prayer? (ESC to cancel)"
                                                    .to_string();
                                            }
                                        }
                                    }
                                    Action::GainSpells => {
                                        if player.flags.confused > 0 {
                                            status_msg = "You are too confused.".to_string();
                                        } else if player.is_priest_caster() {
                                            if player.new_spells_to_learn == 0 {
                                                status_msg =
                                                    "You can't learn any new prayers!".to_string();
                                            } else {
                                                let learnable = player.learnable_spells();
                                                if learnable.is_empty() {
                                                    status_msg = "You cannot learn any new prayers at your current level.".to_string();
                                                } else {
                                                    let choice = learnable
                                                        [rng.gen_range(0..learnable.len())];
                                                    player.learn_spell(choice);
                                                    if let Some(def) = crate::magic::get_spell_def(
                                                        player.class,
                                                        choice,
                                                        true,
                                                    ) {
                                                        status_msg = format!(
                                                            "You have learned the prayer of {}.",
                                                            def.name
                                                        );
                                                    }
                                                    player_acted = true;
                                                }
                                            }
                                        } else if player.is_mage_caster() {
                                            if player.flags.blind > 0 {
                                                status_msg =
                                                    "You can't see to read your spell book!"
                                                        .to_string();
                                            } else if player.new_spells_to_learn == 0 {
                                                status_msg =
                                                    "You can't learn any new spells!".to_string();
                                            } else {
                                                let learnable = player.learnable_spells();
                                                if learnable.is_empty() {
                                                    status_msg = "You seem to be missing a book."
                                                        .to_string();
                                                } else {
                                                    screen_mode = ScreenMode::StudyMenu;
                                                    status_msg =
                                                        "Learn which spell? (ESC to cancel)"
                                                            .to_string();
                                                }
                                            }
                                        } else {
                                            status_msg =
                                                "You cannot learn spells or prayers.".to_string();
                                        }
                                    }
                                    Action::CharacterStats => {
                                        screen_mode = ScreenMode::CharacterStatsMenu;
                                        status_msg = "Viewing character sheet.".to_string();
                                    }
                                    Action::Disarm => {
                                        screen_mode = ScreenMode::SelectDisarmDirection;
                                        status_msg = "Disarm trap: choose direction...".to_string();
                                    }
                                    Action::Inscribe => {
                                        if player.inventory.is_empty() {
                                            status_msg =
                                                "You have nothing to inscribe!".to_string();
                                        } else {
                                            screen_mode = ScreenMode::InscribeMenu;
                                            status_msg =
                                                "Inscribe item: select a letter.".to_string();
                                        }
                                    }
                                    Action::OpenDoor => {
                                        screen_mode = ScreenMode::SelectOpenDirection;
                                        status_msg = "Open door: choose direction...".to_string();
                                    }
                                    Action::Look => {
                                        screen_mode = ScreenMode::SelectLookDirection;
                                        status_msg = "Look: choose direction...".to_string();
                                    }
                                    Action::CloseDoor => {
                                        screen_mode = ScreenMode::SelectCloseDirection;
                                        status_msg = "Close door: choose direction...".to_string();
                                    }
                                    Action::JamDoor => {
                                        let has_spike = player
                                            .inventory
                                            .iter()
                                            .any(|i| i.name == "Iron Spike" && i.count > 0);
                                        if has_spike {
                                            screen_mode = ScreenMode::SelectJamDirection;
                                            status_msg =
                                                "Jam door: choose direction...".to_string();
                                        } else {
                                            status_msg =
                                                "You do not carry any iron spikes!".to_string();
                                        }
                                    }
                                    Action::DropItem => {
                                        screen_mode = ScreenMode::DropMenu;
                                        status_msg = "Drop item: select a letter.".to_string();
                                    }
                                    Action::Help => {
                                        screen_mode = ScreenMode::HelpMenu;
                                        status_msg = "Browsing command list.".to_string();
                                    }
                                    Action::SearchOneTurn => {
                                        let is_rogue = player.class == Class::Rogue;
                                        let is_elf =
                                            matches!(player.race, Race::Elf | Race::HalfElf);
                                        let trap_chance = if is_rogue { 0.70 } else { 0.30 };
                                        let door_chance = if is_rogue {
                                            0.60
                                        } else if is_elf {
                                            0.40
                                        } else {
                                            0.20
                                        };

                                        let mut found_something = false;

                                        for dy in -1..=1 {
                                            for dx in -1..=1 {
                                                if dx == 0 && dy == 0 {
                                                    continue;
                                                }
                                                let sx = (player.x as isize + dx) as usize;
                                                let sy = (player.y as isize + dy) as usize;

                                                if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                    match tile.tile_type {
                                                        TileType::SecretDoor => {
                                                            if rng.gen_bool(door_chance) {
                                                                tile.tile_type =
                                                                    TileType::DoorClosed {
                                                                        spikes: 0,
                                                                    };
                                                                status_msg =
                                                                    "You found a secret door!"
                                                                        .to_string();
                                                                found_something = true;
                                                            }
                                                        }
                                                        TileType::Trap {
                                                            ref mut detected, ..
                                                        } if !*detected
                                                            && rng.gen_bool(trap_chance) =>
                                                        {
                                                            *detected = true;
                                                            status_msg =
                                                                "You detected a trap!".to_string();
                                                            found_something = true;
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }

                                        if !found_something {
                                            status_msg = "You search but find nothing.".to_string();
                                        }
                                        player_acted = true;
                                    }
                                    Action::ToggleSearch => {
                                        player.searching = !player.searching;
                                        if player.searching {
                                            status_msg = "Searching...".to_string();
                                        } else {
                                            status_msg = "No longer searching.".to_string();
                                        }
                                    }
                                    Action::Quit => {
                                        if player.is_wizard {
                                            status_msg = "TEST MODE ACTIVE: Character cannot be saved. Quitting...".to_string();
                                            draw_map(
                                                &mut level,
                                                &player,
                                                &monsters,
                                                &status_msg,
                                                screen_mode,
                                                active_shop,
                                                active_haggle.as_ref(),
                                                active_inscribe.as_ref(),
                                            )?;
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                800,
                                            ));
                                            break 'game_loop;
                                        }
                                        if save_path.exists() {
                                            let _ = fs::remove_file(&save_path);
                                        }
                                        if player.balrog_killed || player.total_winner {
                                            player.died_from = "Ripe Old Age".to_string();
                                            player.total_winner = true;
                                            screen_mode = ScreenMode::RoyalCoronation;
                                        } else {
                                            player.died_from = "Quitting".to_string();
                                            screen_mode = ScreenMode::Tombstone;
                                        }
                                    }
                                    Action::Run(dir) => {
                                        execute_player_run(
                                            dir,
                                            &mut player,
                                            &mut level,
                                            &mut monsters,
                                            &mut status_msg,
                                            &mut screen_mode,
                                            &save_path,
                                            active_shop,
                                            active_haggle.as_ref(),
                                            &mut game_turn,
                                            max_depth,
                                            &mut level_cache,
                                        )?;
                                        player_acted = true;
                                    }
                                    Action::ReducedMap => {
                                        screen_mode = ScreenMode::ReducedMap;
                                    }
                                    Action::LocateMap => {
                                        if player.flags.blind > 0 {
                                            status_msg = "You can't see your map.".to_string();
                                        } else {
                                            player.locate_offset = (0, 0);
                                            screen_mode = ScreenMode::LocateMap;
                                        }
                                    }
                                    Action::IdentifySymbol => {
                                        player.identified_symbol = None;
                                        screen_mode = ScreenMode::IdentifySymbol;
                                    }
                                    Action::OptionsMenu => {
                                        screen_mode = ScreenMode::OptionsMenu;
                                    }
                                    Action::HighScores => {
                                        player.high_score_page = 0;
                                        screen_mode = ScreenMode::HighScores;
                                    }
                                    _ => {}
                                }
                            }
                        } else if screen_mode == ScreenMode::BarterBuyMenu
                            || screen_mode == ScreenMode::BarterSellMenu
                        {
                            let shop_info =
                                level.shops.iter_mut().find(|s| s.shop_type == active_shop);
                            handle_haggle_input(
                                key_event,
                                &mut screen_mode,
                                &mut active_haggle,
                                &mut player,
                                &mut status_msg,
                                game_turn,
                                shop_info,
                            );
                        } else {
                            match key_event.code {
                                KeyCode::Esc => {
                                    if screen_mode == ScreenMode::ShopSellMenu {
                                        screen_mode = ScreenMode::Shop;
                                        let shop_info =
                                            level.shops.iter().find(|s| s.shop_type == active_shop);
                                        let owner_name = shop_info
                                            .map(|s| s.owner().name.trim())
                                            .unwrap_or("Store");
                                        status_msg = format!("{} - Buy Menu.", owner_name);
                                    } else if screen_mode == ScreenMode::HighScores {
                                        if player.hp <= 0 || !player.died_from.is_empty() {
                                            break 'game_loop;
                                        } else {
                                            player.high_score_page = 0;
                                            screen_mode = ScreenMode::Dungeon;
                                            status_msg = "Returned to dungeon.".to_string();
                                        }
                                    } else if screen_mode == ScreenMode::Tombstone
                                        || screen_mode == ScreenMode::GameOver
                                    {
                                        let entry = HighScoreEntry::from_player(
                                            &player,
                                            level.depth,
                                            &player.died_from,
                                        );
                                        let _ = record_high_score(entry, get_scores_path());
                                        screen_mode = ScreenMode::HighScores;
                                    } else if screen_mode == ScreenMode::RoyalCoronation {
                                        screen_mode = ScreenMode::Tombstone;
                                    } else if screen_mode == ScreenMode::LocateMap {
                                        player.locate_offset = (0, 0);
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                    } else {
                                        player.identified_symbol = None;
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                        active_haggle = None;
                                    }
                                }
                                KeyCode::PageDown => {
                                    if screen_mode == ScreenMode::HighScores {
                                        player.high_score_page += 1;
                                    }
                                }
                                KeyCode::PageUp => {
                                    if screen_mode == ScreenMode::HighScores {
                                        player.high_score_page =
                                            player.high_score_page.saturating_sub(1);
                                    }
                                }
                                KeyCode::Enter => {
                                    if screen_mode == ScreenMode::HighScores {
                                        if player.hp <= 0 || !player.died_from.is_empty() {
                                            break 'game_loop;
                                        } else {
                                            player.high_score_page = 0;
                                            screen_mode = ScreenMode::Dungeon;
                                            status_msg = "Returned to dungeon.".to_string();
                                        }
                                    } else if screen_mode == ScreenMode::RoyalCoronation {
                                        screen_mode = ScreenMode::Tombstone;
                                    } else if screen_mode == ScreenMode::Tombstone
                                        || screen_mode == ScreenMode::GameOver
                                    {
                                        let entry = HighScoreEntry::from_player(
                                            &player,
                                            level.depth,
                                            &player.died_from,
                                        );
                                        let _ = record_high_score(entry, get_scores_path());
                                        screen_mode = ScreenMode::HighScores;
                                    } else if screen_mode == ScreenMode::LocateMap {
                                        player.locate_offset = (0, 0);
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                    } else if screen_mode == ScreenMode::ReducedMap
                                        || screen_mode == ScreenMode::OptionsMenu
                                    {
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                    } else if screen_mode == ScreenMode::IdentifySymbol {
                                        player.identified_symbol = None;
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if (screen_mode == ScreenMode::InventoryList && c == 'i')
                                        || (screen_mode == ScreenMode::EquipmentList && c == 'I')
                                        || screen_mode == ScreenMode::HelpMenu
                                        || screen_mode == ScreenMode::CombatLog
                                        || screen_mode == ScreenMode::MessageHistory
                                        || screen_mode == ScreenMode::BrowseBookMenu
                                        || screen_mode == ScreenMode::ReducedMap
                                    {
                                        screen_mode = ScreenMode::Dungeon;
                                        status_msg = "Returned to dungeon.".to_string();
                                    } else if screen_mode == ScreenMode::LocateMap {
                                        let dir_opt = mapper.map_direction_key(c).or_else(|| {
                                            match mapper.map_key(c) {
                                                Action::Move(d)
                                                | Action::Run(d)
                                                | Action::MoveWithoutPickup(d) => Some(d),
                                                _ => None,
                                            }
                                        });
                                        if let Some(dir) = dir_opt {
                                            let (dx, dy) = match dir {
                                                Direction::NorthWest => (-20, -10),
                                                Direction::North => (0, -10),
                                                Direction::NorthEast => (20, -10),
                                                Direction::West => (-20, 0),
                                                Direction::East => (20, 0),
                                                Direction::SouthWest => (-20, 10),
                                                Direction::South => (0, 10),
                                                Direction::SouthEast => (20, 10),
                                                Direction::Rest => (0, 0),
                                            };
                                            player.locate_offset.0 = (player.locate_offset.0 + dx)
                                                .clamp(
                                                    -(level.width as isize),
                                                    level.width as isize,
                                                );
                                            player.locate_offset.1 = (player.locate_offset.1 + dy)
                                                .clamp(
                                                    -(level.height as isize),
                                                    level.height as isize,
                                                );
                                        } else if c == ' '
                                            || c == 'q'
                                            || c == 'Q'
                                            || c == '\n'
                                            || c == '\r'
                                        {
                                            player.locate_offset = (0, 0);
                                            screen_mode = ScreenMode::Dungeon;
                                            status_msg = "Returned to dungeon.".to_string();
                                        }
                                    } else if screen_mode == ScreenMode::IdentifySymbol {
                                        if player.identified_symbol.is_none() {
                                            player.identified_symbol = Some(c);
                                        } else {
                                            player.identified_symbol = None;
                                            screen_mode = ScreenMode::Dungeon;
                                            status_msg = "Returned to dungeon.".to_string();
                                        }
                                    } else if screen_mode == ScreenMode::OptionsMenu {
                                        if c == '1' {
                                            player.keybinding_mode = match player.keybinding_mode {
                                                KeybindingMode::Original => {
                                                    KeybindingMode::Roguelike
                                                }
                                                KeybindingMode::Roguelike => {
                                                    KeybindingMode::ModernWasd
                                                }
                                                KeybindingMode::ModernWasd => {
                                                    KeybindingMode::Original
                                                }
                                            };
                                            mapper.mode = player.keybinding_mode;
                                            status_msg = format!(
                                                "Keybinding profile: {:?}",
                                                player.keybinding_mode
                                            );
                                        } else if c == '2' {
                                            player.auto_pickup = !player.auto_pickup;
                                            status_msg =
                                                format!("Auto-pickup: {}", player.auto_pickup);
                                        } else if c == ' '
                                            || c == 'q'
                                            || c == 'Q'
                                            || c == '\n'
                                            || c == '\r'
                                        {
                                            screen_mode = ScreenMode::Dungeon;
                                            status_msg = "Returned to dungeon.".to_string();
                                        }
                                    } else if screen_mode == ScreenMode::HighScores {
                                        if c == ' ' || c == 'n' || c == 'N' {
                                            player.high_score_page += 1;
                                        } else if c == 'p' || c == 'P' {
                                            player.high_score_page =
                                                player.high_score_page.saturating_sub(1);
                                        } else if c == 'q' || c == 'Q' || c == '\n' || c == '\r' {
                                            if player.hp <= 0 || !player.died_from.is_empty() {
                                                break 'game_loop;
                                            } else {
                                                player.high_score_page = 0;
                                                screen_mode = ScreenMode::Dungeon;
                                                status_msg = "Returned to dungeon.".to_string();
                                            }
                                        }
                                    } else if screen_mode == ScreenMode::RoyalCoronation {
                                        screen_mode = ScreenMode::Tombstone;
                                    } else if screen_mode == ScreenMode::CharacterStatsMenu {
                                        // Exit sheet via ESC
                                    } else if screen_mode == ScreenMode::SelectDisarmDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            let mut disarmed = false;
                                            let mut chest_ref = None;
                                            if let Some(tile) = level.get_tile(sx, sy)
                                                && let TileType::Chest { trapped, locked } =
                                                    tile.tile_type
                                            {
                                                chest_ref = Some((trapped, locked));
                                            }

                                            if let Some((trapped, locked)) = chest_ref {
                                                if trapped {
                                                    let total = player.trap_disarm_ability();
                                                    let level_diff = level.depth as i32;
                                                    if total + 100 - level_diff
                                                        > rng.gen_range(1..=100)
                                                    {
                                                        if let Some(tile) =
                                                            level.get_tile_mut(sx, sy)
                                                        {
                                                            tile.tile_type = TileType::Chest {
                                                                trapped: false,
                                                                locked,
                                                            };
                                                        }
                                                        player
                                                            .add_experience(level.depth.max(1) * 2);
                                                        status_msg = "You successfully disarmed the chest trap.".to_string();
                                                    } else if total > 5
                                                        && rng.gen_range(1..=total) > 5
                                                    {
                                                        status_msg =
                                                            "You failed to disarm the chest trap."
                                                                .to_string();
                                                    } else {
                                                        status_msg = "You set off the chest trap while trying to disarm it!".to_string();
                                                        level.trigger_chest_trap(
                                                            sx,
                                                            sy,
                                                            &mut player,
                                                            &mut rng,
                                                            &mut status_msg,
                                                            &mut monsters,
                                                        );
                                                    }
                                                } else if locked {
                                                    let total = player.trap_disarm_ability();
                                                    let level_diff = level.depth as i32;
                                                    if total + 100 - level_diff
                                                        > rng.gen_range(1..=100)
                                                    {
                                                        if let Some(tile) =
                                                            level.get_tile_mut(sx, sy)
                                                        {
                                                            tile.tile_type = TileType::Chest {
                                                                trapped: false,
                                                                locked: false,
                                                            };
                                                        }
                                                        player.add_experience(level.depth.max(1));
                                                        status_msg =
                                                            "You picked the lock on the chest."
                                                                .to_string();
                                                    } else {
                                                        status_msg = "You failed to pick the lock on the chest.".to_string();
                                                    }
                                                } else {
                                                    status_msg = "The chest is already disarmed and unlocked.".to_string();
                                                }
                                                disarmed = true;
                                            } else {
                                                let mut trap_info = None;
                                                if let Some(tile) = level.get_tile(sx, sy)
                                                    && let TileType::Trap {
                                                        detected,
                                                        trap_type,
                                                    } = tile.tile_type
                                                    && detected
                                                {
                                                    trap_info = Some(trap_type);
                                                }

                                                if let Some(trap_type) = trap_info {
                                                    let total = player.trap_disarm_ability();
                                                    let level_diff = trap_type.difficulty();
                                                    if total + 100 - level_diff
                                                        > rng.gen_range(1..=100)
                                                    {
                                                        if let Some(tile) =
                                                            level.get_tile_mut(sx, sy)
                                                        {
                                                            tile.tile_type = TileType::Floor;
                                                        }
                                                        player.add_experience(
                                                            trap_type.disarm_exp() as u32,
                                                        );
                                                        status_msg =
                                                            "You successfully disarmed the trap."
                                                                .to_string();
                                                    } else if total > 5
                                                        && rng.gen_range(1..=total) > 5
                                                    {
                                                        status_msg =
                                                            "You failed to disarm the trap."
                                                                .to_string();
                                                    } else {
                                                        status_msg = "You set off the trap while trying to disarm it! ".to_string();
                                                        let trap_msg = trigger_trap_effect(
                                                            trap_type,
                                                            &mut player,
                                                            &mut level,
                                                            &mut monsters,
                                                            &mut rng,
                                                            sx,
                                                            sy,
                                                        );
                                                        status_msg.push_str(&trap_msg);
                                                        if trap_type != TrapType::CoveredPit
                                                            && trap_type != TrapType::Rockfall
                                                            && let Some(tile) =
                                                                level.get_tile_mut(sx, sy)
                                                        {
                                                            tile.tile_type = TileType::Floor;
                                                        }
                                                    }
                                                    disarmed = true;
                                                }
                                            }
                                            if !disarmed {
                                                status_msg = "No detected trap in that direction."
                                                    .to_string();
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::EatMenu {
                                        let idx = c as usize - 'a' as usize;
                                        let food_indices: Vec<usize> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(item.item_type, ItemType::Food { .. })
                                            })
                                            .map(|(i, _)| i)
                                            .collect();

                                        if idx < food_indices.len() {
                                            let inv_idx = food_indices[idx];
                                            let nutrition = if let ItemType::Food { nutrition } =
                                                player.inventory[inv_idx].item_type
                                            {
                                                nutrition
                                            } else {
                                                0
                                            };

                                            let current_food = player.food.max(0);
                                            player.food = (current_food + nutrition).min(15000);
                                            let name = player.inventory[inv_idx].name.clone();

                                            player.inventory[inv_idx].count -= 1;
                                            if player.inventory[inv_idx].count == 0 {
                                                player.inventory.remove(inv_idx);
                                            }

                                            status_msg = format!(
                                                "You ate {}! Restored {} nutrition.",
                                                name, nutrition
                                            );
                                            screen_mode = ScreenMode::Dungeon;
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::InscribeMenu {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.inventory.len() {
                                            active_inscribe = Some(InscribeState {
                                                item_index: idx,
                                                is_equipment: false,
                                                input: String::new(),
                                            });
                                            screen_mode = ScreenMode::Inscribing;
                                            status_msg =
                                                "Inscribing... type characters and press ENTER."
                                                    .to_string();
                                        }
                                    } else if screen_mode == ScreenMode::GameOver
                                        || screen_mode == ScreenMode::Tombstone
                                    {
                                        if c == 'y' || c == 'Y' {
                                            let dump =
                                                player.generate_character_record(level.depth);
                                            let _ = fs::write("rmoria_char.txt", dump);
                                            let entry = HighScoreEntry::from_player(
                                                &player,
                                                level.depth,
                                                &player.died_from,
                                            );
                                            let _ = record_high_score(entry, get_scores_path());
                                            status_msg =
                                                "Character dump written to rmoria_char.txt."
                                                    .to_string();
                                            screen_mode = ScreenMode::HighScores;
                                        } else if c == 'r' || c == 'R' {
                                            let entry = HighScoreEntry::from_player(
                                                &player,
                                                level.depth,
                                                &player.died_from,
                                            );
                                            let _ = record_high_score(entry, get_scores_path());
                                            restart_game = true;
                                            break 'game_loop;
                                        } else {
                                            let entry = HighScoreEntry::from_player(
                                                &player,
                                                level.depth,
                                                &player.died_from,
                                            );
                                            let _ = record_high_score(entry, get_scores_path());
                                            screen_mode = ScreenMode::HighScores;
                                        }
                                    } else if screen_mode == ScreenMode::SelectOpenDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                match tile.tile_type {
                                                    TileType::DoorClosed { spikes } => {
                                                        if spikes > 0 {
                                                            status_msg = format!(
                                                                "The door is spiked shut! ({} spikes)",
                                                                spikes
                                                            );
                                                        } else {
                                                            tile.tile_type = TileType::DoorOpen;
                                                            status_msg =
                                                                "You open the door.".to_string();
                                                            player_acted = true;
                                                        }
                                                    }
                                                    TileType::Chest { trapped: _, locked } => {
                                                        if locked {
                                                            status_msg =
                                                                "The chest is locked!".to_string();
                                                        } else {
                                                            level.open_chest(
                                                                sx,
                                                                sy,
                                                                &mut player,
                                                                &mut rng,
                                                                &mut status_msg,
                                                                &mut monsters,
                                                            );
                                                            player_acted = true;
                                                        }
                                                    }
                                                    _ => {
                                                        status_msg =
                                                            "That is not a closed door or chest."
                                                                .to_string();
                                                    }
                                                }
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectLookDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            if dx != 0 || dy != 0 {
                                                let mut sx = player.x as isize;
                                                let mut sy = player.y as isize;
                                                let mut npc_found = None;
                                                loop {
                                                    sx += dx;
                                                    sy += dy;
                                                    if sx < 0
                                                        || sx >= level.width as isize
                                                        || sy < 0
                                                        || sy >= level.height as isize
                                                    {
                                                        break;
                                                    }
                                                    let x_u = sx as usize;
                                                    let y_u = sy as usize;

                                                    if let Some(tile) = level.get_tile(x_u, y_u) {
                                                        if !tile.is_passable()
                                                            || tile.tile_type
                                                                == TileType::SecretDoor
                                                        {
                                                            break;
                                                        }
                                                        if tile.visible
                                                            && let Some(m) = monsters
                                                                .iter()
                                                                .find(|m| m.x == x_u && m.y == y_u)
                                                        {
                                                            npc_found = Some(m.name.clone());
                                                            break;
                                                        }
                                                    } else {
                                                        break;
                                                    }
                                                }
                                                if let Some(name) = npc_found {
                                                    status_msg = format!("You see: {}.", name);
                                                } else {
                                                    status_msg =
                                                        "You see nothing in that direction."
                                                            .to_string();
                                                }
                                            } else {
                                                status_msg =
                                                    "You look around but focus on nothing."
                                                        .to_string();
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectCloseDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                if tile.tile_type == TileType::DoorOpen {
                                                    tile.tile_type =
                                                        TileType::DoorClosed { spikes: 0 };
                                                    status_msg = "You close the door.".to_string();
                                                    player_acted = true;
                                                } else {
                                                    status_msg =
                                                        "That is not an open door.".to_string();
                                                }
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectJamDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                if let TileType::DoorClosed { ref mut spikes } =
                                                    tile.tile_type
                                                {
                                                    if let Some(spike_idx) =
                                                        player.inventory.iter().position(|i| {
                                                            i.name == "Iron Spike" && i.count > 0
                                                        })
                                                    {
                                                        player.inventory[spike_idx].count -= 1;
                                                        if player.inventory[spike_idx].count == 0 {
                                                            player.inventory.remove(spike_idx);
                                                        }
                                                        *spikes += 1;
                                                        status_msg = format!(
                                                            "You jammed the door. ({} spikes used total)",
                                                            spikes
                                                        );
                                                        player_acted = true;
                                                    } else {
                                                        status_msg =
                                                            "You do not carry any iron spikes!"
                                                                .to_string();
                                                    }
                                                } else {
                                                    status_msg =
                                                        "That is not a closed door.".to_string();
                                                }
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectBashDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            let monster_idx = monsters
                                                .iter()
                                                .position(|m| m.x == sx && m.y == sy);
                                            if player.flags.afraid > 0 {
                                                status_msg = "You are afraid!".to_string();
                                                player_acted = true;
                                            } else if let Some(m_idx) = monster_idx {
                                                let monster_name = monsters[m_idx].name.clone();
                                                let mut base_to_hit = player.stats.strength as i32;
                                                let shield_weight = player
                                                    .equipment
                                                    .iter()
                                                    .find(|i| i.get_equipment_slot() == "On arm")
                                                    .map(|i| i.weight)
                                                    .unwrap_or(0);
                                                base_to_hit += (shield_weight / 2) as i32;
                                                base_to_hit += (player.body_weight / 10) as i32;

                                                let hit = test_being_hit(
                                                    base_to_hit,
                                                    player.level as i32,
                                                    player.stats.dexterity as i32,
                                                    monsters[m_idx].ac as i32,
                                                    0,
                                                    &mut rng,
                                                );

                                                if hit {
                                                    let shield_dice = player
                                                        .equipment
                                                        .iter()
                                                        .find_map(|i| {
                                                            if i.get_equipment_slot() == "On arm" {
                                                                Some(Dice::new(1, 4))
                                                            } else {
                                                                None
                                                            }
                                                        })
                                                        .unwrap_or(Dice::new(1, 1));
                                                    let mut damage =
                                                        shield_dice.roll(&mut rng) as i32;
                                                    damage += (player.body_weight / 60) as i32 + 3;
                                                    if damage < 0 {
                                                        damage = 0;
                                                    }

                                                    status_msg = format!(
                                                        "You bash the {} for {} damage!",
                                                        monster_name, damage
                                                    );

                                                    let can_stun = !(monster_name == "The Balrog"
                                                        || monsters[m_idx].is_win_creature());
                                                    let avg_max_hp = monsters[m_idx].max_hp;
                                                    if can_stun
                                                        && (100
                                                            + rng.gen_range(1..=400)
                                                            + rng.gen_range(1..=400)
                                                            > monsters[m_idx].hp + avg_max_hp)
                                                    {
                                                        monsters[m_idx].stunned = (monsters[m_idx]
                                                            .stunned
                                                            + rng.gen_range(1..=3)
                                                            + 1)
                                                        .min(24);
                                                        status_msg.push_str(" It appears stunned!");
                                                    }

                                                    monsters[m_idx].was_attacked = true;
                                                    let m_lvl = monsters[m_idx].level;
                                                    let is_win = monsters[m_idx].is_win_creature()
                                                        || monster_name == "The Balrog";
                                                    let died = monsters[m_idx].take_damage(damage);
                                                    if died {
                                                        if level.depth == 0 {
                                                            player.killed_town_npcs += 1;
                                                        }
                                                        status_msg.push_str(&format!(
                                                            " You have slain the {}!",
                                                            monster_name
                                                        ));

                                                        let exp_reward =
                                                            monsters[m_idx].experience_reward;
                                                        let old_exp = player.exp;
                                                        let old_lvl = player.level;
                                                        let leveled_up = player
                                                            .gain_kill_experience(
                                                                exp_reward, m_lvl,
                                                            );
                                                        let gained =
                                                            player.exp.saturating_sub(old_exp);
                                                        if leveled_up || player.level > old_lvl {
                                                            status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                                        } else if gained > 0 {
                                                            status_msg.push_str(&format!(
                                                                " Gained {} EXP.",
                                                                gained
                                                            ));
                                                        } else if exp_reward > 0 {
                                                            status_msg.push_str(
                                                                " Gained fractional EXP.",
                                                            );
                                                        } else {
                                                            status_msg.push_str(
                                                                " Gained 0 EXP (Town monster).",
                                                            );
                                                        }

                                                        if is_win {
                                                            player.balrog_killed = true;
                                                            player.total_winner = true;
                                                        }

                                                        let is_boss = monster_name == "The Balrog"
                                                            || monster_name.contains("Lich");
                                                        if is_boss || rng.gen_bool(0.30) {
                                                            let drop_item =
                                                                if monster_name == "The Balrog" {
                                                                    Item::new(
                                                                        "Broadsword of Slaying",
                                                                        1,
                                                                        150,
                                                                        ItemType::Weapon {
                                                                            damage: Dice::new(3, 6),
                                                                        },
                                                                    )
                                                                } else {
                                                                    generate_random_floor_item(
                                                                        level.depth,
                                                                        &mut rng,
                                                                    )
                                                                };
                                                            level.items.push(FloorItem {
                                                                x: sx,
                                                                y: sy,
                                                                item: drop_item.clone(),
                                                            });
                                                            status_msg.push_str(&format!(
                                                                " It dropped a {}!",
                                                                drop_item.name
                                                            ));
                                                        }

                                                        if monster_name == "The Balrog" {
                                                            player.balrog_killed = true;
                                                            player.total_winner = true;
                                                            status_msg.push_str(" CONGRATULATIONS! YOU HAVE SLAIN THE BALROG! You are a Total Winner! Use 'Q' when you wish to retire.");
                                                        }
                                                        monsters.remove(m_idx);
                                                    }
                                                } else {
                                                    status_msg = format!(
                                                        "You swing and miss the {}.",
                                                        monster_name
                                                    );
                                                    if rng.gen_range(1..=150)
                                                        > player.stats.dexterity as i32
                                                    {
                                                        status_msg
                                                            .push_str(" You are off-balance.");
                                                        player.flags.paralysis +=
                                                            1 + rng.gen_range(1..=2);
                                                    }
                                                }
                                                player_acted = true;
                                            } else if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                match tile.tile_type {
                                                    TileType::DoorClosed { ref mut spikes } => {
                                                        let chance = player.stats.strength as i32
                                                            + (player.body_weight as i32 / 2);
                                                        let abs_spikes = (*spikes as i32).abs();
                                                        let roll = rng.gen_range(
                                                            0..=(chance * (20 + abs_spikes)).max(1),
                                                        );
                                                        let target =
                                                            (10 * (chance - abs_spikes)).max(1);

                                                        if roll < target {
                                                            tile.tile_type = TileType::DoorOpen;
                                                            status_msg = "The door crashes open!"
                                                                .to_string();
                                                            player.move_to(sx, sy);
                                                        } else {
                                                            if rng.gen_range(1..=150)
                                                                > player.stats.dexterity as i32
                                                            {
                                                                status_msg =
                                                                    "The door holds firm. You are off-balance."
                                                                        .to_string();
                                                                player.flags.paralysis +=
                                                                    1 + rng.gen_range(1..=2);
                                                            } else {
                                                                status_msg = "The door holds firm."
                                                                    .to_string();
                                                            }
                                                        }
                                                        player_acted = true;
                                                    }
                                                    TileType::Chest { trapped: _, locked } => {
                                                        if rng.gen_range(1..=10) == 1 {
                                                            tile.tile_type = TileType::Floor;
                                                            status_msg = "You have destroyed the chest and its contents!".to_string();
                                                        } else if locked
                                                            && rng.gen_range(1..=10) == 1
                                                        {
                                                            status_msg =
                                                                "The lock breaks open!".to_string();
                                                            level.open_chest(
                                                                sx,
                                                                sy,
                                                                &mut player,
                                                                &mut rng,
                                                                &mut status_msg,
                                                                &mut monsters,
                                                            );
                                                        } else {
                                                            status_msg =
                                                                "The chest holds firm.".to_string();
                                                        }
                                                        player_acted = true;
                                                    }
                                                    _ => {
                                                        status_msg =
                                                            "There is nothing there to bash."
                                                                .to_string();
                                                    }
                                                }
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectTunnelDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;

                                            let monster_idx = monsters
                                                .iter()
                                                .position(|m| m.x == sx && m.y == sy);
                                            if monster_idx.is_some() {
                                                status_msg =
                                                    "There is a monster in the way!".to_string();
                                            } else if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                let ability = player.digging_ability();
                                                match tile.tile_type {
                                                    TileType::Wall => {
                                                        if ability == 0 {
                                                            status_msg = "You dig with your hands, making no progress.".to_string();
                                                            player_acted = true;
                                                        } else {
                                                            let chance =
                                                                rng.gen_range(1..=1200) + 80;
                                                            if ability > chance {
                                                                tile.tile_type = TileType::Floor;
                                                                status_msg =
                                                                    "You have finished the tunnel."
                                                                        .to_string();
                                                            } else {
                                                                status_msg = "You tunnel into the granite wall.".to_string();
                                                            }
                                                            player_acted = true;
                                                        }
                                                    }
                                                    TileType::MagmaVein { gold } => {
                                                        if ability == 0 {
                                                            status_msg = "You dig with your hands, making no progress.".to_string();
                                                            player_acted = true;
                                                        } else {
                                                            let chance =
                                                                rng.gen_range(1..=600) + 10;
                                                            if ability > chance {
                                                                tile.tile_type = TileType::Floor;
                                                                status_msg =
                                                                    "You have finished the tunnel."
                                                                        .to_string();
                                                                if gold {
                                                                    let amount =
                                                                        rng.gen_range(50..=200);
                                                                    player.gold += amount;
                                                                    status_msg = format!(
                                                                        "You found a vein of gold! (Gained {} gp)",
                                                                        amount
                                                                    );
                                                                }
                                                            } else {
                                                                status_msg = "You tunnel into the magma intrusion.".to_string();
                                                            }
                                                            player_acted = true;
                                                        }
                                                    }
                                                    TileType::QuartzVein { gold } => {
                                                        if ability == 0 {
                                                            status_msg = "You dig with your hands, making no progress.".to_string();
                                                            player_acted = true;
                                                        } else {
                                                            let chance =
                                                                rng.gen_range(1..=400) + 10;
                                                            if ability > chance {
                                                                tile.tile_type = TileType::Floor;
                                                                status_msg =
                                                                    "You have finished the tunnel."
                                                                        .to_string();
                                                                if gold {
                                                                    let amount =
                                                                        rng.gen_range(50..=200);
                                                                    player.gold += amount;
                                                                    status_msg = format!(
                                                                        "You found a vein of gold! (Gained {} gp)",
                                                                        amount
                                                                    );
                                                                }
                                                            } else {
                                                                status_msg = "You tunnel into the quartz vein.".to_string();
                                                            }
                                                            player_acted = true;
                                                        }
                                                    }
                                                    TileType::Rubble => {
                                                        if ability == 0 {
                                                            status_msg = "You dig with your hands, making no progress.".to_string();
                                                            player_acted = true;
                                                        } else {
                                                            let chance = rng.gen_range(0..180);
                                                            if ability > chance {
                                                                tile.tile_type = TileType::Floor;
                                                                status_msg =
                                                                    "You have removed the rubble."
                                                                        .to_string();
                                                                if rng.gen_bool(0.10) {
                                                                    let item =
                                                                        generate_random_floor_item(
                                                                            level.depth,
                                                                            &mut rng,
                                                                        );
                                                                    status_msg = format!(
                                                                        "You removed the rubble and found a {} under it!",
                                                                        item.name
                                                                    );
                                                                    level.items.push(FloorItem {
                                                                        x: sx,
                                                                        y: sy,
                                                                        item,
                                                                    });
                                                                }
                                                            } else {
                                                                status_msg =
                                                                    "You dig in the rubble."
                                                                        .to_string();
                                                            }
                                                            player_acted = true;
                                                        }
                                                    }
                                                    TileType::SecretDoor => {
                                                        tile.tile_type =
                                                            TileType::DoorClosed { spikes: 0 };
                                                        status_msg = "You tunnel into the granite wall. You found a secret door!".to_string();
                                                        player_acted = true;
                                                    }
                                                    TileType::Floor | TileType::DoorOpen => {
                                                        status_msg =
                                                            "Tunnel through what? Empty air?!?"
                                                                .to_string();
                                                    }
                                                    _ => {
                                                        status_msg =
                                                            "You can't tunnel through that."
                                                                .to_string();
                                                    }
                                                }
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::DropMenu {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.inventory.len() {
                                            let mut dropped = player.inventory[idx].clone();
                                            dropped.count = 1;

                                            let name = player.inventory[idx].name.clone();
                                            player.inventory[idx].count -= 1;
                                            if player.inventory[idx].count == 0 {
                                                player.inventory.remove(idx);
                                            }

                                            level.items.push(FloorItem {
                                                x: player.x,
                                                y: player.y,
                                                item: dropped,
                                            });
                                            status_msg = format!("You dropped a {}.", name);
                                            player_acted = true;
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                    } else if screen_mode == ScreenMode::IdentifyMenu {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.inventory.len() {
                                            let target_name = player.inventory[idx].name.clone();
                                            player.identify_item_kind(&target_name);

                                            if let Some(scroll_pos) = player
                                                .inventory
                                                .iter()
                                                .position(|i| i.name.contains("Identify"))
                                            {
                                                player.inventory[scroll_pos].count -= 1;
                                                if player.inventory[scroll_pos].count == 0 {
                                                    player.inventory.remove(scroll_pos);
                                                }
                                            }
                                            player.identify_item_kind("Scroll of Identify");

                                            status_msg =
                                                format!("You identified the {}!", target_name);
                                            player_acted = true;
                                        } else {
                                            status_msg = "Invalid item choice.".to_string();
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                    } else if screen_mode == ScreenMode::SelectThrowItem {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.inventory.len() {
                                            selected_throw_inv_idx = idx;
                                            screen_mode = ScreenMode::SelectThrowDirection;
                                            status_msg = format!(
                                                "Throw {} in which direction? (q/w/e/a/d/z/x/c, ESC to cancel):",
                                                player.inventory[idx].name
                                            );
                                        } else {
                                            status_msg = "Invalid item choice.".to_string();
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectThrowDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let mut dx = match direction {
                                                Direction::NorthWest
                                                | Direction::West
                                                | Direction::SouthWest => -1,
                                                Direction::NorthEast
                                                | Direction::East
                                                | Direction::SouthEast => 1,
                                                _ => 0,
                                            };
                                            let mut dy = match direction {
                                                Direction::NorthWest
                                                | Direction::North
                                                | Direction::NorthEast => -1,
                                                Direction::SouthWest
                                                | Direction::South
                                                | Direction::SouthEast => 1,
                                                _ => 0,
                                            };

                                            if dx == 0 && dy == 0 {
                                                status_msg =
                                                    "Cannot throw at your own feet.".to_string();
                                            } else if selected_throw_inv_idx
                                                >= player.inventory.len()
                                            {
                                                status_msg =
                                                    "Item no longer in inventory.".to_string();
                                            } else {
                                                let confused_shot = player.flags.confused > 0;
                                                if confused_shot {
                                                    let random_dirs = [
                                                        (-1, -1),
                                                        (0, -1),
                                                        (1, -1),
                                                        (-1, 0),
                                                        (1, 0),
                                                        (-1, 1),
                                                        (0, 1),
                                                        (1, 1),
                                                    ];
                                                    let rdir = random_dirs
                                                        [rng.gen_range(0..random_dirs.len())];
                                                    dx = rdir.0;
                                                    dy = rdir.1;
                                                }

                                                // Extract 1 count of the item
                                                let mut thrown_item = player.inventory
                                                    [selected_throw_inv_idx]
                                                    .clone();
                                                thrown_item.count = 1;
                                                player.inventory[selected_throw_inv_idx].count -= 1;
                                                if player.inventory[selected_throw_inv_idx].count
                                                    == 0
                                                {
                                                    player.inventory.remove(selected_throw_inv_idx);
                                                }

                                                // Determine launcher synergy, range, damage, and to-hit bonus
                                                let launcher = player.equipment.iter().find(|i| {
                                                    matches!(i.item_type, ItemType::Bow { .. })
                                                });
                                                let (range, damage, to_hit_bonus) =
                                                    match &thrown_item.item_type {
                                                        ItemType::Missile {
                                                            damage: missile_dice,
                                                        } => {
                                                            let is_arrow = thrown_item
                                                                .name
                                                                .to_lowercase()
                                                                .contains("arrow");
                                                            let is_bolt = thrown_item
                                                                .name
                                                                .to_lowercase()
                                                                .contains("bolt");
                                                            let is_shot = thrown_item
                                                                .name
                                                                .to_lowercase()
                                                                .contains("shot")
                                                                || thrown_item
                                                                    .name
                                                                    .to_lowercase()
                                                                    .contains("pebble");

                                                            let matches_launcher = launcher
                                                                .map(|l| {
                                                                    let lname =
                                                                        l.name.to_lowercase();
                                                                    (is_arrow
                                                                        && lname.contains("bow")
                                                                        && !lname
                                                                            .contains("crossbow"))
                                                                        || (is_bolt
                                                                            && lname.contains(
                                                                                "crossbow",
                                                                            ))
                                                                        || (is_shot
                                                                            && lname
                                                                                .contains("sling"))
                                                                })
                                                                .unwrap_or(false);

                                                            if matches_launcher {
                                                                let mult = match launcher
                                                                    .unwrap()
                                                                    .item_type
                                                                {
                                                                    ItemType::Bow {
                                                                        multiplier,
                                                                    } => multiplier,
                                                                    _ => 1,
                                                                };
                                                                let rng_dist = match mult {
                                                                    2 => 25,
                                                                    3 => 30,
                                                                    _ => 35,
                                                                };
                                                                let dmg = missile_dice
                                                                    .roll(&mut rng)
                                                                    * mult;
                                                                let th = (player.stats.dexterity
                                                                    as i32
                                                                    - 10)
                                                                    * 3;
                                                                (rng_dist, dmg, th)
                                                            } else {
                                                                (10, missile_dice.roll(&mut rng), 0)
                                                            }
                                                        }
                                                        _ => {
                                                            let weight = thrown_item.weight.max(1);
                                                            let max_dist = (((player.stats.strength
                                                                as u32
                                                                + 20)
                                                                * 10)
                                                                / weight)
                                                                .clamp(1, 10)
                                                                as usize;
                                                            let dmg = if thrown_item
                                                                .name
                                                                .contains("Flask of Oil")
                                                            {
                                                                Dice::new(2, 6).roll(&mut rng)
                                                            } else if let ItemType::Weapon {
                                                                damage,
                                                            } = &thrown_item.item_type
                                                            {
                                                                damage.roll(&mut rng)
                                                            } else if thrown_item
                                                                .name
                                                                .contains("Spike")
                                                            {
                                                                Dice::new(1, 4).roll(&mut rng)
                                                            } else {
                                                                1
                                                            };
                                                            let th = (player.stats.strength as i32
                                                                - 10)
                                                                * 2;
                                                            (max_dist, dmg, th)
                                                        }
                                                    };

                                                let mut cx = player.x as isize;
                                                let mut cy = player.y as isize;
                                                let mut last_valid = (player.x, player.y);
                                                let mut hit_monster_idx: Option<usize> = None;
                                                let mut distance = 0;

                                                while distance < range {
                                                    cx += dx;
                                                    cy += dy;
                                                    distance += 1;

                                                    if cx <= 0
                                                        || cx >= (level.width - 1) as isize
                                                        || cy <= 0
                                                        || cy >= (level.height - 1) as isize
                                                    {
                                                        break;
                                                    }
                                                    let (ux, uy) = (cx as usize, cy as usize);
                                                    if let Some(tile) = level.get_tile(ux, uy)
                                                        && matches!(
                                                            tile.tile_type,
                                                            TileType::Wall
                                                                | TileType::SecretDoor
                                                                | TileType::DoorClosed { .. }
                                                                | TileType::MagmaVein { .. }
                                                                | TileType::QuartzVein { .. }
                                                                | TileType::Rubble
                                                        )
                                                    {
                                                        break;
                                                    }
                                                    last_valid = (ux, uy);

                                                    if let Some(m_idx) = monsters
                                                        .iter()
                                                        .position(|m| m.x == ux && m.y == uy)
                                                    {
                                                        hit_monster_idx = Some(m_idx);
                                                        break;
                                                    }
                                                }

                                                if let Some(m_idx) = hit_monster_idx {
                                                    let base_chance = 0.70
                                                        + (to_hit_bonus as f64 * 0.03)
                                                        - (distance as f64 * 0.02);
                                                    let hits =
                                                        rng.gen_bool(base_chance.clamp(0.20, 0.95));
                                                    if hits {
                                                        if thrown_item.name.contains("Flask of Oil")
                                                        {
                                                            status_msg = format!(
                                                                "The Flask of Oil bursts into flames! {} takes {} fire damage!",
                                                                monsters[m_idx].name, damage
                                                            );
                                                        } else {
                                                            status_msg = format!(
                                                                "The {} strikes {} for {} damage!",
                                                                thrown_item.display_name(),
                                                                monsters[m_idx].name,
                                                                damage
                                                            );
                                                        }

                                                        monsters[m_idx].was_attacked = true;
                                                        if monsters[m_idx]
                                                            .take_damage(damage as i32)
                                                        {
                                                            let exp_reward =
                                                                monsters[m_idx].experience_reward;
                                                            let m_lvl = monsters[m_idx].level;
                                                            let monster_name =
                                                                monsters[m_idx].name.clone();
                                                            let is_win = monsters[m_idx]
                                                                .is_win_creature()
                                                                || monster_name == "The Balrog";
                                                            monsters.remove(m_idx);
                                                            status_msg.push_str(&format!(
                                                                " You killed {}!",
                                                                monster_name
                                                            ));
                                                            let old_exp = player.exp;
                                                            let old_lvl = player.level;
                                                            let leveled_up = player
                                                                .gain_kill_experience(
                                                                    exp_reward, m_lvl,
                                                                );
                                                            let gained =
                                                                player.exp.saturating_sub(old_exp);
                                                            if leveled_up || player.level > old_lvl
                                                            {
                                                                status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                                            } else if gained > 0 {
                                                                status_msg.push_str(&format!(
                                                                    " Gained {} EXP.",
                                                                    gained
                                                                ));
                                                            }
                                                            if is_win {
                                                                player.balrog_killed = true;
                                                                player.total_winner = true;
                                                            }
                                                        }
                                                    } else {
                                                        status_msg = format!(
                                                            "The {} misses the {}.",
                                                            thrown_item.display_name(),
                                                            monsters[m_idx].name
                                                        );
                                                    }
                                                } else {
                                                    status_msg = format!(
                                                        "The {} flies through the air and lands.",
                                                        thrown_item.display_name()
                                                    );
                                                }

                                                if confused_shot {
                                                    status_msg = format!(
                                                        "You are confused! Your shot veers wildly! {}",
                                                        status_msg
                                                    );
                                                }

                                                let oil_exploded =
                                                    thrown_item.name.contains("Flask of Oil")
                                                        && hit_monster_idx.is_some();
                                                let breaks = oil_exploded || rng.gen_bool(0.10);
                                                if !breaks {
                                                    level.items.push(FloorItem {
                                                        x: last_valid.0,
                                                        y: last_valid.1,
                                                        item: thrown_item,
                                                    });
                                                } else if !oil_exploded {
                                                    status_msg
                                                        .push_str(" It is destroyed upon landing.");
                                                }

                                                player_acted = true;
                                            }
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                    } else if screen_mode == ScreenMode::AimWandMenu {
                                        let wands: Vec<(usize, &Item)> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(item.item_type, ItemType::Wand { .. })
                                            })
                                            .collect();
                                        let idx = c as usize - 'a' as usize;
                                        if idx < wands.len() {
                                            let (inv_idx, item) = wands[idx];
                                            if let ItemType::Wand { charges, .. } = item.item_type {
                                                if charges > 0 {
                                                    selected_wand_inv_idx = inv_idx;
                                                    screen_mode = ScreenMode::SelectWandDirection;
                                                    status_msg =
                                                        "Aim wand: choose direction...".to_string();
                                                } else {
                                                    status_msg = "That wand has no charges left!"
                                                        .to_string();
                                                    screen_mode = ScreenMode::Dungeon;
                                                }
                                            }
                                        } else {
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::SelectWandDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };

                                            let mut charges_left = 0;
                                            let mut s_idx = 0;
                                            if let ItemType::Wand {
                                                ref mut charges,
                                                spell_index,
                                            } = player.inventory[selected_wand_inv_idx].item_type
                                                && *charges > 0
                                            {
                                                *charges -= 1;
                                                charges_left = *charges;
                                                s_idx = spell_index;
                                            }

                                            let mut cx = player.x as isize + dx;
                                            let mut cy = player.y as isize + dy;
                                            status_msg =
                                                "Your wand beam fades into the dark!".to_string();

                                            while let Some(tile) =
                                                level.get_tile(cx as usize, cy as usize)
                                            {
                                                if tile.tile_type == TileType::Wall
                                                    || tile.tile_type == TileType::SecretDoor
                                                    || matches!(
                                                        tile.tile_type,
                                                        TileType::MagmaVein { .. }
                                                            | TileType::QuartzVein { .. }
                                                            | TileType::Rubble
                                                    )
                                                {
                                                    status_msg = "Your wand blast strikes a wall and dissipates!".to_string();
                                                    break;
                                                }
                                                if let Some(m_idx) = monsters.iter().position(|m| {
                                                    m.x == cx as usize && m.y == cy as usize
                                                }) {
                                                    let dmg = if s_idx == 0 {
                                                        rng.gen_range(2..=8)
                                                    } else {
                                                        rng.gen_range(4..=24)
                                                    };
                                                    let m_name = monsters[m_idx].name.clone();
                                                    let exp = monsters[m_idx].experience_reward;
                                                    let m_lvl = monsters[m_idx].level;
                                                    let is_win = monsters[m_idx].is_win_creature()
                                                        || m_name == "The Balrog";
                                                    status_msg = format!(
                                                        "Wand beam hits {} for {} damage!",
                                                        m_name, dmg
                                                    );
                                                    monsters[m_idx].was_attacked = true;
                                                    if monsters[m_idx].take_damage(dmg) {
                                                        status_msg.push_str(" You killed it!");
                                                        let old_exp = player.exp;
                                                        let old_lvl = player.level;
                                                        let leveled_up =
                                                            player.gain_kill_experience(exp, m_lvl);
                                                        let gained =
                                                            player.exp.saturating_sub(old_exp);
                                                        if leveled_up || player.level > old_lvl {
                                                            status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                                        } else if gained > 0 {
                                                            status_msg.push_str(&format!(
                                                                " Gained {} EXP.",
                                                                gained
                                                            ));
                                                        }
                                                        if level.depth == 0 {
                                                            player.killed_town_npcs += 1;
                                                        }
                                                        if is_win {
                                                            player.balrog_killed = true;
                                                            player.total_winner = true;
                                                        }
                                                        monsters.remove(m_idx);
                                                    }
                                                    break;
                                                }
                                                cx += dx;
                                                cy += dy;
                                            }

                                            status_msg.push_str(&format!(
                                                " ({} charges remaining)",
                                                charges_left
                                            ));
                                            screen_mode = ScreenMode::Dungeon;
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::UseStaffMenu {
                                        let staves: Vec<(usize, &Item)> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(item.item_type, ItemType::Staff { .. })
                                            })
                                            .collect();
                                        let idx = c as usize - 'a' as usize;
                                        if idx < staves.len() {
                                            let (inv_idx, _item) = staves[idx];
                                            let mut charges_left = 0;
                                            let mut p_idx = 0;
                                            let mut used_charge = false;
                                            if let ItemType::Staff {
                                                ref mut charges,
                                                prayer_index,
                                            } = player.inventory[inv_idx].item_type
                                                && *charges > 0
                                            {
                                                *charges -= 1;
                                                charges_left = *charges;
                                                p_idx = prayer_index;
                                                used_charge = true;
                                            }

                                            if used_charge {
                                                if p_idx == 1 {
                                                    let heal = rng.gen_range(2..=16);
                                                    player.hp =
                                                        (player.hp + heal).min(player.max_hp);
                                                    status_msg = format!(
                                                        "You use the staff. Restored {} HP. ({} charges remaining)",
                                                        heal, charges_left
                                                    );
                                                } else {
                                                    status_msg = format!(
                                                        "You use the staff, but nothing happens. ({} charges remaining)",
                                                        charges_left
                                                    );
                                                }
                                                player_acted = true;
                                            } else {
                                                status_msg =
                                                    "That staff has no charges left!".to_string();
                                            }
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                    } else if screen_mode == ScreenMode::StudyMenu {
                                        let idx = (c as u8).saturating_sub(b'a') as usize;
                                        let learnable = player.learnable_spells();
                                        if idx < learnable.len() {
                                            let spell_idx = learnable[idx];
                                            player.learn_spell(spell_idx);
                                            if let Some(def) = crate::magic::get_spell_def(
                                                player.class,
                                                spell_idx,
                                                player.is_priest_caster(),
                                            ) {
                                                status_msg = format!(
                                                    "You have learned the spell of {}.",
                                                    def.name
                                                );
                                            }
                                            if player.new_spells_to_learn == 0
                                                || player.learnable_spells().is_empty()
                                            {
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::CastSpellMenu {
                                        let idx = (c as u8).saturating_sub(b'a') as usize;
                                        let known = player.known_spells(false);
                                        if idx < known.len() {
                                            let spell_idx = known[idx];
                                            if let Some(def) = crate::magic::get_spell_def(
                                                player.class,
                                                spell_idx,
                                                false,
                                            ) {
                                                let fail_chance =
                                                    crate::magic::spell_chance_of_success(
                                                        player.level,
                                                        player.class,
                                                        player.stats.intelligence,
                                                        player.mana,
                                                        def,
                                                    );
                                                let failed = rng.gen_range(1..=100) <= fail_chance;
                                                if failed {
                                                    status_msg = "You failed to get the spell off!"
                                                        .to_string();
                                                    crate::magic::apply_spell_mana_and_fatigue(
                                                        &mut player,
                                                        def.mana_required,
                                                        false,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    screen_mode = ScreenMode::Dungeon;
                                                    player_acted = true;
                                                } else if crate::magic::is_directional_spell(
                                                    spell_idx, false,
                                                ) {
                                                    selected_spell_idx = spell_idx;
                                                    selected_spell_is_prayer = false;
                                                    screen_mode = ScreenMode::SelectSpellDirection;
                                                    status_msg = format!(
                                                        "Aim {}: choose direction (ESC to cancel)...",
                                                        def.name
                                                    );
                                                } else {
                                                    crate::magic::execute_non_directional_spell(
                                                        spell_idx,
                                                        false,
                                                        &mut player,
                                                        &mut level,
                                                        &mut monsters,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    if (player.spells_worked & (1 << spell_idx))
                                                        == 0
                                                    {
                                                        player.add_experience(def.exp_gain * 4);
                                                        player.spells_worked |= 1 << spell_idx;
                                                    }
                                                    crate::magic::apply_spell_mana_and_fatigue(
                                                        &mut player,
                                                        def.mana_required,
                                                        false,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    screen_mode = ScreenMode::Dungeon;
                                                    player_acted = true;
                                                }
                                            }
                                        }
                                    } else if screen_mode == ScreenMode::PrayMenu {
                                        let idx = (c as u8).saturating_sub(b'a') as usize;
                                        let known = player.known_spells(true);
                                        if idx < known.len() {
                                            let spell_idx = known[idx];
                                            if let Some(def) = crate::magic::get_spell_def(
                                                player.class,
                                                spell_idx,
                                                true,
                                            ) {
                                                let fail_chance =
                                                    crate::magic::spell_chance_of_success(
                                                        player.level,
                                                        player.class,
                                                        player.stats.wisdom,
                                                        player.mana,
                                                        def,
                                                    );
                                                let failed = rng.gen_range(1..=100) <= fail_chance;
                                                if failed {
                                                    status_msg =
                                                        "You lost your concentration!".to_string();
                                                    crate::magic::apply_spell_mana_and_fatigue(
                                                        &mut player,
                                                        def.mana_required,
                                                        true,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    screen_mode = ScreenMode::Dungeon;
                                                    player_acted = true;
                                                } else if crate::magic::is_directional_spell(
                                                    spell_idx, true,
                                                ) {
                                                    selected_spell_idx = spell_idx;
                                                    selected_spell_is_prayer = true;
                                                    screen_mode = ScreenMode::SelectSpellDirection;
                                                    status_msg = format!(
                                                        "Aim {}: choose direction (ESC to cancel)...",
                                                        def.name
                                                    );
                                                } else {
                                                    crate::magic::execute_non_directional_spell(
                                                        spell_idx,
                                                        true,
                                                        &mut player,
                                                        &mut level,
                                                        &mut monsters,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    if (player.spells_worked & (1 << spell_idx))
                                                        == 0
                                                    {
                                                        player.add_experience(def.exp_gain * 4);
                                                        player.spells_worked |= 1 << spell_idx;
                                                    }
                                                    crate::magic::apply_spell_mana_and_fatigue(
                                                        &mut player,
                                                        def.mana_required,
                                                        true,
                                                        &mut rng,
                                                        &mut status_msg,
                                                    );
                                                    screen_mode = ScreenMode::Dungeon;
                                                    player_acted = true;
                                                }
                                            }
                                        }
                                    } else if screen_mode == ScreenMode::SelectSpellDirection {
                                        let action = mapper.map_key(c);
                                        if let Action::Move(direction) = action {
                                            let (dx, dy) = match direction {
                                                Direction::NorthWest => (-1, -1),
                                                Direction::North => (0, -1),
                                                Direction::NorthEast => (1, -1),
                                                Direction::West => (-1, 0),
                                                Direction::Rest => (0, 0),
                                                Direction::East => (1, 0),
                                                Direction::SouthWest => (-1, 1),
                                                Direction::South => (0, 1),
                                                Direction::SouthEast => (1, 1),
                                            };
                                            if (dx != 0 || dy != 0)
                                                && let Some(def) = crate::magic::get_spell_def(
                                                    player.class,
                                                    selected_spell_idx,
                                                    selected_spell_is_prayer,
                                                )
                                            {
                                                crate::magic::execute_directional_spell(
                                                    selected_spell_idx,
                                                    selected_spell_is_prayer,
                                                    dx,
                                                    dy,
                                                    &mut player,
                                                    &mut level,
                                                    &mut monsters,
                                                    &mut rng,
                                                    &mut status_msg,
                                                );
                                                if (player.spells_worked
                                                    & (1 << selected_spell_idx))
                                                    == 0
                                                {
                                                    player.add_experience(def.exp_gain * 4);
                                                    player.spells_worked |= 1 << selected_spell_idx;
                                                }
                                                crate::magic::apply_spell_mana_and_fatigue(
                                                    &mut player,
                                                    def.mana_required,
                                                    selected_spell_is_prayer,
                                                    &mut rng,
                                                    &mut status_msg,
                                                );
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            }
                                        }
                                    } else if screen_mode == ScreenMode::Shop {
                                        let shop_info =
                                            level.shops.iter().find(|s| s.shop_type == active_shop);
                                        let owner_idx = shop_info
                                            .map(|s| s.owner_index)
                                            .unwrap_or(active_shop as usize);
                                        dungeon::handle_shop_buy_input(
                                            c,
                                            active_shop,
                                            owner_idx,
                                            &player,
                                            &mut screen_mode,
                                            &mut active_haggle,
                                            &mut status_msg,
                                            shop_info,
                                        );
                                    } else if screen_mode == ScreenMode::ShopSellMenu {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.inventory.len() {
                                            let item = &player.inventory[idx];
                                            let base_val = dungeon::get_item_base_value(item);
                                            let shop_info = level
                                                .shops
                                                .iter()
                                                .find(|s| s.shop_type == active_shop);
                                            let owner_idx = shop_info
                                                .map(|s| s.owner_index)
                                                .unwrap_or(active_shop as usize);
                                            let owner = &dungeon::STORE_OWNERS
                                                [owner_idx.min(dungeon::MAX_OWNERS - 1)];
                                            let (min_offer, max_offer) =
                                                dungeon::calculate_sell_price(
                                                    base_val,
                                                    owner,
                                                    player.race,
                                                    player.stats.charisma,
                                                );

                                            active_haggle = Some(HaggleState {
                                                item: item.clone(),
                                                item_index: idx,
                                                initial_price: min_offer,
                                                min_price: min_offer,
                                                max_price: max_offer,
                                                current_asking: min_offer,
                                                last_bid: None,
                                                insults: 0,
                                                offers_count: 0,
                                                comment: format!(
                                                    "I can offer you {} gp.",
                                                    min_offer
                                                ),
                                                player_input: String::new(),
                                                owner_index: owner_idx,
                                            });
                                            screen_mode = ScreenMode::BarterSellMenu;
                                            status_msg =
                                                format!("Bartering to sell {}.", item.name);
                                        }
                                    } else if screen_mode == ScreenMode::WearMenu {
                                        let idx = c as usize - 'a' as usize;
                                        let equippable_indices: Vec<usize> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(
                                                    item.item_type,
                                                    ItemType::Weapon { .. }
                                                        | ItemType::Bow { .. }
                                                        | ItemType::Armor { .. }
                                                        | ItemType::Light { .. }
                                                        | ItemType::Ring { .. }
                                                        | ItemType::Amulet { .. }
                                                )
                                            })
                                            .map(|(i, _)| i)
                                            .collect();

                                        if idx < equippable_indices.len() {
                                            let inv_idx = equippable_indices[idx];
                                            let item = &player.inventory[inv_idx];

                                            let target_slot = match &item.item_type {
                                                ItemType::Ring { .. } => {
                                                    let right_idx =
                                                        player.equipment.iter().position(|eq| {
                                                            eq.get_equipment_slot()
                                                                == "On right hand"
                                                        });
                                                    let left_idx =
                                                        player.equipment.iter().position(|eq| {
                                                            eq.get_equipment_slot()
                                                                == "On left hand"
                                                        });
                                                    if right_idx.is_none() {
                                                        "On right hand".to_string()
                                                    } else if left_idx.is_none() {
                                                        "On left hand".to_string()
                                                    } else {
                                                        let right_cursed = right_idx
                                                            .map(|i| {
                                                                player.equipment[i].is_cursed()
                                                            })
                                                            .unwrap_or(false);
                                                        let left_cursed = left_idx
                                                            .map(|i| {
                                                                player.equipment[i].is_cursed()
                                                            })
                                                            .unwrap_or(false);
                                                        if right_cursed && !left_cursed {
                                                            "On left hand".to_string()
                                                        } else {
                                                            "On right hand".to_string()
                                                        }
                                                    }
                                                }
                                                ItemType::Amulet { .. } => {
                                                    "Around neck".to_string()
                                                }
                                                _ => item.get_equipment_slot().to_string(),
                                            };

                                            let already_equipped =
                                                player.equipment.iter().position(|eq| {
                                                    eq.get_equipment_slot() == target_slot
                                                });

                                            if let Some(eq_idx) = already_equipped
                                                && player.equipment[eq_idx].is_cursed()
                                            {
                                                status_msg = format!(
                                                    "The {} you are wearing appears to be cursed.",
                                                    player.equipment[eq_idx].display_name()
                                                );
                                                screen_mode = ScreenMode::Dungeon;
                                                continue 'game_loop;
                                            }

                                            let mut item = player.inventory.remove(inv_idx);
                                            item.equipped_slot = Some(target_slot.clone());

                                            if let Some(eq_idx) = already_equipped {
                                                let mut old = player.equipment.remove(eq_idx);
                                                old.equipped_slot = None;
                                                status_msg = format!(
                                                    "You take off {} and equip {}.",
                                                    old.name, item.name
                                                );
                                                player.inventory.push(old);
                                            } else {
                                                status_msg =
                                                    format!("You wear/wield {}.", item.name);
                                            }

                                            player.equipment.push(item);
                                            player.update_equipment_bonuses();
                                            screen_mode = ScreenMode::Dungeon;
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::TakeOffMenu {
                                        let idx = c as usize - 'a' as usize;
                                        if idx < player.equipment.len() {
                                            if player.equipment[idx].is_cursed() {
                                                status_msg =
                                                    "Hmmm, it seems to be cursed.".to_string();
                                            } else if player.inventory.len() >= 22 {
                                                status_msg = "You can't carry it.".to_string();
                                            } else {
                                                let mut item = player.equipment.remove(idx);
                                                item.equipped_slot = None;
                                                status_msg = format!("You took off {}.", item.name);
                                                player.inventory.push(item);
                                                player.update_equipment_bonuses();
                                                player_acted = true;
                                            }
                                            screen_mode = ScreenMode::Dungeon;
                                        }
                                    } else if screen_mode == ScreenMode::QuaffMenu {
                                        let idx = c as usize - 'a' as usize;
                                        let potion_indices: Vec<usize> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(item.item_type, ItemType::Potion { .. })
                                            })
                                            .map(|(i, _)| i)
                                            .collect();

                                        if idx < potion_indices.len() {
                                            let inv_idx = potion_indices[idx];
                                            let potion_item = player.inventory[inv_idx].clone();
                                            player.inventory[inv_idx].count -= 1;
                                            if player.inventory[inv_idx].count == 0 {
                                                player.inventory.remove(inv_idx);
                                            }
                                            player.identify_item_kind(&potion_item.name);
                                            crate::magic::quaff_potion(
                                                &potion_item,
                                                &mut player,
                                                &mut level,
                                                &mut monsters,
                                                &mut rng,
                                                &mut status_msg,
                                            );
                                            screen_mode = ScreenMode::Dungeon;
                                            player_acted = true;
                                        }
                                    } else if screen_mode == ScreenMode::ReadMenu {
                                        let idx = c as usize - 'a' as usize;
                                        let scroll_indices: Vec<usize> = player
                                            .inventory
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, item)| {
                                                matches!(item.item_type, ItemType::Scroll { .. })
                                                    && !item.name.contains("Torch")
                                                    && !item.name.contains("Spellbook")
                                                    && !item.name.contains("Prayerbook")
                                            })
                                            .map(|(i, _)| i)
                                            .collect();

                                        if idx < scroll_indices.len() {
                                            let inv_idx = scroll_indices[idx];
                                            let scroll_item = player.inventory[inv_idx].clone();

                                            if scroll_item.name.contains("Identify") {
                                                screen_mode = ScreenMode::IdentifyMenu;
                                                status_msg = "Item you wish identified? Select a letter [a-z] or [ESC] to cancel...".to_string();
                                            } else {
                                                player.inventory[inv_idx].count -= 1;
                                                if player.inventory[inv_idx].count == 0 {
                                                    player.inventory.remove(inv_idx);
                                                }
                                                player.identify_item_kind(&scroll_item.name);
                                                crate::magic::read_scroll(
                                                    &scroll_item,
                                                    &mut player,
                                                    &mut level,
                                                    &mut monsters,
                                                    &mut rng,
                                                    &mut status_msg,
                                                );
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        if player_acted {
                            // Auto-searching
                            if player.searching {
                                let is_rogue = player.class == Class::Rogue;
                                let is_elf = matches!(player.race, Race::Elf | Race::HalfElf);
                                let trap_chance = if is_rogue { 0.70 } else { 0.30 };
                                let door_chance = if is_rogue {
                                    0.60
                                } else if is_elf {
                                    0.40
                                } else {
                                    0.20
                                };

                                let mut found_something = false;
                                for dy in -1..=1 {
                                    for dx in -1..=1 {
                                        if dx == 0 && dy == 0 {
                                            continue;
                                        }
                                        let sx = (player.x as isize + dx) as usize;
                                        let sy = (player.y as isize + dy) as usize;

                                        if let Some(tile) = level.get_tile_mut(sx, sy) {
                                            match tile.tile_type {
                                                TileType::SecretDoor => {
                                                    if rng.gen_bool(door_chance) {
                                                        tile.tile_type =
                                                            TileType::DoorClosed { spikes: 0 };
                                                        status_msg =
                                                            "You found a secret door!".to_string();
                                                        found_something = true;
                                                    }
                                                }
                                                TileType::Trap {
                                                    ref mut detected, ..
                                                } if !*detected && rng.gen_bool(trap_chance) => {
                                                    *detected = true;
                                                    status_msg = "You detected a trap!".to_string();
                                                    found_something = true;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                if found_something {
                                    player.searching = false;
                                }
                            }

                            game_turn += 1;
                            process_end_of_turn(
                                &mut player,
                                &mut level,
                                &mut monsters,
                                &mut status_msg,
                                &mut screen_mode,
                                &save_path,
                                active_shop,
                                active_haggle.as_ref(),
                                false,
                                game_turn,
                                max_depth,
                                &mut level_cache,
                            )?;
                            draw_map(
                                &mut level,
                                &player,
                                &monsters,
                                &status_msg,
                                screen_mode,
                                active_shop,
                                active_haggle.as_ref(),
                                active_inscribe.as_ref(),
                            )?;
                        } else {
                            draw_map(
                                &mut level,
                                &player,
                                &monsters,
                                &status_msg,
                                screen_mode,
                                active_shop,
                                active_haggle.as_ref(),
                                active_inscribe.as_ref(),
                            )?;
                        }
                    }
                    _ => {}
                }
            } else if is_resting {
                // Execute rest tick
                let player_acted = true;

                // HP/Mana regen (1 HP/Mana per resting turn)
                player.hp = (player.hp + 1).min(player.max_hp);
                player.mana = (player.mana + 1).min(player.max_mana);

                if let Some(ref mut n) = resting_turns {
                    *n -= 1;
                    if *n <= 0 {
                        resting_turns = None;
                        status_msg = "Rest finished.".to_string();
                    }
                }

                if resting_until_healed
                    && player.hp == player.max_hp
                    && player.mana == player.max_mana
                {
                    resting_until_healed = false;
                    status_msg = "Fully healed. Rest finished.".to_string();
                }

                if player.food < 300 {
                    resting_turns = None;
                    resting_until_healed = false;
                    if player.food < 0 {
                        status_msg = "You are starving! Rest interrupted.".to_string();
                    } else {
                        status_msg = "You are fainting from hunger! Rest interrupted.".to_string();
                    }
                }

                let has_visible_monster = monsters.iter().any(|m| {
                    if let Some(t) = level.get_tile(m.x, m.y) {
                        t.visible
                    } else {
                        false
                    }
                });
                if has_visible_monster {
                    resting_turns = None;
                    resting_until_healed = false;
                    status_msg = "Monster sighted! Rest interrupted.".to_string();
                }

                // Same digestion & monster updates for the rest tick
                if player_acted {
                    game_turn += 1;
                    process_end_of_turn(
                        &mut player,
                        &mut level,
                        &mut monsters,
                        &mut status_msg,
                        &mut screen_mode,
                        &save_path,
                        active_shop,
                        active_haggle.as_ref(),
                        true,
                        game_turn,
                        max_depth,
                        &mut level_cache,
                    )?;
                    draw_map(
                        &mut level,
                        &player,
                        &monsters,
                        &status_msg,
                        screen_mode,
                        active_shop,
                        active_haggle.as_ref(),
                        active_inscribe.as_ref(),
                    )?;
                }
            }
        }

        if restart_game {
            if !is_gui {
                let _ = execute!(
                    stdout,
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                );
            }
            continue 'outer_loop;
        }
        break 'outer_loop;
    }

    if !is_gui {
        _terminal_guard.active = false;
        execute!(stdout, Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
    }

    Ok(())
}
