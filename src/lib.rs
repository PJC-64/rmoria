mod input;
mod dungeon;
mod player;
mod save;
mod dice;
mod entity;
pub mod rendering;

pub use rendering::draw_map;

use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use std::sync::atomic::AtomicBool;
use std::sync::{OnceLock, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

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

pub fn init_gui_mode() -> (Sender<crossterm::event::Event>, Receiver<GuiState>, Sender<serde_json::Value>) {
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
use std::io::{self};
use std::fs;
use rand::Rng;
use input::{InputMapper, KeyboardProfile, Action, Direction};
use dungeon::{DungeonLevel, TileType, ShopType, FloorItem, tile::TrapType, generate_random_floor_item, get_shop_items, HaggleState, handle_haggle_input};
use player::{Player, Race, Class, Item, ItemType, InscribeState};
use save::GameState;
use dice::Dice;
use entity::monster::Monster;


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
    GameOver,
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
) -> Result<(), std::io::Error> {
    let mut rng = rand::thread_rng();

    // Light source fuel consumption
    if let Some(light_item) = player.equipment.iter_mut().find(|i| matches!(i.item_type, ItemType::Light { .. }))
        && let ItemType::Light { ref mut fuel } = light_item.item_type
            && *fuel > 0 {
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
        *screen_mode = ScreenMode::GameOver;
        *status_msg = format!("{} You have died from poison! Game Over. Press [r] to restart, or [q] to quit.", status_msg);
        draw_map(level, player, monsters, status_msg, *screen_mode, active_shop, active_haggle, None)?;
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
        *level = DungeonLevel::new(66, 22, new_depth, max_depth);
        let (px, py) = level.find_random_floor_tile();
        player.move_to(px, py);
        *monsters = level.generate_monsters(player.balrog_killed);
        level.update_fov(player);
    }

    // Food consumption, natural health/mana regeneration, and starvation
    let (hunger_msg, starved) = player.tick_digestion_and_regen(is_resting);
    if starved {
        if save_path.exists() {
            let _ = std::fs::remove_file(save_path);
        }
        *screen_mode = ScreenMode::GameOver;
        *status_msg = format!("{} You have starved to death! Game Over. Press [r] to restart, or [q] to quit.", status_msg);
        draw_map(level, player, monsters, status_msg, *screen_mode, active_shop, active_haggle, None)?;
        return Ok(());
    } else if let Some(msg) = hunger_msg
        && !status_msg.contains(msg.trim()) {
            status_msg.push_str(msg);
        }

    // Track max depth visited
    player.max_depth_reached = player.max_depth_reached.max(level.depth);

    let occupied_positions: Vec<(usize, usize)> = monsters.iter().map(|m| (m.x, m.y)).collect();

    for monster in monsters.iter_mut() {
        if monster.stunned > 0 {
            monster.stunned -= 1;
            continue;
        }
        let dx = (player.x as isize - monster.x as isize).abs();
        let dy = (player.y as isize - monster.y as isize).abs();
        
        let is_hostile = level.depth > 0 || monster.was_attacked || player.killed_town_npcs > player.town_npc_attack_threshold;
        let mut did_attack = false;
        if dx <= 1 && dy <= 1 && is_hostile {
            let m_damage = monster.damage.roll(&mut rng) as i32;
            player.hp -= m_damage;
            player.searching = false;
            status_msg.push_str(&format!(" {} hits you for {}!", monster.name, m_damage));

            // Special monster status afflictions:
            let m_name = monster.name.to_lowercase();
            if (m_name.contains("spider") || m_name.contains("snake") || m_name.contains("viper") || m_name.contains("scorpion")) && rng.gen_bool(0.35) {
                player.flags.poisoned += rng.gen_range(10..=25);
                status_msg.push_str(" You have been poisoned!");
            } else if (m_name.contains("ghost") || m_name.contains("phantom") || m_name.contains("wraith") || m_name.contains("banshee")) && rng.gen_bool(0.35) {
                if player.flags.heroism == 0 && player.flags.super_heroism == 0 {
                    player.flags.afraid += rng.gen_range(5..=15);
                    status_msg.push_str(" You are terrified!");
                }
            } else if (m_name.contains("ghoul") || m_name.contains("crawler") || m_name.contains("lich")) && rng.gen_bool(0.25) {
                if player.flags.free_action {
                    status_msg.push_str(" You resist the paralysis!");
                } else {
                    player.flags.paralysis += rng.gen_range(2..=5);
                    status_msg.push_str(" You are paralyzed!");
                }
            } else if (m_name.contains("gazer") || m_name.contains("umber")) && rng.gen_bool(0.30) {
                player.flags.confused += rng.gen_range(5..=15);
                status_msg.push_str(" You feel confused!");
            } else if m_name.contains("eye") && rng.gen_bool(0.25) {
                player.flags.blind += rng.gen_range(10..=20);
                status_msg.push_str(" You are blinded!");
            } else if (m_name.contains("shadow") || m_name.contains("vampire") || m_name.contains("spectre")) && rng.gen_bool(0.25) {
                let stat_to_drain = rng.gen_range(0..6);
                if player.drain_stat(stat_to_drain, &mut rng) {
                    let stat_name = match stat_to_drain {
                        0 => "strength",
                        1 => "intelligence",
                        2 => "wisdom",
                        3 => "dexterity",
                        4 => "constitution",
                        _ => "charisma",
                    };
                    status_msg.push_str(&format!(" Your {} was drained!", stat_name));
                } else if player.is_stat_sustained(stat_to_drain) {
                    status_msg.push_str(" Your stats were sustained!");
                }
            }

            if player.hp <= 0 {
                player.hp = 0;
                if save_path.exists() {
                    let _ = std::fs::remove_file(save_path);
                }
                *screen_mode = ScreenMode::GameOver;
                status_msg.push_str(" You have died! Game Over. Press [r] to restart, or [q] to quit.");
                break;
            }
            did_attack = true;
        }
        if !did_attack
            && let Some((mx, my)) = monster.update_ai(player.x, player.y, level, level.depth, player.killed_town_npcs, player.town_npc_attack_threshold) {
                let occupied_by_player = mx == player.x && my == player.y;
                let occupied_by_monster = occupied_positions.iter().any(|&(ox, oy)| ox == mx && oy == my);
                
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
                            && let TileType::DoorClosed { ref mut spikes } = t.tile_type {
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

                        let dist = (((monster.x as isize - player.x as isize).pow(2) + (monster.y as isize - player.y as isize).pow(2)) as f64).sqrt();
                        if dist <= 10.0 {
                            if broke_spike {
                                status_msg.push_str(" You hear a door spike snap!");
                            } else if opened {
                                status_msg.push_str(&format!(" The {} opens a door.", monster.name));
                            } else {
                                status_msg.push_str(" You hear a door being bashed!");
                            }
                        }

                        if opened {
                            monster.x = mx;
                            monster.y = my;
                        }
                    } else {
                        monster.x = mx;
                        monster.y = my;
                    }
                }
            }
    }
    
    Ok(())
}

#[allow(deprecated)]
pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = get_save_path();
    let mut rng = rand::thread_rng();

    let args: Vec<String> = std::env::args().collect();
    let is_gui = args.iter().any(|arg| arg == "--gui") || GUI_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);

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
            #[allow(deprecated)]
            let server = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
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
                        if ws.write_message(tungstenite::Message::Text(update_msg.to_string())).is_err() {
                            break;
                        }
                    }

                    // Read incoming message
                    let msg = match ws.read_message() {
                        Ok(m) => m,
                        Err(e) => {
                            if e.to_string().contains("WouldBlock") || e.to_string().contains("TimedOut") {
                                continue;
                            }
                            break;
                        }
                    };

                    if let tungstenite::Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if val["type"] == "load" || val["type"] == "create_character" || val["type"] == "roll_character" || val["type"] == "accept_character" {
                                if let Some(tx_mutex) = GUI_SETUP_TX.get() {
                                    let tx = tx_mutex.lock().unwrap();
                                    let _ = tx.send(val);
                                }
                            } else if val["type"] == "activate_cheat" {
                                activate_cheat_global();
                                let dummy = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                    crossterm::event::KeyCode::Char(' '),
                                    crossterm::event::KeyModifiers::empty(),
                                ));
                                let _ = input_tx.send(dummy);
                            } else if val["type"] == "key"
                                && let Some(key_str) = val["key"].as_str() {
                                    let event = match key_str {
                                        "\x1b" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                            crossterm::event::KeyCode::Esc,
                                            crossterm::event::KeyModifiers::empty(),
                                        )),
                                        "\n" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                            crossterm::event::KeyCode::Enter,
                                            crossterm::event::KeyModifiers::empty(),
                                        )),
                                        "\u{0008}" => crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
                                            crossterm::event::KeyCode::Backspace,
                                            crossterm::event::KeyModifiers::empty(),
                                        )),
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
        execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
    }

    'outer_loop: loop {
        let mut restart_game = false;
        let (mut player, mut level, mut monsters, mut status_msg, max_depth) = if is_gui {
        if let Some(rx_mutex) = GUI_SETUP_RX.get() {
            let rx = rx_mutex.lock().unwrap();
            let state_tx = crate::GUI_STATE_TX.get().unwrap().lock().unwrap().clone();
            println!("Waiting for client initialization choice...");
            let mut val = rx.recv()?;
            loop {
                if val["type"] == "load" {
                    let state = GameState::load_from_file(&save_path)?;
                    break (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string(), state.max_depth);
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
                    let stats = player::Attributes::new(s_str as i16, s_int as i16, s_wis as i16, s_dex as i16, s_con as i16, s_chr as i16);

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
                    let stats = player::Attributes::new(s_str, s_int, s_wis, s_dex, s_con, s_chr);

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

                    break (final_player, lvl, mons, "Welcome to rmoria! Find town shops or descend stairs (>)".to_string(), rolled_max);
                } else {
                    val = rx.recv()?;
                }
            }
        } else {
            return Err("GUI Setup channel not initialized".into());
        }
    } else if save_path.exists() {
        match GameState::load_from_file(&save_path) {
            Ok(state) => {
                (state.player, state.level, state.monsters, "Save game loaded successfully!".to_string(), state.max_depth)
            }
            Err(e) => {
                let rolled_max = rng.gen_range(8..=15);
                let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
                lvl.generate_simple_floor();
                let p = player::run_character_creation(&mut stdout, &mut rng, None)?;
                let start_pos = lvl.find_random_floor_tile();
                let mut player = p;
                player.move_to(start_pos.0, start_pos.1);
                let mons = lvl.generate_monsters(false);
                (player, lvl, mons, format!("Failed to load save: {}. Started new game.", e), rolled_max)
            }
        }
    } else {
        let rolled_max = rng.gen_range(8..=15);
        let mut lvl = DungeonLevel::new(66, 22, 0, rolled_max);
        lvl.generate_simple_floor();
        let p = player::run_character_creation(&mut stdout, &mut rng, None)?;
        let start_pos = lvl.find_random_floor_tile();
        let mut player = p;
        player.move_to(start_pos.0, start_pos.1);
        let mons = lvl.generate_monsters(false);
        (player, lvl, mons, "Welcome to rmoria! Find town shops or descend stairs (>)".to_string(), rolled_max)
    };

    let mut screen_mode = ScreenMode::Dungeon;
    let mut active_shop = ShopType::General;
    let mut selected_spell_idx: usize = 0;
    let mut selected_wand_inv_idx: usize = 0;
    let mut active_haggle: Option<HaggleState> = None;
    let mut active_inscribe: Option<InscribeState> = None;
    let mut resting_turns: Option<i32> = None;
    let mut resting_until_healed = false;
    let mut rest_input_buffer = String::new();
    let mut game_turn: u64 = 0;

    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;

    let mapper = InputMapper::new(KeyboardProfile::StandardQweasd);

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
                Item::new("Mage Spellbook [Beginner's Magick]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Mage Spellbook [Magic II - Incantations]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Mage Spellbook [Magic III - Sorcery]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Mage Spellbook [Master's Arcana]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Priest Prayerbook [Beginner's Handbook]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Priest Prayerbook [Words of Wisdom]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Priest Prayerbook [Chants and Prayers]", 1, 20, ItemType::Scroll { teleport: false }),
                Item::new("Priest Prayerbook [Holy Teachings]", 1, 20, ItemType::Scroll { teleport: false }),
            ];
            for book in books {
                if !player.inventory.iter().any(|i| i.name == book.name) {
                    player.inventory.push(book);
                }
            }

            status_msg = "TEST MODE ENABLED: ALL SPELLBOOKS & IMMORTALITY GRANTED!".to_string();
            let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
            draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
        }

        if player.is_wizard {
            player.hp = player.max_hp;
            player.mana = player.max_mana;
            player.food = 15000;
        }

        if screen_mode != ScreenMode::GameOver && player.flags.paralysis > 0 {
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
            )?;
            draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
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
                    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                }
                Event::Key(key_event) => {
                    if is_resting {
                        resting_turns = None;
                        resting_until_healed = false;
                        status_msg = "Rest interrupted by keypress.".to_string();
                        let _ = event::read()?;
                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                        continue 'game_loop;
                    }

                    if key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        break 'game_loop;
                    }

                    let mut player_acted = false;

                    if screen_mode == ScreenMode::SelectRestCount {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                if c.is_ascii_digit() || c == '*' {
                                    rest_input_buffer.push(c);
                                    status_msg = format!("Rest how long? (number of turns, or * for until healed): {}", rest_input_buffer);
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                }
                            }
                            KeyCode::Backspace => {
                                rest_input_buffer.pop();
                                status_msg = format!("Rest how long? (number of turns, or * for until healed): {}", rest_input_buffer);
                                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                            }
                            KeyCode::Enter => {
                                if rest_input_buffer == "*" {
                                    resting_until_healed = true;
                                    resting_turns = None;
                                    status_msg = "Resting...".to_string();
                                    screen_mode = ScreenMode::Dungeon;
                                } else if let Ok(n) = rest_input_buffer.parse::<i32>()
                                    && n > 0 {
                                        resting_turns = Some(n);
                                        resting_until_healed = false;
                                        status_msg = "Resting...".to_string();
                                        screen_mode = ScreenMode::Dungeon;
                                    }
                            }
                            KeyCode::Esc => {
                                status_msg = "Cancelled.".to_string();
                                screen_mode = ScreenMode::Dungeon;
                                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                            }
                            _ => {}
                        }
                    } else if screen_mode == ScreenMode::Inscribing {
                        if let Some(ref mut state) = active_inscribe {
                            match key_event.code {
                                KeyCode::Char(c) => {
                                    if state.input.len() < 30 {
                                        state.input.push(c);
                                        status_msg = format!("Inscribing... type characters and press ENTER: {}", state.input);
                                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                    }
                                }
                                KeyCode::Backspace => {
                                    state.input.pop();
                                    status_msg = format!("Inscribing... type characters and press ENTER: {}", state.input);
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                }
                                KeyCode::Enter => {
                                    let item_idx = state.item_index;
                                    let is_eq = state.is_equipment;
                                    let ins_text = state.input.trim().to_string();
                                    let ins_val = if ins_text.is_empty() { None } else { Some(ins_text) };
                                    
                                    if is_eq {
                                        if item_idx < player.equipment.len() {
                                            player.equipment[item_idx].inscription = ins_val.clone();
                                        }
                                    } else {
                                        if item_idx < player.inventory.len() {
                                            player.inventory[item_idx].inscription = ins_val.clone();
                                        }
                                    }
                                    
                                    status_msg = if let Some(txt) = ins_val {
                                        format!("Inscribed item with: {}", txt)
                                    } else {
                                        "Cleared inscription.".to_string()
                                    };
                                    
                                    active_inscribe = None;
                                    screen_mode = ScreenMode::Dungeon;
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                }
                                KeyCode::Esc => {
                                    status_msg = "Cancelled.".to_string();
                                    active_inscribe = None;
                                    screen_mode = ScreenMode::Dungeon;
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                }
                                _ => {}
                            }
                        }
                    } else if screen_mode == ScreenMode::Dungeon {
                        if let KeyCode::Char(c) = key_event.code {
                            let action = mapper.map_key(c);
                            match action {
                                Action::Move(mut direction) => {
                                    if player.flags.confused > 0 && direction != Direction::Rest && rng.gen_bool(0.75) {
                                        let random_dirs = [
                                            Direction::NorthWest, Direction::North, Direction::NorthEast,
                                            Direction::West, Direction::East,
                                            Direction::SouthWest, Direction::South, Direction::SouthEast,
                                        ];
                                        direction = random_dirs[rng.gen_range(0..random_dirs.len())];
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
                                        && let Some(shop_info) = level.shops.iter().find(|s| s.door_x == next_x && s.door_y == next_y) {
                                            screen_mode = ScreenMode::Shop;
                                            active_shop = shop_info.shop_type;
                                            let shop_name = match active_shop {
                                                ShopType::General => "Town General Store",
                                                ShopType::Armory => "Town Armory",
                                                ShopType::Weaponsmith => "Weaponsmith Forge",
                                                ShopType::Temple => "Town Temple",
                                                ShopType::Alchemy => "Alchemy Lab",
                                                ShopType::Magic => "Magic-User Conclave",
                                            };
                                            status_msg = format!("Welcome to the {}!", shop_name);
                                            entered_shop = true;
                                        }

                                    if entered_shop {
                                        // Shop entered
                                    }
                                    else if let Some(m_idx) = monsters.iter().position(|m| m.x == next_x && m.y == next_y) {
                                        let is_m_visible = level.get_tile(next_x, next_y).map(|t| t.visible).unwrap_or(false);
                                        
                                        if is_m_visible {
                                            if player.flags.afraid > 0 {
                                                status_msg = "You are too afraid!".to_string();
                                                player_acted = true;
                                            } else {
                                                let damage = player.roll_melee_damage(&mut rng);
                                                status_msg = format!("You hit {} for {} damage!", monsters[m_idx].name, damage);
                                            
                                            monsters[m_idx].was_attacked = true;
                                            let monster_name = monsters[m_idx].name.clone();
                                            let exp_reward = monsters[m_idx].experience_reward;

                                            if monsters[m_idx].take_damage(damage) {
                                                if level.depth == 0 {
                                                    player.killed_town_npcs += 1;
                                                }
                                                status_msg.push_str(&format!(" You killed {}!", monster_name));
                                                
                                                if player.add_experience(exp_reward) {
                                                    status_msg.push_str(&format!(" Congratulations! You reached level {}.", player.level));
                                                } else if exp_reward > 0 {
                                                    status_msg.push_str(&format!(" Gained {} EXP.", exp_reward));
                                                } else {
                                                    status_msg.push_str(" Gained 0 EXP (Town monster).");
                                                }

                                                let is_boss = monster_name == "The Balrog" || monster_name.contains("Lich");
                                                if is_boss || rng.gen_bool(0.30) {
                                                    let drop_item = if monster_name == "The Balrog" {
                                                        Item::new("Broadsword of Slaying", 1, 150, ItemType::Weapon { damage: Dice::new(3, 6) })
                                                    } else {
                                                        generate_random_floor_item(level.depth, &mut rng)
                                                    };
                                                    level.items.push(FloorItem { x: next_x, y: next_y, item: drop_item.clone() });
                                                    status_msg.push_str(&format!(" It dropped a {}!", drop_item.name));
                                                }
                                                
                                                if monster_name == "The Balrog" {
                                                    player.balrog_killed = true;
                                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                                    std::thread::sleep(std::time::Duration::from_secs(1));
                                                    
                                                    execute!(stdout, Show, LeaveAlternateScreen)?;
                                                    disable_raw_mode()?;
                                                    
                                                    println!("============================================================");
                                                    println!("         CONGRATULATIONS! YOU HAVE SLAIN THE BALROG!        ");
                                                    println!("        You have completed the quest and won rmoria!        ");
                                                    println!("============================================================");
                                                    
                                                    if save_path.exists() {
                                                        let _ = fs::remove_file(&save_path);
                                                    }
                                                    break 'game_loop;
                                                }
                                                monsters.remove(m_idx);
                                            }
                                            player_acted = true;
                                            }
                                        } else {
                                            if let Some(tile) = level.get_tile(next_x, next_y)
                                                && tile.is_passable() {
                                                    player.move_to(next_x, next_y);
                                                    status_msg = "You step into the darkness and bump into a monster!".to_string();
                                                    player_acted = true;
                                                }
                                        }
                                    } else {
                                        if let Some(tile) = level.get_tile(next_x, next_y) {
                                            if tile.is_passable() || direction == Direction::Rest {
                                                let mut trigger_trap = None;
                                                if let TileType::Trap { trap_type, .. } = tile.tile_type {
                                                    trigger_trap = Some(trap_type);
                                                }

                                                player.move_to(next_x, next_y);
                                                
                                                if direction == Direction::Rest {
                                                    if player.food < 0 {
                                                        status_msg = "You are starving and cannot rest.".to_string();
                                                    } else if player.food < 300 {
                                                        status_msg = "You are too faint to rest effectively.".to_string();
                                                    } else {
                                                        player.hp = (player.hp + 1).min(player.max_hp);
                                                        player.mana = (player.mana + 1).min(player.max_mana);
                                                        status_msg = "You rest and recover health/mana.".to_string();
                                                    }
                                                } else {
                                                    status_msg = format!("Moved to ({}, {})", next_x, next_y);
                                                }

                                                if let Some(tt) = trigger_trap {
                                                    match tt {
                                                        TrapType::Arrow => {
                                                            let dmg = rng.gen_range(1..=6);
                                                            player.hp -= dmg;
                                                            status_msg = format!("Click! You set off an arrow trap! You take {} damage!", dmg);
                                                        }
                                                        TrapType::PoisonGas => {
                                                            let dmg = rng.gen_range(2..=8);
                                                            player.hp -= dmg;
                                                            player.flags.poisoned += rng.gen_range(10..=30);
                                                            status_msg = format!("Click! Poison gas fills the corridor! You take {} damage and are poisoned!", dmg);
                                                        }
                                                        TrapType::Teleport => {
                                                            let dest = level.find_random_floor_tile();
                                                            player.move_to(dest.0, dest.1);
                                                            status_msg = "Click! A teleport trap warps you to another location!".to_string();
                                                        }
                                                        TrapType::SleepingGas => {
                                                            if player.flags.free_action {
                                                                status_msg = "Click! A strange white mist surrounds you! You are unaffected.".to_string();
                                                            } else {
                                                                player.flags.paralysis += rng.gen_range(1..=10) + 4;
                                                                status_msg = "Click! A strange white mist surrounds you! You fall asleep.".to_string();
                                                            }
                                                        }
                                                        TrapType::BlindGas => {
                                                            player.flags.blind += rng.gen_range(1..=50) + 50;
                                                            status_msg = "Click! Black gas surrounds you! You are blinded!".to_string();
                                                        }
                                                        TrapType::ConfusionGas => {
                                                            player.flags.confused += rng.gen_range(1..=15) + 15;
                                                            status_msg = "Click! Gas of scintillating colors surrounds you! You are confused!".to_string();
                                                        }
                                                    }
                                                    if let Some(t) = level.get_tile_mut(player.x, player.y) {
                                                        t.tile_type = TileType::Floor;
                                                    }
                                                }

                                                let item_idx = level.items.iter().position(|i| i.x == player.x && i.y == player.y);
                                                if let Some(idx) = item_idx {
                                                    let floor_item = level.items.remove(idx);
                                                    let item = floor_item.item;
                                                    if item.name.contains("Gold Pile") {
                                                        let gold_amount = item.name.split('[')
                                                            .nth(1)
                                                            .and_then(|s| s.split(' ').next())
                                                            .and_then(|s| s.parse::<u32>().ok())
                                                            .unwrap_or(20);
                                                        player.gold += gold_amount;
                                                        status_msg = format!("You picked up {} gold pieces.", gold_amount);
                                                    } else {
                                                        status_msg = format!("You picked up a {}.", item.name);
                                                        player.add_item_to_inventory(item);
                                                    }
                                                }

                                                player_acted = true;
                                            } else {
                                                status_msg = "Ouch! You bumped into a wall.".to_string();
                                            }
                                        }
                                    }
                                }
                                Action::GoUpStairs => {
                                    if let Some(tile) = level.get_tile(player.x, player.y) {
                                        if tile.tile_type == TileType::StairsUp && level.depth > 0 {
                                            let new_depth = level.depth - 1;
                                            level = DungeonLevel::new(66, 22, new_depth, max_depth);
                                            level.generate_simple_floor();
                                            monsters = level.generate_monsters(player.balrog_killed);
                                            
                                            let sd_pos = level.tiles.iter().enumerate()
                                                .find(|(_, t)| t.tile_type == TileType::StairsDown)
                                                .map(|(i, _)| (i % level.width, i / level.width))
                                                .unwrap_or((40, 15));
                                            player.move_to(sd_pos.0, sd_pos.1);
                                            
                                            status_msg = format!("You climb up to depth {} ({} feet).", new_depth, new_depth * 50);
                                            player_acted = true;
                                        } else {
                                            status_msg = "You see no up stairs (<) here.".to_string();
                                        }
                                    }
                                }
                                Action::GoDownStairs => {
                                    if let Some(tile) = level.get_tile(player.x, player.y) {
                                        if tile.tile_type == TileType::StairsDown && level.depth < max_depth {
                                            let new_depth = level.depth + 1;
                                            level = DungeonLevel::new(66, 22, new_depth, max_depth);
                                            level.generate_simple_floor();
                                            monsters = level.generate_monsters(player.balrog_killed);
                                            
                                            let su_pos = level.tiles.iter().enumerate()
                                                .find(|(_, t)| t.tile_type == TileType::StairsUp)
                                                .map(|(i, _)| (i % level.width, i / level.width))
                                                .unwrap_or((15, 15));
                                            player.move_to(su_pos.0, su_pos.1);
                                            
                                            status_msg = format!("You climb down to depth {} ({} feet).", new_depth, new_depth * 50);
                                            player_acted = true;
                                        } else {
                                            status_msg = "You see no down stairs (>) here.".to_string();
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
                                Action::Eat => {
                                     let has_food = player.inventory.iter().any(|i| matches!(i.item_type, ItemType::Food { .. }));
                                     if has_food {
                                         screen_mode = ScreenMode::EatMenu;
                                         status_msg = "Select food to eat.".to_string();
                                     } else {
                                         status_msg = "You do not carry any food!".to_string();
                                     }
                                 }
                                 Action::Rest => {
                                     if player.food < 0 {
                                         status_msg = "You are starving and cannot rest!".to_string();
                                     } else if player.food < 300 {
                                         status_msg = "You are too faint from hunger to rest!".to_string();
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
                                     let has_refillable_light = player.equipment.iter().any(|i| {
                                         matches!(i.item_type, ItemType::Light { .. }) && (i.name.contains("Lantern") || i.name.contains("Lamp"))
                                     });
                                     if !has_refillable_light {
                                         let is_wielding_torch = player.equipment.iter().any(|i| {
                                             matches!(i.item_type, ItemType::Light { .. }) && i.name.contains("Torch")
                                         });
                                         if is_wielding_torch {
                                             status_msg = "You cannot refill a torch!".to_string();
                                         } else {
                                             status_msg = "You are not wielding a lamp or lantern.".to_string();
                                         }
                                     } else {
                                         let oil_position = player.inventory.iter().position(|i| i.name.contains("Flask of Oil"));
                                         match oil_position {
                                             None => {
                                                 status_msg = "You have no oil.".to_string();
                                             }
                                             Some(idx) => {
                                                 // We have oil and a refillable light source equipped
                                                 if let Some(light_item) = player.equipment.iter_mut().find(|i| {
                                                     matches!(i.item_type, ItemType::Light { .. }) && (i.name.contains("Lantern") || i.name.contains("Lamp"))
                                                 })
                                                     && let ItemType::Light { ref mut fuel } = light_item.item_type {
                                                         if *fuel >= 7500 {
                                                             *fuel = 7500;
                                                             status_msg = "Your lamp overflows, spilling oil on the ground.".to_string();
                                                         } else {
                                                             *fuel = (*fuel + 7500).min(7500);
                                                             status_msg = "You refill your lamp.".to_string();
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
                                        status_msg = "You can't see to read the scroll.".to_string();
                                    } else if player.flags.confused > 0 {
                                        status_msg = "You are too confused to read a scroll.".to_string();
                                    } else {
                                        screen_mode = ScreenMode::ReadMenu;
                                        status_msg = "Select scroll to read.".to_string();
                                    }
                                }
                                Action::BrowseBook => {
                                    if player.flags.blind > 0 {
                                        status_msg = "You can't see to read your book!".to_string();
                                    } else {
                                        let has_book = player.is_wizard || player.inventory.iter().any(|i| i.name.contains("Spellbook") || i.name.contains("Prayerbook"));
                                        if has_book {
                                            screen_mode = ScreenMode::BrowseBookMenu;
                                            status_msg = "Browsing spells/prayers.".to_string();
                                        } else {
                                            status_msg = "You do not carry a spellbook or prayerbook!".to_string();
                                        }
                                    }
                                }
                                Action::CastSpell => {
                                    if player.flags.blind > 0 {
                                        status_msg = "You can't see to read your spell book!".to_string();
                                    } else if player.flags.confused > 0 {
                                        status_msg = "You are too confused.".to_string();
                                    } else {
                                        let is_mage_caster = player.is_wizard || matches!(player.class, Class::Mage | Class::Rogue | Class::Ranger);
                                        if !is_mage_caster {
                                            status_msg = "Your class cannot cast mage spells!".to_string();
                                        } else {
                                            let has_book = player.is_wizard || player.inventory.iter().any(|i| i.name.contains("Mage Spellbook"));
                                            if has_book {
                                                screen_mode = ScreenMode::CastSpellMenu;
                                                status_msg = "Cast Mage Spell: select a letter.".to_string();
                                            } else {
                                                status_msg = "You need a Mage Spellbook to cast spells!".to_string();
                                            }
                                        }
                                    }
                                }
                                Action::Pray => {
                                    if player.flags.blind > 0 {
                                        status_msg = "You can't see to read your prayer!".to_string();
                                    } else if player.flags.confused > 0 {
                                        status_msg = "You are too confused.".to_string();
                                    } else {
                                        let is_priest_caster = player.is_wizard || matches!(player.class, Class::Priest | Class::Paladin);
                                        if !is_priest_caster {
                                            status_msg = "Your class cannot recite priestly prayers!".to_string();
                                        } else {
                                            let has_book = player.is_wizard || player.inventory.iter().any(|i| i.name.contains("Priest Prayerbook"));
                                            if has_book {
                                                screen_mode = ScreenMode::PrayMenu;
                                                status_msg = "Recite Clerical Prayer: select a letter.".to_string();
                                            } else {
                                                status_msg = "You need a Priest Prayerbook to pray!".to_string();
                                            }
                                        }
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
                                        status_msg = "You have nothing to inscribe!".to_string();
                                    } else {
                                        screen_mode = ScreenMode::InscribeMenu;
                                        status_msg = "Inscribe item: select a letter.".to_string();
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
                                    let has_spike = player.inventory.iter().any(|i| i.name == "Iron Spike" && i.count > 0);
                                    if has_spike {
                                        screen_mode = ScreenMode::SelectJamDirection;
                                        status_msg = "Jam door: choose direction...".to_string();
                                    } else {
                                        status_msg = "You do not carry any iron spikes!".to_string();
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
                                    let is_elf = matches!(player.race, Race::Elf | Race::HalfElf);
                                    let trap_chance = if is_rogue { 0.70 } else { 0.30 };
                                    let door_chance = if is_rogue { 0.60 } else if is_elf { 0.40 } else { 0.20 };
                                    
                                    let mut found_something = false;

                                    for dy in -1..=1 {
                                        for dx in -1..=1 {
                                            if dx == 0 && dy == 0 { continue; }
                                            let sx = (player.x as isize + dx) as usize;
                                            let sy = (player.y as isize + dy) as usize;
                                            
                                            if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                match tile.tile_type {
                                                    TileType::SecretDoor => {
                                                        if rng.gen_bool(door_chance) {
                                                            tile.tile_type = TileType::DoorClosed { spikes: 0 };
                                                            status_msg = "You found a secret door!".to_string();
                                                            found_something = true;
                                                        }
                                                    }
                                                    TileType::Trap { ref mut detected, .. }
                                                        if !*detected && rng.gen_bool(trap_chance) => {
                                                            *detected = true;
                                                            status_msg = "You detected a trap!".to_string();
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
                                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                        std::thread::sleep(std::time::Duration::from_millis(1500));
                                        break 'game_loop;
                                    }
                                    status_msg = "Saving game and quitting...".to_string();
                                    draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                    
                                    let state = GameState::new(player.clone(), level.clone(), monsters.clone(), max_depth);
                                    if let Err(e) = state.save_to_file(&save_path) {
                                        status_msg = format!("Save failed: {}", e);
                                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                        std::thread::sleep(std::time::Duration::from_secs(2));
                                    } else {
                                        std::thread::sleep(std::time::Duration::from_millis(800));
                                    }
                                    break 'game_loop;
                                }
                                _ => {}
                            }
                        }
                    } else if screen_mode == ScreenMode::BarterBuyMenu || screen_mode == ScreenMode::BarterSellMenu {
                        handle_haggle_input(
                            key_event,
                            &mut screen_mode,
                            &mut active_haggle,
                            &mut player,
                            &mut status_msg,
                        );
                    } else {
                        match key_event.code {
                            KeyCode::Esc => {
                                if screen_mode == ScreenMode::ShopSellMenu {
                                    screen_mode = ScreenMode::Shop;
                                    status_msg = "General Store - Buy Menu.".to_string();
                                } else {
                                    screen_mode = ScreenMode::Dungeon;
                                    status_msg = "Returned to dungeon.".to_string();
                                    active_haggle = None;
                                }
                            }
                            KeyCode::Char(c) => {
                                if (screen_mode == ScreenMode::InventoryList && c == 'i')
                                    || (screen_mode == ScreenMode::EquipmentList && c == 'I')
                                    || screen_mode == ScreenMode::HelpMenu
                                {
                                    screen_mode = ScreenMode::Dungeon;
                                    status_msg = "Returned to dungeon.".to_string();
                                } else if screen_mode == ScreenMode::BrowseBookMenu {
                                    // Exit browse via ESC
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
                                             && let TileType::Chest { trapped, locked } = tile.tile_type {
                                                 chest_ref = Some((trapped, locked));
                                             }

                                         if let Some((trapped, locked)) = chest_ref {
                                             if trapped {
                                                 let check_chance = if player.class == Class::Rogue { 0.85 } else { 0.40 };
                                                 if rng.gen_bool(check_chance) {
                                                     if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                         tile.tile_type = TileType::Chest { trapped: false, locked };
                                                     }
                                                     status_msg = "You successfully disarmed the chest trap.".to_string();
                                                 } else {
                                                     status_msg = "You set off the chest trap while trying to disarm it!".to_string();
                                                     level.trigger_chest_trap(sx, sy, &mut player, &mut rng, &mut status_msg, &mut monsters);
                                                 }
                                             } else if locked {
                                                 let check_chance = if player.class == Class::Rogue { 0.75 } else { 0.30 };
                                                 if rng.gen_bool(check_chance) {
                                                     if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                         tile.tile_type = TileType::Chest { trapped: false, locked: false };
                                                     }
                                                     status_msg = "You picked the lock on the chest.".to_string();
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
                                                 && let TileType::Trap { detected, trap_type } = tile.tile_type
                                                     && detected {
                                                         trap_info = Some(trap_type);
                                                     }

                                             if let Some(trap_type) = trap_info {
                                                 let is_rogue = player.class == Class::Rogue;
                                                 let check_chance = if is_rogue { 0.85 } else { 0.40 };
                                                 if rng.gen_bool(check_chance) {
                                                     if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                         tile.tile_type = TileType::Floor;
                                                     }
                                                     status_msg = "You successfully disarmed the trap.".to_string();
                                                 } else {
                                                     status_msg = "You set off the trap while trying to disarm it!".to_string();
                                                     match trap_type {
                                                         TrapType::Arrow => {
                                                             let dmg = rng.gen_range(1..=6);
                                                             player.hp -= dmg;
                                                             status_msg.push_str(&format!(" Arrow hits you for {}!", dmg));
                                                         }
                                                         TrapType::PoisonGas => {
                                                             let dmg = rng.gen_range(2..=8);
                                                             player.hp -= dmg;
                                                             player.flags.poisoned += rng.gen_range(10..=30);
                                                             status_msg.push_str(&format!(" Poison gas hits you for {}! You are poisoned!", dmg));
                                                         }
                                                         TrapType::Teleport => {
                                                             let dest = level.find_random_floor_tile();
                                                             player.move_to(dest.0, dest.1);
                                                             status_msg.push_str(" You are teleported!");
                                                         }
                                                         TrapType::SleepingGas => {
                                                             if player.flags.free_action {
                                                                 status_msg.push_str(" A strange white mist surrounds you! You are unaffected.");
                                                             } else {
                                                                 player.flags.paralysis += rng.gen_range(1..=10) + 4;
                                                                 status_msg.push_str(" A strange white mist surrounds you! You fall asleep.");
                                                             }
                                                         }
                                                         TrapType::BlindGas => {
                                                             player.flags.blind += rng.gen_range(1..=50) + 50;
                                                             status_msg.push_str(" Black gas surrounds you! You are blinded!");
                                                         }
                                                         TrapType::ConfusionGas => {
                                                             player.flags.confused += rng.gen_range(1..=15) + 15;
                                                             status_msg.push_str(" Scintillating colors swirl! You are confused!");
                                                         }
                                                     }
                                                     if let Some(tile) = level.get_tile_mut(sx, sy) {
                                                         tile.tile_type = TileType::Floor;
                                                     }
                                                 }
                                                 disarmed = true;
                                             }
                                         }
                                         if !disarmed {
                                             status_msg = "No detected trap in that direction.".to_string();
                                         }
                                         screen_mode = ScreenMode::Dungeon;
                                         player_acted = true;
                                     }
                                 } else if screen_mode == ScreenMode::EatMenu {
                                     let idx = c as usize - 'a' as usize;
                                     let food_indices: Vec<usize> = player.inventory.iter()
                                         .enumerate()
                                         .filter(|(_, item)| matches!(item.item_type, ItemType::Food {..}))
                                         .map(|(i, _)| i)
                                         .collect();

                                     if idx < food_indices.len() {
                                         let inv_idx = food_indices[idx];
                                         let nutrition = if let ItemType::Food { nutrition } = player.inventory[inv_idx].item_type {
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

                                         status_msg = format!("You ate {}! Restored {} nutrition.", name, nutrition);
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
                                         status_msg = "Inscribing... type characters and press ENTER.".to_string();
                                     }
                                 } else if screen_mode == ScreenMode::GameOver {
                                     match c {
                                         'r' | 'R' => {
                                             restart_game = true;
                                             break 'game_loop;
                                         }
                                         'q' | 'Q' | '\u{1b}' => { // 'q', 'Q', or Esc
                                             break 'game_loop;
                                         }
                                         _ => {}
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
                                                         status_msg = format!("The door is spiked shut! ({} spikes)", spikes);
                                                     } else {
                                                         tile.tile_type = TileType::DoorOpen;
                                                         status_msg = "You open the door.".to_string();
                                                         player_acted = true;
                                                     }
                                                 }
                                                 TileType::Chest { trapped: _, locked } => {
                                                     if locked {
                                                         status_msg = "The chest is locked!".to_string();
                                                     } else {
                                                         level.open_chest(sx, sy, &mut player, &mut rng, &mut status_msg, &mut monsters);
                                                         player_acted = true;
                                                     }
                                                 }
                                                 _ => {
                                                     status_msg = "That is not a closed door or chest.".to_string();
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
                                                 if sx < 0 || sx >= level.width as isize || sy < 0 || sy >= level.height as isize {
                                                     break;
                                                 }
                                                 let x_u = sx as usize;
                                                 let y_u = sy as usize;
                                                 
                                                 if let Some(tile) = level.get_tile(x_u, y_u) {
                                                     if !tile.is_passable() || tile.tile_type == TileType::SecretDoor {
                                                         break;
                                                     }
                                                     if tile.visible && let Some(m) = monsters.iter().find(|m| m.x == x_u && m.y == y_u) {
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
                                                 status_msg = "You see nothing in that direction.".to_string();
                                             }
                                         } else {
                                             status_msg = "You look around but focus on nothing.".to_string();
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
                                                 tile.tile_type = TileType::DoorClosed { spikes: 0 };
                                                 status_msg = "You close the door.".to_string();
                                                 player_acted = true;
                                             } else {
                                                 status_msg = "That is not an open door.".to_string();
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
                                             if let TileType::DoorClosed { ref mut spikes } = tile.tile_type {
                                                 if let Some(spike_idx) = player.inventory.iter().position(|i| i.name == "Iron Spike" && i.count > 0) {
                                                     player.inventory[spike_idx].count -= 1;
                                                     if player.inventory[spike_idx].count == 0 {
                                                         player.inventory.remove(spike_idx);
                                                     }
                                                     *spikes += 1;
                                                     status_msg = format!("You jammed the door. ({} spikes used total)", spikes);
                                                     player_acted = true;
                                                 } else {
                                                     status_msg = "You do not carry any iron spikes!".to_string();
                                                 }
                                             } else {
                                                 status_msg = "That is not a closed door.".to_string();
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

                                        let monster_idx = monsters.iter().position(|m| m.x == sx && m.y == sy);
                                        if let Some(m_idx) = monster_idx {
                                            let monster_name = monsters[m_idx].name.clone();
                                            let check_chance = 0.40 + (player.stats.strength as f64 - 10.0) * 0.05 + (player.stats.dexterity as f64 - 10.0) * 0.02;
                                            let check_chance = check_chance.clamp(0.10, 0.95);
                                            
                                            if rng.gen_bool(check_chance) {
                                                let base_dmg = rng.gen_range(1..=6);
                                                let str_bonus = (player.stats.strength as i32 - 10) / 2;
                                                let damage = (base_dmg + str_bonus).max(1);
                                                
                                                status_msg = format!("You bash the {} for {} damage!", monster_name, damage);
                                                
                                                let can_stun = !(monster_name == "The Balrog" || monster_name.contains("Lich"));
                                                if can_stun {
                                                    let stun_turns = rng.gen_range(1..=3) + 1;
                                                    monsters[m_idx].stunned += stun_turns;
                                                    status_msg.push_str(" It appears stunned!");
                                                }

                                                monsters[m_idx].was_attacked = true;
                                                let died = monsters[m_idx].take_damage(damage);
                                                if died {
                                                    if level.depth == 0 {
                                                        player.killed_town_npcs += 1;
                                                    }
                                                    status_msg.push_str(&format!(" You have slain the {}!", monster_name));
                                                    
                                                    let exp_reward = monsters[m_idx].experience_reward;
                                                    if level.depth > 0 {
                                                        player.exp += exp_reward;
                                                        status_msg.push_str(&format!(" Gained {} EXP.", exp_reward));
                                                    } else {
                                                        status_msg.push_str(" Gained 0 EXP (Town monster).");
                                                    }
                                                    
                                                    let is_boss = monster_name == "The Balrog" || monster_name.contains("Lich");
                                                    if is_boss || rng.gen_bool(0.30) {
                                                        let drop_item = if monster_name == "The Balrog" {
                                                            Item::new("Broadsword of Slaying", 1, 150, ItemType::Weapon { damage: Dice::new(3, 6) })
                                                        } else {
                                                            generate_random_floor_item(level.depth, &mut rng)
                                                        };
                                                        level.items.push(FloorItem { x: sx, y: sy, item: drop_item.clone() });
                                                        status_msg.push_str(&format!(" It dropped a {}!", drop_item.name));
                                                    }
                                                    
                                                    if monster_name == "The Balrog" {
                                                        player.balrog_killed = true;
                                                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                                        execute!(stdout, Show, LeaveAlternateScreen)?;
                                                        disable_raw_mode()?;
                                                        println!("============================================================");
                                                        println!("         CONGRATULATIONS! YOU HAVE SLAIN THE BALROG!        ");
                                                        println!("        You have completed the quest and won rmoria!        ");
                                                        println!("============================================================");
                                                        if save_path.exists() {
                                                            let _ = fs::remove_file(&save_path);
                                                        }
                                                        break 'game_loop;
                                                    }
                                                    monsters.remove(m_idx);
                                                }
                                            } else {
                                                status_msg = format!("You swing and miss the {}.", monster_name);
                                                if rng.gen_range(0..150) > player.stats.dexterity as i32 {
                                                    status_msg.push_str(" You are off-balance.");
                                                }
                                            }
                                            player_acted = true;
                                        } else if let Some(tile) = level.get_tile_mut(sx, sy) {
                                            match tile.tile_type {
                                                 TileType::DoorClosed { ref mut spikes } => {
                                                      let chance = player.stats.strength as f64;
                                                      let diff = 8.0 + (*spikes as f64) * 4.0;
                                                      
                                                      if rng.gen_range(0.0..chance) > diff {
                                                          tile.tile_type = TileType::DoorOpen;
                                                          status_msg = "The door crashes open!".to_string();
                                                          player.move_to(sx, sy);
                                                      } else {
                                                          status_msg = "The door holds firm.".to_string();
                                                          if rng.gen_range(0..150) > player.stats.dexterity as i32 {
                                                              status_msg.push_str(" You are off-balance.");
                                                          }
                                                      }
                                                      player_acted = true;
                                                 }
                                                 TileType::Chest { trapped: _, locked } => {
                                                     if rng.gen_bool(0.10) {
                                                        tile.tile_type = TileType::Floor;
                                                        status_msg = "You have destroyed the chest and its contents!".to_string();
                                                    } else if locked {
                                                        let chance = player.stats.strength as f64;
                                                        if rng.gen_range(0.0..chance) > 12.0 {
                                                            status_msg = "You break the lock!".to_string();
                                                            level.open_chest(sx, sy, &mut player, &mut rng, &mut status_msg, &mut monsters);
                                                        } else {
                                                            status_msg = "The lock holds firm.".to_string();
                                                        }
                                                    } else {
                                                        status_msg = "You bash the chest open!".to_string();
                                                        level.open_chest(sx, sy, &mut player, &mut rng, &mut status_msg, &mut monsters);
                                                    }
                                                    player_acted = true;
                                                }
                                                _ => {
                                                    status_msg = "There is nothing there to bash.".to_string();
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

                                        let monster_idx = monsters.iter().position(|m| m.x == sx && m.y == sy);
                                        if monster_idx.is_some() {
                                            status_msg = "There is a monster in the way!".to_string();
                                        } else if let Some(tile) = level.get_tile_mut(sx, sy) {
                                            let ability = player.digging_ability();
                                            match tile.tile_type {
                                                TileType::Wall => {
                                                    if ability == 0 {
                                                        status_msg = "You dig with your hands, making no progress.".to_string();
                                                        player_acted = true;
                                                    } else {
                                                        let chance = rng.gen_range(1..=1200) + 80;
                                                        if ability > chance {
                                                            tile.tile_type = TileType::Floor;
                                                            status_msg = "You have finished the tunnel.".to_string();
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
                                                        let chance = rng.gen_range(1..=600) + 10;
                                                        if ability > chance {
                                                            tile.tile_type = TileType::Floor;
                                                            status_msg = "You have finished the tunnel.".to_string();
                                                            if gold {
                                                                let amount = rng.gen_range(50..=200);
                                                                player.gold += amount;
                                                                status_msg = format!("You found a vein of gold! (Gained {} gp)", amount);
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
                                                        let chance = rng.gen_range(1..=400) + 10;
                                                        if ability > chance {
                                                            tile.tile_type = TileType::Floor;
                                                            status_msg = "You have finished the tunnel.".to_string();
                                                            if gold {
                                                                let amount = rng.gen_range(50..=200);
                                                                player.gold += amount;
                                                                status_msg = format!("You found a vein of gold! (Gained {} gp)", amount);
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
                                                            status_msg = "You have removed the rubble.".to_string();
                                                            if rng.gen_bool(0.10) {
                                                                let item = generate_random_floor_item(level.depth, &mut rng);
                                                                status_msg = format!("You removed the rubble and found a {} under it!", item.name);
                                                                level.items.push(FloorItem { x: sx, y: sy, item });
                                                            }
                                                        } else {
                                                            status_msg = "You dig in the rubble.".to_string();
                                                        }
                                                        player_acted = true;
                                                    }
                                                }
                                                TileType::SecretDoor => {
                                                    tile.tile_type = TileType::DoorClosed { spikes: 0 };
                                                    status_msg = "You tunnel into the granite wall. You found a secret door!".to_string();
                                                    player_acted = true;
                                                }
                                                TileType::Floor | TileType::DoorOpen => {
                                                    status_msg = "Tunnel through what? Empty air?!?".to_string();
                                                }
                                                _ => {
                                                    status_msg = "You can't tunnel through that.".to_string();
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
                                        
                                        level.items.push(FloorItem { x: player.x, y: player.y, item: dropped });
                                        status_msg = format!("You dropped a {}.", name);
                                        player_acted = true;
                                    }
                                    screen_mode = ScreenMode::Dungeon;
                                } else if screen_mode == ScreenMode::AimWandMenu {
                                    let wands: Vec<(usize, &Item)> = player.inventory.iter()
                                        .enumerate()
                                        .filter(|(_, item)| matches!(item.item_type, ItemType::Wand {..}))
                                        .collect();
                                    let idx = c as usize - 'a' as usize;
                                    if idx < wands.len() {
                                        let (inv_idx, item) = wands[idx];
                                        if let ItemType::Wand { charges, .. } = item.item_type {
                                            if charges > 0 {
                                                selected_wand_inv_idx = inv_idx;
                                                screen_mode = ScreenMode::SelectWandDirection;
                                                status_msg = "Aim wand: choose direction...".to_string();
                                            } else {
                                                status_msg = "That wand has no charges left!".to_string();
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
                                        if let ItemType::Wand { ref mut charges, spell_index } = player.inventory[selected_wand_inv_idx].item_type
                                            && *charges > 0 {
                                                *charges -= 1;
                                                charges_left = *charges;
                                                s_idx = spell_index;
                                            }

                                        let mut cx = player.x as isize + dx;
                                        let mut cy = player.y as isize + dy;
                                        status_msg = "Your wand beam fades into the dark!".to_string();
                                        
                                        while let Some(tile) = level.get_tile(cx as usize, cy as usize) {
                                            if tile.tile_type == TileType::Wall || tile.tile_type == TileType::SecretDoor || matches!(tile.tile_type, TileType::MagmaVein { .. } | TileType::QuartzVein { .. } | TileType::Rubble) {
                                                status_msg = "Your wand blast strikes a wall and dissipates!".to_string();
                                                break;
                                            }
                                            if let Some(m_idx) = monsters.iter().position(|m| m.x == cx as usize && m.y == cy as usize) {
                                                let dmg = if s_idx == 0 { rng.gen_range(2..=8) } else { rng.gen_range(4..=24) };
                                                let m_name = monsters[m_idx].name.clone();
                                                let exp = monsters[m_idx].experience_reward;
                                                status_msg = format!("Wand beam hits {} for {} damage!", m_name, dmg);
                                                monsters[m_idx].was_attacked = true;
                                                if monsters[m_idx].take_damage(dmg) {
                                                    status_msg.push_str(" You killed it!");
                                                    player.add_experience(exp);
                                                    if level.depth == 0 {
                                                        player.killed_town_npcs += 1;
                                                    }
                                                    monsters.remove(m_idx);
                                                }
                                                break;
                                            }
                                            cx += dx;
                                            cy += dy;
                                        }

                                        status_msg.push_str(&format!(" ({} charges remaining)", charges_left));
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
                                } else if screen_mode == ScreenMode::UseStaffMenu {
                                    let staves: Vec<(usize, &Item)> = player.inventory.iter()
                                        .enumerate()
                                        .filter(|(_, item)| matches!(item.item_type, ItemType::Staff {..}))
                                        .collect();
                                    let idx = c as usize - 'a' as usize;
                                    if idx < staves.len() {
                                        let (inv_idx, _item) = staves[idx];
                                        let mut charges_left = 0;
                                        let mut p_idx = 0;
                                        let mut used_charge = false;
                                        if let ItemType::Staff { ref mut charges, prayer_index } = player.inventory[inv_idx].item_type
                                            && *charges > 0 {
                                                *charges -= 1;
                                                charges_left = *charges;
                                                p_idx = prayer_index;
                                                used_charge = true;
                                            }

                                        if used_charge {
                                            if p_idx == 1 {
                                                let heal = rng.gen_range(2..=16);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("You use the staff. Restored {} HP. ({} charges remaining)", heal, charges_left);
                                            } else {
                                                status_msg = format!("You use the staff, but nothing happens. ({} charges remaining)", charges_left);
                                            }
                                            player_acted = true;
                                        } else {
                                            status_msg = "That staff has no charges left!".to_string();
                                        }
                                    }
                                    screen_mode = ScreenMode::Dungeon;
                                } else if screen_mode == ScreenMode::CastSpellMenu {
                                    match c {
                                        'a' => {
                                            if player.mana >= 1 {
                                                selected_spell_idx = 0;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Magic Missile: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'b' => {
                                            if player.mana >= 2 {
                                                player.mana -= 2;
                                                let dest = level.find_random_floor_tile();
                                                player.move_to(dest.0, dest.1);
                                                status_msg = "You cast Phase Door and warp!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'c' => {
                                            if player.level < 3 {
                                                status_msg = "Too low level (requires Level 3)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 3 {
                                                player.mana -= 3;
                                                for y in (player.y as isize - 5)..=(player.y as isize + 5) {
                                                    for x in (player.x as isize - 5)..=(player.x as isize + 5) {
                                                        if let Some(tile) = level.get_tile_mut(x as usize, y as usize) {
                                                            tile.visible = true;
                                                            tile.remembered = true;
                                                        }
                                                    }
                                                }
                                                status_msg = "Brilliant light flashes and illuminates the area!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'd' => {
                                            if player.level < 5 {
                                                status_msg = "Too low level (requires Level 5)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 5 {
                                                selected_spell_idx = 1;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Fire Bolt: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'e' => {
                                            if player.level < 10 {
                                                status_msg = "Too low level (requires Level 10)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 8 {
                                                selected_spell_idx = 2;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Frost Bolt: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'f' => {
                                            if player.level < 15 {
                                                status_msg = "Too low level (requires Level 15)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 10 {
                                                player.mana -= 10;
                                                let dest = level.find_random_floor_tile();
                                                player.move_to(dest.0, dest.1);
                                                status_msg = "You teleport yourself across space!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'g' => {
                                            if player.level < 20 {
                                                status_msg = "Too low level (requires Level 20)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 15 {
                                                selected_spell_idx = 3;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Lightning Bolt: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'h' => {
                                            if player.level < 25 {
                                                status_msg = "Too low level (requires Level 25)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 20 {
                                                selected_spell_idx = 4;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Fire Ball: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'i' => {
                                            if player.level < 35 {
                                                status_msg = "Too low level (requires Level 35)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 30 {
                                                selected_spell_idx = 5;
                                                screen_mode = ScreenMode::SelectSpellDirection;
                                                status_msg = "Aim Mana Storm: choose direction...".to_string();
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if screen_mode == ScreenMode::PrayMenu {
                                    match c {
                                        'a' => {
                                            if player.mana >= 1 {
                                                player.mana -= 1;
                                                for m in &monsters {
                                                    if let Some(tile) = level.get_tile_mut(m.x, m.y) {
                                                        tile.remembered = true;
                                                    }
                                                }
                                                status_msg = format!("You sense the presence of {} dark entities!", monsters.len());
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'b' => {
                                            if player.mana >= 2 {
                                                player.mana -= 2;
                                                let heal = rng.gen_range(2..=16);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("You pray for healing. Restored {} HP.", heal);
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'c' => {
                                            if player.level < 3 {
                                                status_msg = "Too low level (requires Level 3)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 3 {
                                                player.mana -= 3;
                                                let heal = rng.gen_range(3..=24);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("Divine blessing heals you for {} HP.", heal);
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'd' => {
                                            if player.level < 4 {
                                                status_msg = "Too low level (requires Level 4)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 4 {
                                                player.mana -= 4;
                                                let dest = level.find_random_floor_tile();
                                                player.move_to(dest.0, dest.1);
                                                status_msg = "You are pulled through a space portal!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'e' => {
                                            if player.level < 7 {
                                                status_msg = "Too low level (requires Level 7)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 7 {
                                                player.mana -= 7;
                                                let mut hit = false;
                                                let mut killed = Vec::new();
                                                for (m_idx, m) in monsters.iter_mut().enumerate() {
                                                    let dx = (player.x as isize - m.x as isize).abs();
                                                    let dy = (player.y as isize - m.y as isize).abs();
                                                    if dx <= 1 && dy <= 1 {
                                                        let dmg = rng.gen_range(6..=48);
                                                        m.was_attacked = true;
                                                        m.take_damage(dmg);
                                                        hit = true;
                                                        if m.hp <= 0 {
                                                            killed.push(m_idx);
                                                        }
                                                    }
                                                }
                                                for &idx in killed.iter().rev() {
                                                    let exp = monsters[idx].experience_reward;
                                                    player.add_experience(exp);
                                                    if level.depth == 0 {
                                                        player.killed_town_npcs += 1;
                                                    }
                                                    monsters.remove(idx);
                                                }
                                                status_msg = if hit {
                                                    "You chant a Holy Word! Nearby enemies are scorched!".to_string()
                                                } else {
                                                    "You chant a Holy Word, but hear only whispers.".to_string()
                                                };
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'f' => {
                                            if player.level < 12 {
                                                status_msg = "Too low level (requires Level 12)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 12 {
                                                player.mana -= 12;
                                                let heal = rng.gen_range(6..=48);
                                                player.hp = (player.hp + heal).min(player.max_hp);
                                                status_msg = format!("Critical wounds heal! Restored {} HP.", heal);
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'g' => {
                                            if player.level < 18 {
                                                status_msg = "Too low level (requires Level 18)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 15 {
                                                player.mana -= 15;
                                                player.hp = player.max_hp;
                                                status_msg = "Sanctuary! Divine protection restores your vitality to maximum!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'h' => {
                                            if player.level < 25 {
                                                status_msg = "Too low level (requires Level 25)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 25 {
                                                player.mana -= 25;
                                                let mut killed = Vec::new();
                                                for (m_idx, m) in monsters.iter_mut().enumerate() {
                                                    let dmg = rng.gen_range(12..=96);
                                                    m.take_damage(dmg);
                                                    if m.hp <= 0 {
                                                        killed.push(m_idx);
                                                    }
                                                }
                                                for &idx in killed.iter().rev() {
                                                    let exp = monsters[idx].experience_reward;
                                                    player.add_experience(exp);
                                                    if level.depth == 0 {
                                                        player.killed_town_npcs += 1;
                                                    }
                                                    monsters.remove(idx);
                                                }
                                                status_msg = "Holy Thunder strikes all visible foes!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        'i' => {
                                            if player.level < 35 {
                                                status_msg = "Too low level (requires Level 35)!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            } else if player.mana >= 35 {
                                                player.mana -= 35;
                                                let mut killed = Vec::new();
                                                for (m_idx, m) in monsters.iter_mut().enumerate() {
                                                    let dmg = rng.gen_range(20..=200);
                                                    m.take_damage(dmg);
                                                    if m.hp <= 0 {
                                                        killed.push(m_idx);
                                                    }
                                                }
                                                for &idx in killed.iter().rev() {
                                                    let exp = monsters[idx].experience_reward;
                                                    player.add_experience(exp);
                                                    if level.depth == 0 {
                                                        player.killed_town_npcs += 1;
                                                    }
                                                    monsters.remove(idx);
                                                }
                                                status_msg = "Divine Wrath obliterates your enemies!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                                player_acted = true;
                                            } else {
                                                status_msg = "Insufficient mana!".to_string();
                                                screen_mode = ScreenMode::Dungeon;
                                            }
                                        }
                                        _ => {}
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

                                        let (cost, spell_name, min_d, max_d) = match selected_spell_idx {
                                            0 => (1, "Magic Missile", 2, 8),
                                            1 => (5, "Fire Bolt", 4, 24),
                                            2 => (8, "Frost Bolt", 6, 36),
                                            3 => (15, "Lightning Bolt", 8, 64),
                                            4 => (20, "Fire Ball", 12, 96),
                                            _ => (30, "Mana Storm", 20, 200),
                                        };

                                        if player.mana >= cost {
                                            player.mana -= cost;
                                            let mut cx = player.x as isize + dx;
                                            let mut cy = player.y as isize + dy;
                                            status_msg = format!("Your {} streaks out into the dark!", spell_name);
                                            while let Some(tile) = level.get_tile(cx as usize, cy as usize) {
                                                if tile.tile_type == TileType::Wall || tile.tile_type == TileType::SecretDoor || matches!(tile.tile_type, TileType::MagmaVein { .. } | TileType::QuartzVein { .. } | TileType::Rubble) {
                                                    status_msg = format!("Your {} strikes a wall!", spell_name);
                                                    break;
                                                }
                                                if let Some(m_idx) = monsters.iter().position(|m| m.x == cx as usize && m.y == cy as usize) {
                                                    let dmg = rng.gen_range(min_d..=max_d);
                                                    let m_name = monsters[m_idx].name.clone();
                                                    let exp = monsters[m_idx].experience_reward;
                                                    status_msg = format!("{} hits {} for {} damage!", spell_name, m_name, dmg);
                                                    monsters[m_idx].was_attacked = true;
                                                    if monsters[m_idx].take_damage(dmg) {
                                                        status_msg.push_str(" You destroyed it!");
                                                        player.add_experience(exp);
                                                        if level.depth == 0 {
                                                            player.killed_town_npcs += 1;
                                                        }
                                                        monsters.remove(m_idx);
                                                    }
                                                    break;
                                                }
                                                cx += dx;
                                                cy += dy;
                                            }
                                        } else {
                                            status_msg = "Insufficient mana!".to_string();
                                        }

                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
                                } else if screen_mode == ScreenMode::Shop {
                                    match c {
                                        's' => {
                                            screen_mode = ScreenMode::ShopSellMenu;
                                            status_msg = "Sell selection mode. Select item.".to_string();
                                        }
                                        letter if letter.is_ascii_lowercase() => {
                                            let idx = letter as usize - 'a' as usize;
                                            let items = get_shop_items(active_shop);
                                            if idx < items.len() {
                                                let (name, price, ref item_type) = items[idx];
                                                let min_p = (price * 6 / 10).max(1);
                                                if player.gold >= min_p {
                                                    let base_p = price;
                                                    let h_item = Item::new(name, 1, match item_type {
                                                         ItemType::Weapon {..} => 10,
                                                         ItemType::Armor {..} => 80,
                                                         ItemType::Potion {..} => 5,
                                                         ItemType::Light {..} => 15,
                                                         _ => 2,
                                                     }, item_type.clone());
                                                    
                                                    active_haggle = Some(HaggleState {
                                                        item: h_item,
                                                        item_index: idx,
                                                        initial_price: base_p,
                                                        min_price: min_p,
                                                        max_price: base_p,
                                                        current_asking: base_p,
                                                        last_bid: None,
                                                        insults: 0,
                                                        offers_count: 0,
                                                        comment: "I will sell this for...".to_string(),
                                                        player_input: String::new(),
                                                    });
                                                    screen_mode = ScreenMode::BarterBuyMenu;
                                                    status_msg = format!("Bartering for {}.", name);
                                                } else {
                                                    status_msg = "You don't have enough gold to make a serious offer!".to_string();
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if screen_mode == ScreenMode::ShopSellMenu {
                                    let idx = c as usize - 'a' as usize;
                                    if idx < player.inventory.len() {
                                        let item = &player.inventory[idx];
                                        let base_val = match item.item_type {
                                            ItemType::Weapon {..} => 25,
                                            ItemType::Armor {..} => 40,
                                            ItemType::Potion {..} => 15,
                                            ItemType::Scroll {..} => 10,
                                            ItemType::Wand {..} => 50,
                                            ItemType::Staff {..} => 60,
                                            ItemType::Food {..} => 5,
                    ItemType::Light {..} => 15,
                                        };

                                        active_haggle = Some(HaggleState {
                                            item: item.clone(),
                                            item_index: idx,
                                            initial_price: (base_val * 6 / 10).max(1),
                                            min_price: (base_val * 5 / 10).max(1),
                                            max_price: (base_val * 13 / 10).max(1),
                                            current_asking: (base_val * 6 / 10).max(1),
                                            last_bid: None,
                                            insults: 0,
                                            offers_count: 0,
                                            comment: "I can pay...".to_string(),
                                            player_input: String::new(),
                                        });
                                        screen_mode = ScreenMode::BarterSellMenu;
                                        status_msg = format!("Bartering to sell {}.", item.name);
                                    }
                                } else if screen_mode == ScreenMode::WearMenu {
                                    let idx = c as usize - 'a' as usize;
                                    let equippable_indices: Vec<usize> = player.inventory.iter()
                                        .enumerate()
                                        .filter(|(_, item)| {
                                            matches!(item.item_type, ItemType::Weapon {..} | ItemType::Armor {..} | ItemType::Light {..})
                                        })
                                        .map(|(i, _)| i)
                                        .collect();

                                    if idx < equippable_indices.len() {
                                        let inv_idx = equippable_indices[idx];
                                        let item = player.inventory.remove(inv_idx);
                                        let item_slot = item.get_equipment_slot();
                                        let already_equipped = player.equipment.iter().position(|eq| {
                                            eq.get_equipment_slot() == item_slot
                                        });

                                        if let Some(eq_idx) = already_equipped {
                                            let old = player.equipment.remove(eq_idx);
                                            status_msg = format!("You take off {} and equip {}.", old.name, item.name);
                                            player.inventory.push(old);
                                        } else {
                                            status_msg = format!("You wear/wield {}.", item.name);
                                        }

                                        player.equipment.push(item);
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
                                } else if screen_mode == ScreenMode::TakeOffMenu {
                                    let idx = c as usize - 'a' as usize;
                                    if idx < player.equipment.len() {
                                        let item = player.equipment.remove(idx);
                                        status_msg = format!("You took off {}.", item.name);
                                        player.inventory.push(item);
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
                                } else if screen_mode == ScreenMode::QuaffMenu {
                                    let idx = c as usize - 'a' as usize;
                                    let potion_indices: Vec<usize> = player.inventory.iter()
                                        .enumerate()
                                        .filter(|(_, item)| matches!(item.item_type, ItemType::Potion {..}))
                                        .map(|(i, _)| i)
                                        .collect();

                                    if idx < potion_indices.len() {
                                        let inv_idx = potion_indices[idx];
                                        let name = player.inventory[inv_idx].name.clone();
                                        let heal = if let ItemType::Potion { heal_amount } = player.inventory[inv_idx].item_type {
                                            heal_amount
                                        } else {
                                            0
                                        };
                                        
                                        player.inventory[inv_idx].count -= 1;
                                        if player.inventory[inv_idx].count == 0 {
                                            player.inventory.remove(inv_idx);
                                        }

                                        if name.contains("Cure Light Wounds") {
                                            player.hp = (player.hp + heal).min(player.max_hp);
                                            player.flags.blind = 0;
                                            player.flags.afraid = 0;
                                            status_msg = format!("You quaffed {}! Restored {} HP. You feel much better.", name, heal);
                                        } else if name.contains("Healing") {
                                            player.hp = player.max_hp;
                                            player.flags.blind = 0;
                                            player.flags.afraid = 0;
                                            player.flags.poisoned = 0;
                                            player.flags.confused = 0;
                                            status_msg = format!("You quaffed {}! Restored full HP and cured conditions.", name);
                                        } else if name.contains("Cure Poison") {
                                            player.flags.poisoned = 0;
                                            status_msg = format!("You quaffed {}! The poison leaves your veins.", name);
                                        } else if name.contains("Speed") {
                                            player.flags.fast += rng.gen_range(15..=40);
                                            status_msg = format!("You quaffed {}! You feel yourself moving faster!", name);
                                        } else if name.contains("Heroism") {
                                            player.hp = (player.hp + 10).min(player.max_hp);
                                            player.flags.afraid = 0;
                                            player.flags.heroism += rng.gen_range(25..=50);
                                            status_msg = format!("You quaffed {}! You feel like a HERO!", name);
                                        } else if name.contains("Restore Strength") {
                                            player.restore_stat(0);
                                            status_msg = format!("You quaffed {}! You feel your strength returning.", name);
                                        } else if name.contains("Restore Dexterity") {
                                            player.restore_stat(3);
                                            status_msg = format!("You quaffed {}! You feel less clumsy.", name);
                                        } else if name.contains("Restore Constitution") {
                                            player.restore_stat(4);
                                            status_msg = format!("You quaffed {}! You feel your health returning.", name);
                                        } else if name.contains("Restore Intelligence") {
                                            player.restore_stat(1);
                                            status_msg = format!("You quaffed {}! Your mind feels clearer.", name);
                                        } else if name.contains("Restore Wisdom") {
                                            player.restore_stat(2);
                                            status_msg = format!("You quaffed {}! You feel your wisdom returning.", name);
                                        } else if name.contains("Restore Charisma") {
                                            player.restore_stat(5);
                                            status_msg = format!("You quaffed {}! You feel your looks returning.", name);
                                        } else {
                                            player.hp = (player.hp + heal).min(player.max_hp);
                                            status_msg = format!("You quaffed {}! Restored {} HP.", name, heal);
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
                                    }
                                } else if screen_mode == ScreenMode::ReadMenu {
                                    let idx = c as usize - 'a' as usize;
                                    let scroll_indices: Vec<usize> = player.inventory.iter()
                                        .enumerate()
                                        .filter(|(_, item)| {
                                            matches!(item.item_type, ItemType::Scroll {..}) &&
                                            !item.name.contains("Torch") &&
                                            !item.name.contains("Spellbook") &&
                                            !item.name.contains("Prayerbook")
                                        })
                                        .map(|(i, _)| i)
                                        .collect();

                                    if idx < scroll_indices.len() {
                                        let inv_idx = scroll_indices[idx];
                                        let name = player.inventory[inv_idx].name.clone();
                                        
                                        player.inventory[inv_idx].count -= 1;
                                        if player.inventory[inv_idx].count == 0 {
                                            player.inventory.remove(inv_idx);
                                        }

                                        if name.contains("Word of Recall") {
                                            if player.flags.word_of_recall == 0 {
                                                player.flags.word_of_recall = rng.gen_range(25..=55);
                                                status_msg = format!("You read the {}! The air about you becomes charged...", name);
                                            } else {
                                                player.flags.word_of_recall = 0;
                                                status_msg = format!("You read the {}! A tension leaves the air around you.", name);
                                            }
                                        } else if name.contains("Phase Door") {
                                            let mut candidates = Vec::new();
                                            for dy in -10..=10 {
                                                for dx in -10..=10 {
                                                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                                                    if dist <= 10.0 && (dx != 0 || dy != 0) {
                                                        let nx = player.x as isize + dx;
                                                        let ny = player.y as isize + dy;
                                                        if nx > 0 && nx < (level.width - 1) as isize && ny > 0 && ny < (level.height - 1) as isize {
                                                            let (ux, uy) = (nx as usize, ny as usize);
                                                            if let Some(tile) = level.get_tile(ux, uy)
                                                                && tile.tile_type == TileType::Floor
                                                                && !monsters.iter().any(|m| m.x == ux && m.y == uy) {
                                                                    candidates.push((ux, uy));
                                                                }
                                                        }
                                                    }
                                                }
                                            }
                                            if !candidates.is_empty() {
                                                let (rx, ry) = candidates[rng.gen_range(0..candidates.len())];
                                                player.move_to(rx, ry);
                                                status_msg = format!("You read the {}! You phase door to ({}, {}).", name, rx, ry);
                                            } else {
                                                status_msg = format!("You read the {}, but nothing seems to happen.", name);
                                            }
                                        } else {
                                            let (rx, ry) = loop {
                                                let tx = rng.gen_range(1..(level.width - 1));
                                                let ty = rng.gen_range(1..(level.height - 1));
                                                if let Some(tile) = level.get_tile(tx, ty)
                                                    && tile.tile_type == TileType::Floor {
                                                        break (tx, ty);
                                                    }
                                            };
                                            player.move_to(rx, ry);

                                            status_msg = format!("You read the {}! You teleport to ({}, {}).", name, rx, ry);
                                        }
                                        screen_mode = ScreenMode::Dungeon;
                                        player_acted = true;
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
                            let door_chance = if is_rogue { 0.60 } else if is_elf { 0.40 } else { 0.20 };
                            
                            let mut found_something = false;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 { continue; }
                                    let sx = (player.x as isize + dx) as usize;
                                    let sy = (player.y as isize + dy) as usize;
                                    
                                    if let Some(tile) = level.get_tile_mut(sx, sy) {
                                        match tile.tile_type {
                                            TileType::SecretDoor => {
                                                if rng.gen_bool(door_chance) {
                                                    tile.tile_type = TileType::DoorClosed { spikes: 0 };
                                                    status_msg = "You found a secret door!".to_string();
                                                    found_something = true;
                                                }
                                            }
                                            TileType::Trap { ref mut detected, .. }
                                                if !*detected && rng.gen_bool(trap_chance) => {
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
                        )?;
                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
                    } else {
                        draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
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
            
            if resting_until_healed && player.hp == player.max_hp && player.mana == player.max_mana {
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
                )?;
                draw_map(&mut level, &player, &monsters, &status_msg, screen_mode, active_shop, active_haggle.as_ref(), active_inscribe.as_ref())?;
            }
        }
    }

        if restart_game {
            if !is_gui {
                let _ = execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
            }
            continue 'outer_loop;
        }
        break 'outer_loop;
    }

    if !is_gui {
        execute!(stdout, Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
    }

    Ok(())
}
