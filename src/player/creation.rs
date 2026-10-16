use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

use crate::player::{
    Class, Gender, Player, Race, create_modify_player_stat_with_rng, format_stat,
    generate_history_with_social_class, roll_character_stats,
};
use crate::rendering::wrap_text;

fn print_creation_screen(title: &str, options: &[&str]) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 22) / 2).max(0) as u16;

    let _ = execute!(
        io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    );
    let mut row_idx = 0;

    let mut print_line = |text: &str| {
        let _ = execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(offset_x, offset_y + row_idx)
        );
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
        let name = val["name"].as_str().unwrap_or("Hero").to_string();
        let race_str = val["race"].as_str().unwrap_or("Human");
        let class_str = val["class"].as_str().unwrap_or("Warrior");
        let gender_str = val["gender"].as_str().unwrap_or("Male");

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
        let gender = match gender_str {
            "Female" => Gender::Female,
            _ => Gender::Male,
        };

        let mut final_player = Player::new_with_gender(&name, race, class, gender, 30, 10);
        let rolled = roll_character_stats(rng);
        final_player.stats = rolled;
        final_player.apply_race_and_class_modifiers();
        final_player.base_stats = final_player.stats.clone();
        final_player.max_stats = final_player.stats.clone();
        final_player.gold = final_player.calculate_start_gold(rng);
        final_player.update_equipment_bonuses();
        final_player.update_max_hp_and_mana();
        final_player.update_spells_to_learn();
        final_player.hp = final_player.max_hp;
        final_player.mana = final_player.max_mana;

        return Ok(final_player);
    }

    // 1. Choose Race
    let race = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen(
            "CHOOSE CHARACTER RACE",
            &[
                "a) Human",
                "b) Half-Elf",
                "c) Elf",
                "d) Halfling",
                "e) Gnome",
                "f) Dwarf",
                "g) Half-Orc",
                "h) Half-Troll",
            ],
        );
        if let Event::Key(key_event) = event::read()?
            && let KeyCode::Char(c) = key_event.code
        {
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

    // 2. Choose Gender
    let gender = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER GENDER", &["m) Male", "f) Female"]);
        if let Event::Key(key_event) = event::read()?
            && let KeyCode::Char(c) = key_event.code
        {
            match c {
                'm' | 'M' => break Gender::Male,
                'f' | 'F' => break Gender::Female,
                _ => {}
            }
        }
    };

    // 3. Roll Characteristics (Stats + Race Adjustment, History, Age, Height, Weight)
    let (post_race_stats, history, social_class, age, height, weight) = loop {
        let rolled = roll_character_stats(rng);
        let (r_str, r_int, r_wis, r_dex, r_con, r_chr) = match race {
            Race::Human => (0, 0, 0, 0, 0, 0),
            Race::HalfElf => (-1, 1, 0, 1, -1, 1),
            Race::Elf => (-1, 2, 1, 1, -2, 1),
            Race::Halfling => (-2, 2, 1, 3, 1, 1),
            Race::Gnome => (-1, 2, 0, 2, 1, -2),
            Race::Dwarf => (2, -3, 1, -2, 2, -3),
            Race::HalfOrc => (2, -1, 0, 0, 1, -4),
            Race::HalfTroll => (4, -4, -2, -4, 3, -6),
        };
        let mut post_race = rolled;
        post_race.strength = create_modify_player_stat_with_rng(post_race.strength, r_str, rng);
        post_race.intelligence =
            create_modify_player_stat_with_rng(post_race.intelligence, r_int, rng);
        post_race.wisdom = create_modify_player_stat_with_rng(post_race.wisdom, r_wis, rng);
        post_race.dexterity = create_modify_player_stat_with_rng(post_race.dexterity, r_dex, rng);
        post_race.constitution =
            create_modify_player_stat_with_rng(post_race.constitution, r_con, rng);
        post_race.charisma = create_modify_player_stat_with_rng(post_race.charisma, r_chr, rng);

        let (history_text, sc) = generate_history_with_social_class(race, rng);
        let (a, h, w) = race.generate_age_height_weight(gender, rng);
        let split_history = wrap_text(&history_text, 54);

        let mut options = vec![
            format!(
                "STR: {:>6}     INT: {:>6}     WIS: {:>6}",
                format_stat(post_race.strength),
                format_stat(post_race.intelligence),
                format_stat(post_race.wisdom)
            ),
            format!(
                "DEX: {:>6}     CON: {:>6}     CHR: {:>6}",
                format_stat(post_race.dexterity),
                format_stat(post_race.constitution),
                format_stat(post_race.charisma)
            ),
            format!(
                "Age: {:>3}   Height: {:>3} in   Weight: {:>3} lbs   Social Class: {:>3}",
                a, h, w, sc
            ),
            "".to_string(),
            "Racial History:".to_string(),
        ];
        for line in &split_history {
            options.push(format!("  {}", line));
        }
        options.push("".to_string());
        options.push("Press [SPACE] to re-roll characteristics.".to_string());
        options.push("Press [ENTER] to accept these characteristics.".to_string());

        let options_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("ROLL CHARACTER ATTRIBUTES", &options_refs);

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Enter | KeyCode::Esc => {
                    break (post_race, history_text, sc, a, h, w);
                }
                _ => {}
            }
        }
    };

    // 4. Choose Class (filtered by race's allowed classes)
    let bit_field = race.classes_bit_field();
    let all_classes = [
        (0u32, Class::Warrior, "Warrior"),
        (1u32, Class::Mage, "Mage"),
        (2u32, Class::Priest, "Priest"),
        (3u32, Class::Rogue, "Rogue"),
        (4u32, Class::Ranger, "Ranger"),
        (5u32, Class::Paladin, "Paladin"),
    ];

    let mut allowed_classes = Vec::new();
    for (idx, cls, name) in all_classes {
        if (bit_field & (1 << idx)) != 0 {
            allowed_classes.push((cls, name));
        }
    }

    let class_options: Vec<String> = allowed_classes
        .iter()
        .enumerate()
        .map(|(i, (_, name))| format!("{}) {}", (b'a' + i as u8) as char, name))
        .collect();
    let class_opt_refs: Vec<&str> = class_options.iter().map(|s| s.as_str()).collect();

    let class = loop {
        execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0))?;
        print_creation_screen("CHOOSE CHARACTER CLASS", &class_opt_refs);
        if let Event::Key(key_event) = event::read()?
            && let KeyCode::Char(c) = key_event.code
        {
            let idx = (c as u8).saturating_sub(b'a') as usize;
            if idx < allowed_classes.len() {
                break allowed_classes[idx].0;
            }
        }
    };

    // Apply class modifiers
    let (c_str, c_int, c_wis, c_dex, c_con, c_chr) = match class {
        Class::Warrior => (5, -2, -2, 2, 2, -1),
        Class::Mage => (-5, 3, 0, 1, -2, 1),
        Class::Priest => (-3, -3, 3, -1, 0, 2),
        Class::Rogue => (2, 1, -2, 3, 1, -1),
        Class::Ranger => (2, 2, 0, 1, 1, 1),
        Class::Paladin => (3, -3, 1, 0, 2, 2),
    };
    let mut final_stats = post_race_stats;
    final_stats.strength = create_modify_player_stat_with_rng(final_stats.strength, c_str, rng);
    final_stats.intelligence =
        create_modify_player_stat_with_rng(final_stats.intelligence, c_int, rng);
    final_stats.wisdom = create_modify_player_stat_with_rng(final_stats.wisdom, c_wis, rng);
    final_stats.dexterity = create_modify_player_stat_with_rng(final_stats.dexterity, c_dex, rng);
    final_stats.constitution =
        create_modify_player_stat_with_rng(final_stats.constitution, c_con, rng);
    final_stats.charisma = create_modify_player_stat_with_rng(final_stats.charisma, c_chr, rng);

    // 5. Enter Character Name
    disable_raw_mode()?;
    execute!(io::stdout(), Show)?;

    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let offset_x = ((cols as isize - 58) / 2).max(0) as u16;
    let offset_y = ((rows as isize - 8) / 2).max(0) as u16;

    let _ = execute!(
        io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    );
    let _ = execute!(io::stdout(), crossterm::cursor::MoveTo(offset_x, offset_y));
    println!("==========================================================");
    let _ = execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(offset_x, offset_y + 1)
    );
    println!("              ENTER CHARACTER NAME                        ");
    let _ = execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(offset_x, offset_y + 2)
    );
    println!("==========================================================");
    let _ = execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(offset_x, offset_y + 4)
    );
    print!("Name: ");
    let _ = io::stdout().flush();

    let mut name = String::new();
    let _ = io::stdin().read_line(&mut name);
    let trimmed_name = name.trim();
    let name_str = if trimmed_name.is_empty() {
        "Hero"
    } else {
        trimmed_name
    };

    enable_raw_mode()?;
    execute!(io::stdout(), Hide)?;

    let mut final_player = Player::new_with_gender(name_str, race, class, gender, 30, 10);
    final_player.stats = final_stats.clone();
    final_player.base_stats = final_stats.clone();
    final_player.max_stats = final_stats;
    final_player.history = history;
    final_player.social_class = social_class;
    final_player.age = age;
    final_player.height = height;
    final_player.body_weight = weight;
    final_player.gold = final_player.calculate_start_gold(rng);
    final_player.update_equipment_bonuses();
    final_player.update_max_hp_and_mana();
    final_player.update_spells_to_learn();
    final_player.hp = final_player.max_hp;
    final_player.mana = final_player.max_mana;

    Ok(final_player)
}
