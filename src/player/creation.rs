use std::io::{self, Write};
use crossterm::{execute, event::{self, Event, KeyCode}, cursor::{Show, Hide}, terminal::{enable_raw_mode, disable_raw_mode}};

use crate::player::{Player, Race, Class, Attributes, PlayerFlags, format_stat, generate_history};
use crate::rendering::wrap_text;

fn print_creation_screen(title: &str, options: &[&str]) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 22) / 2).max(0) as u16;

    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    let mut row_idx = 0;
    
    let mut print_line = |text: &str| {
        let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + row_idx));
        print!("{}", text);
        row_idx += 1;
    };

    print_line("==========================================================");
    print_line(&format!("  Moria Character Creator: {:<30}", title));
    print_line("==========================================================");
    print_line("");
    for opt in options {
        print_line(&format!("  {}", opt));
    }
    if options.len() < 16 {
        for _ in 0..(16 - options.len()) {
            print_line("");
        }
    }
    print_line("==========================================================");
    let _ = io::stdout().flush();
}

pub fn run_character_creation(
    _stdout: &mut io::Stdout,
    rng: &mut impl rand::Rng,
    setup_val: Option<serde_json::Value>,
) -> Result<Player, Box<dyn std::error::Error>> {
    if let Some(val) = setup_val {
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

        let s_str = rng.gen_range(8..=18);
        let s_int = rng.gen_range(8..=18);
        let s_wis = rng.gen_range(8..=18);
        let s_dex = rng.gen_range(8..=18);
        let s_con = rng.gen_range(8..=18);
        let s_chr = rng.gen_range(8..=18);
        let stats = Attributes::new(s_str, s_int, s_wis, s_dex, s_con, s_chr);

        let mut final_player = Player::new(&name, race, class, 30, 10);
        final_player.stats = stats.clone();
        final_player.max_stats = stats;
        let history = generate_history(race);
        final_player.history = history;
        final_player.apply_race_and_class_modifiers();
        final_player.max_stats = final_player.stats.clone();
        final_player.update_max_hp_and_mana();
        final_player.hp = final_player.max_hp;
        final_player.mana = final_player.max_mana;
        
        return Ok(final_player);
    }
    let race = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER RACE", &[
            "a) Human",
            "b) Half-Elf",
            "c) Elf",
            "d) Halfling",
            "e) Gnome",
            "f) Dwarf",
            "g) Half-Orc",
            "h) Half-Troll",
        ]);
        if let Event::Key(key_event) = event::read()?
            && let KeyCode::Char(c) = key_event.code {
                match c {
                    'a' => break Race::Human,
                    'b' => break Race::HalfElf,
                    'c' => break Race::Elf,
                    'd' => break Race::Halfling,
                    'e' => break Race::Gnome,
                    'f' => break Race::Dwarf,
                    'g' => break Race::HalfOrc,
                    'h' => break Race::HalfTroll,
                    _ => {}
                }
            }
    };

    let class = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER CLASS", &[
            "a) Warrior",
            "b) Mage",
            "c) Priest",
            "d) Rogue",
            "e) Ranger",
            "f) Paladin",
        ]);
        if let Event::Key(key_event) = event::read()?
            && let KeyCode::Char(c) = key_event.code {
                match c {
                    'a' => break Class::Warrior,
                    'b' => break Class::Mage,
                    'c' => break Class::Priest,
                    'd' => break Class::Rogue,
                    'e' => break Class::Ranger,
                    'f' => break Class::Paladin,
                    _ => {}
                }
            }
    };

    let (stats, history) = loop {
        let s_str = rng.gen_range(8..=18);
        let s_int = rng.gen_range(8..=18);
        let s_wis = rng.gen_range(8..=18);
        let s_dex = rng.gen_range(8..=18);
        let s_con = rng.gen_range(8..=18);
        let s_chr = rng.gen_range(8..=18);
        let rolled = Attributes::new(s_str, s_int, s_wis, s_dex, s_con, s_chr);

        let mut temp_player = Player {
            name: "".to_string(),
            race,
            class,
            level: 1,
            exp: 0,
            gold: 150,
            x: 0, y: 0,
            max_hp: 15, hp: 15,
            max_mana: 0, mana: 0,
            food: 7500,
            stats: rolled.clone(),
            max_stats: rolled.clone(),
            flags: PlayerFlags::default(),
            max_depth_reached: 0,
            flavors: crate::flavor::FlavorRegistry::new(rng),
            inventory: Vec::new(),
            equipment: Vec::new(),
            balrog_killed: false,
            searching: false,
            is_wizard: false,
            killed_town_npcs: 0,
            town_npc_attack_threshold: rng.gen_range(1..=5),
            base_hp_levels: vec![15; 40],
            exp_factor: 100,
            history: "".to_string(),
        };
        temp_player.apply_race_and_class_modifiers();
        temp_player.max_stats = temp_player.stats.clone();
        let history = generate_history(race);
        let split_history = wrap_text(&history, 54);

        let mut options = vec![
            format!("STR: {:>6}     INT: {:>6}     WIS: {:>6}", format_stat(temp_player.stats.strength), format_stat(temp_player.stats.intelligence), format_stat(temp_player.stats.wisdom)),
            format!("DEX: {:>6}     CON: {:>6}     CHR: {:>6}", format_stat(temp_player.stats.dexterity), format_stat(temp_player.stats.constitution), format_stat(temp_player.stats.charisma)),
            "".to_string(),
            "Racial History:".to_string(),
        ];
        for line in &split_history {
            options.push(format!("  {}", line));
        }
        options.push("".to_string());
        options.push("Press [SPACE] to re-roll stats & history.".to_string());
        options.push("Press [ENTER] to accept these characteristics.".to_string());

        let options_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("ROLL CHARACTER ATTRIBUTES", &options_refs);

        if let Event::Key(key_event) = event::read()?
            && key_event.code == KeyCode::Enter {
                break (rolled, history);
            }
    };

    disable_raw_mode()?;
    execute!(io::stdout(), Show)?;
    
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 8) / 2).max(0) as u16;

    let _ = execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y));
    println!("==========================================================");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 1));
    println!("              ENTER CHARACTER NAME                        ");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 2));
    println!("==========================================================");
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y + 4));
    print!("Name: ");
    let _ = io::stdout().flush();
    
    let mut name = String::new();
    let _ = io::stdin().read_line(&mut name);
    let trimmed_name = name.trim();
    let name_str = if trimmed_name.is_empty() { "Hero" } else { trimmed_name };

    enable_raw_mode()?;
    execute!(io::stdout(), Hide)?;

    let mut final_player = Player::new(name_str, race, class, 30, 10);
    final_player.stats = stats.clone();
    final_player.max_stats = stats;
    final_player.history = history;
    final_player.apply_race_and_class_modifiers();
    final_player.max_stats = final_player.stats.clone();
    final_player.update_max_hp_and_mana();
    final_player.hp = final_player.max_hp;
    final_player.mana = final_player.max_mana;

    Ok(final_player)
}
