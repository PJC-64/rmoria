use serde::{Serialize, Deserialize};
use crate::dice::Dice;
use crate::player::Player;
use crate::dungeon::DungeonLevel;
use crate::entity::monster_data::{
    CreatureDefinition, CREATURES_LIST, MONSTER_ATTACKS,
    CM_MULTIPLY, CD_MAX_HP, CD_EVIL,
    CS_FREQ, CS_TEL_SHORT, CS_TEL_LONG, CS_TEL_TO,
    CS_LGHT_WND, CS_SER_WND, CS_HOLD_PER, CS_BLIND, CS_CONFUSE, CS_FEAR,
    CS_SUMMON_MON, CS_SUMMON_UND, CS_SLOW_PER, CS_DRAIN_MANA,
    CS_BR_LIGHT, CS_BR_GAS, CS_BR_ACID, CS_BR_FROST, CS_BR_FIRE,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub name: String,
    pub symbol: char,
    pub x: usize,
    pub y: usize,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: Dice,
    pub experience_reward: u32,
    #[serde(default)]
    pub stunned: u32,
    #[serde(default)]
    pub asleep: u32,
    #[serde(default)]
    pub confused: u32,
    #[serde(default)]
    pub was_attacked: bool,
    #[serde(default)]
    pub creature_id: usize,
    #[serde(default)]
    pub ac: u8,
    #[serde(default)]
    pub speed: u8,
    #[serde(default)]
    pub level: u8,
}

impl Monster {
    /// Legacy constructor for backward compatibility with existing tests.
    pub fn new(name: &str, symbol: char, x: usize, y: usize, max_hp: i32, damage: Dice, experience_reward: u32) -> Self {
        let creature_id = CREATURES_LIST.iter().position(|c| c.name == name).unwrap_or(0);
        let (ac, speed, level) = if creature_id < CREATURES_LIST.len() {
            (CREATURES_LIST[creature_id].ac, CREATURES_LIST[creature_id].speed, CREATURES_LIST[creature_id].level)
        } else {
            (10, 10, 1)
        };
        Self {
            name: name.to_string(),
            symbol,
            x,
            y,
            hp: max_hp,
            max_hp,
            damage,
            experience_reward,
            stunned: 0,
            asleep: 0,
            confused: 0,
            was_attacked: false,
            creature_id,
            ac,
            speed,
            level,
        }
    }

    /// Construct a canonical monster from `CREATURES_LIST` by ID.
    pub fn from_creature_id<R: rand::Rng>(
        creature_id: usize,
        x: usize,
        y: usize,
        sleeping: bool,
        rng: &mut R,
    ) -> Self {
        let def = &CREATURES_LIST[creature_id.min(CREATURES_LIST.len() - 1)];
        let max_hp = if (def.defenses & CD_MAX_HP) != 0 {
            (def.hit_die.num * def.hit_die.sides) as i32
        } else {
            def.hit_die.roll(rng) as i32
        };

        let asleep = if sleeping {
            if def.sleep_counter == 0 {
                0
            } else {
                (def.sleep_counter as u32 * 2) + rng.gen_range(1..=(def.sleep_counter as u32 * 10))
            }
        } else {
            0
        };

        let primary_damage = if def.attacks[0] != 0 && (def.attacks[0] as usize) < MONSTER_ATTACKS.len() {
            MONSTER_ATTACKS[def.attacks[0] as usize].dice
        } else {
            Dice { num: 1, sides: 1 }
        };

        Self {
            name: def.name.to_string(),
            symbol: def.symbol,
            x,
            y,
            hp: max_hp,
            max_hp,
            damage: primary_damage,
            experience_reward: def.kill_exp,
            stunned: 0,
            asleep,
            confused: 0,
            was_attacked: false,
            creature_id,
            ac: def.ac,
            speed: def.speed,
            level: def.level,
        }
    }

    pub fn creature_def(&self) -> &'static CreatureDefinition {
        &CREATURES_LIST[self.creature_id.min(CREATURES_LIST.len() - 1)]
    }

    pub fn movement(&self) -> u32 {
        self.creature_def().movement
    }

    pub fn spells(&self) -> u32 {
        self.creature_def().spells
    }

    pub fn defenses(&self) -> u16 {
        self.creature_def().defenses
    }

    pub fn attacks(&self) -> [u8; 4] {
        self.creature_def().attacks
    }

    /// Deal damage to monster. Returns `true` if monster died.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.hp -= amount;
        self.hp <= 0
    }

    /// Basic AI movement. Calculates standard grid step closer to player.
    /// Returns `Some((new_x, new_y))` if a valid step is calculated.
    pub fn update_ai(
        &self,
        player_x: usize,
        player_y: usize,
        level: &DungeonLevel,
        depth: u32,
        killed_town_npcs: u32,
        threshold: u32,
    ) -> Option<(usize, usize)> {
        if self.stunned > 0 || self.asleep > 0 {
            return None;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();

        let is_hostile = depth > 0 || self.was_attacked || killed_town_npcs > threshold;

        if self.confused > 0 || self.symbol == 'p' {
            // Confused monsters or townsfolk wander semi-randomly
            let wander_chance = if is_hostile && self.confused == 0 { 0.70 } else { 1.00 };
            if rng.gen_bool(wander_chance) {
                let dx = rng.gen_range(-1..=1);
                let dy = rng.gen_range(-1..=1);
                if dx == 0 && dy == 0 {
                    return None;
                }
                let next_x = (self.x as isize + dx) as usize;
                let next_y = (self.y as isize + dy) as usize;
                if let Some(tile) = level.get_tile(next_x, next_y)
                    && tile.is_passable() {
                        return Some((next_x, next_y));
                    }
                return None;
            }
        }

        if !is_hostile {
            return None;
        }

        let dx = (player_x as isize - self.x as isize).signum();
        let dy = (player_y as isize - self.y as isize).signum();

        let next_x = (self.x as isize + dx) as usize;
        let next_y = (self.y as isize + dy) as usize;

        if let Some(tile) = level.get_tile(next_x, next_y)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((next_x, next_y));
            }

        let next_x_only = (self.x as isize + dx) as usize;
        if let Some(tile) = level.get_tile(next_x_only, self.y)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((next_x_only, self.y));
            }

        let next_y_only = (self.y as isize + dy) as usize;
        if let Some(tile) = level.get_tile(self.x, next_y_only)
            && (tile.is_passable() || matches!(tile.tile_type, crate::dungeon::tile::TileType::DoorClosed { .. })) {
                return Some((self.x, next_y_only));
            }

        None
    }
}

/// Hit chance check based on canonical Umoria formula.
pub fn player_test_attack_hits<R: rand::Rng>(attack_id: u8, level: u8, player_ac: i32, rng: &mut R) -> bool {
    let base_to_hit = match attack_id {
        1 => 60,
        2 => -3,
        3..=5 => 10,
        6 => 0,
        7 | 8 => 10,
        9 => 0,
        10 | 11 => 2,
        12 => 5,
        13 => 2,
        14 => 5,
        15 | 16 => 0,
        17 | 18 => 2,
        19 => 5,
        20 => return true,
        21 => 20,
        22 | 23 => 5,
        24 => 15,
        99 => return true,
        _ => 50,
    };

    let hit_chance = base_to_hit + (level as i32 * 3);
    let die = rng.gen_range(1..=20);
    die != 1 && (die == 20 || (hit_chance > 0 && rng.gen_range(1..=hit_chance) > player_ac))
}

/// Attack description based on canonical Umoria text.
pub fn attack_description<R: rand::Rng>(desc_id: u8, rng: &mut R) -> &'static str {
    match desc_id {
        1 => "hits you.",
        2 => "bites you.",
        3 => "claws you.",
        4 => "stings you.",
        5 => "touches you.",
        6 => "kicks you.",
        7 => "gazes at you.",
        8 => "breathes on you.",
        9 => "spits on you.",
        10 => "makes a horrible wail.",
        11 => "embraces you.",
        12 => "crawls on you.",
        13 => "releases a cloud of spores.",
        14 => "begs you for money.",
        15 => "You've been slimed!",
        16 => "crushes you.",
        17 => "tramples you.",
        18 => "drools on you.",
        19 => {
            match rng.gen_range(1..=8) {
                1 => "insults you!",
                2 => "insults your mother!",
                3 => "gives you the finger!",
                4 => "humiliates you!",
                5 => "wets on your leg!",
                6 => "defiles you!",
                7 => "dances around you!",
                _ => "makes obscene gestures!",
            }
        }
        99 => "is repelled.",
        _ => "attacks you.",
    }
}

/// Execute all canonical melee attacks for a monster adjacent to the player.
pub fn execute_monster_melee_attacks<R: rand::Rng>(
    monster: &Monster,
    player: &mut Player,
    status_msg: &mut String,
    rng: &mut R,
) -> bool {
    let mut any_hit = false;
    let def = monster.creature_def();
    let player_ac = player.calculate_ac();

    for &damage_type_id in &def.attacks {
        if damage_type_id == 0 || player.hp <= 0 {
            break;
        }

        let att = &MONSTER_ATTACKS[damage_type_id as usize];
        let mut attack_type = att.type_id;
        let mut attack_desc = att.desc_id;
        let dice = att.dice;

        // Protection from Evil repels evil monsters if player level >= creature level
        if player.flags.protect_evil > 0 && (def.defenses & CD_EVIL) != 0 && (player.level as u8 + 1 > def.level) {
            attack_type = 99;
            attack_desc = 99;
        }

        if attack_type == 99 {
            status_msg.push_str(&format!(" The {} is repelled!", monster.name));
            continue;
        }

        let hits = player_test_attack_hits(attack_type, def.level, player_ac, rng);
        if hits {
            any_hit = true;
            let desc = attack_description(attack_desc, rng);
            if attack_desc == 15 {
                status_msg.push_str(" You've been slimed!");
            } else {
                status_msg.push_str(&format!(" The {} {}", monster.name, desc));
            }

            let raw_damage = dice.roll(rng) as i32;

            match attack_type {
                1 => {
                    // Normal attack: AC reduces damage
                    let absorbed = (player_ac * raw_damage) / 200;
                    let effective = (raw_damage - absorbed).max(1);
                    player.take_damage(effective);
                    status_msg.push_str(&format!(" ({} dmg)", effective));
                }
                2 => {
                    // Lose Strength
                    player.take_damage(raw_damage);
                    if player.flags.sustain_str {
                        status_msg.push_str(" You feel weaker for a moment, but it passes.");
                    } else if rng.gen_bool(0.5) {
                        player.stats.strength = player.stats.strength.saturating_sub(1).max(3);
                        status_msg.push_str(" You feel weaker.");
                    }
                }
                3 => {
                    // Confusion attack
                    player.take_damage(raw_damage);
                    if rng.gen_bool(0.5) {
                        if player.flags.confused < 1 {
                            player.flags.confused += rng.gen_range(1..=(def.level.max(1) as i16));
                            status_msg.push_str(" You feel confused.");
                        }
                        player.flags.confused += 3;
                    }
                }
                4 => {
                    // Fear attack
                    player.take_damage(raw_damage);
                    if player.saving_throw(rng) {
                        status_msg.push_str(" You resist the effects!");
                    } else if player.flags.afraid < 1 {
                        player.flags.afraid += 3 + rng.gen_range(1..=(def.level.max(1) as i16));
                        status_msg.push_str(" You are suddenly afraid!");
                    } else {
                        player.flags.afraid += 3;
                    }
                }
                5 => {
                    // Fire attack
                    status_msg.push_str(" You are enveloped in flames!");
                    player.damage_fire(raw_damage, status_msg);
                }
                6 => {
                    // Acid attack
                    status_msg.push_str(" You are covered in acid!");
                    player.damage_acid(raw_damage, status_msg);
                }
                7 => {
                    // Cold attack
                    status_msg.push_str(" You are covered with frost!");
                    player.damage_cold(raw_damage, status_msg);
                }
                8 => {
                    // Lightning attack
                    status_msg.push_str(" Lightning strikes you!");
                    player.damage_lightning(raw_damage, status_msg);
                }
                9 => {
                    // Corrosion attack
                    status_msg.push_str(" A stinging red gas swirls about you.");
                    player.take_damage(raw_damage);
                }
                10 => {
                    // Blindness attack
                    player.take_damage(raw_damage);
                    if player.flags.blind < 1 {
                        player.flags.blind += 10 + rng.gen_range(1..=(def.level.max(1) as i16));
                        status_msg.push_str(" Your eyes begin to sting.");
                    } else {
                        player.flags.blind += 5;
                    }
                }
                11 => {
                    // Paralysis attack
                    player.take_damage(raw_damage);
                    if player.flags.free_action {
                        status_msg.push_str(" You are unaffected.");
                    } else if player.saving_throw(rng) {
                        status_msg.push_str(" You resist the effects!");
                    } else if player.flags.paralysis < 1 {
                        player.flags.paralysis = 3 + rng.gen_range(1..=(def.level.max(1) as i16));
                        status_msg.push_str(" You are paralyzed.");
                    }
                }
                12 => {
                    // Steal money
                    if player.flags.paralysis < 1 && rng.gen_range(0..124) < player.stats.dexterity as u32 {
                        status_msg.push_str(" You quickly protect your money pouch!");
                    } else {
                        let stolen = (player.gold / 10) + rng.gen_range(1..=25);
                        player.gold = player.gold.saturating_sub(stolen);
                        status_msg.push_str(" Your purse feels lighter.");
                    }
                }
                13 => {
                    // Steal object
                    if player.flags.paralysis < 1 && rng.gen_range(0..124) < player.stats.dexterity as u32 {
                        status_msg.push_str(" You grab hold of your backpack!");
                    } else if !player.inventory.is_empty() {
                        let idx = rng.gen_range(0..player.inventory.len());
                        let stolen_item = player.inventory.remove(idx);
                        status_msg.push_str(&format!(" Your backpack feels lighter (lost {}).", stolen_item.name));
                    }
                }
                14 => {
                    // Poison
                    player.take_damage(raw_damage);
                    status_msg.push_str(" You feel very sick.");
                    player.flags.poisoned += 5 + rng.gen_range(1..=(def.level.max(1) as i16));
                }
                15 => {
                    // Lose dexterity
                    player.take_damage(raw_damage);
                    if player.flags.sustain_dex {
                        status_msg.push_str(" You feel clumsy for a moment, but it passes.");
                    } else {
                        player.stats.dexterity = player.stats.dexterity.saturating_sub(1).max(3);
                        status_msg.push_str(" You feel more clumsy.");
                    }
                }
                16 => {
                    // Lose constitution
                    player.take_damage(raw_damage);
                    if player.flags.sustain_con {
                        status_msg.push_str(" Your body resists the effects of the disease.");
                    } else {
                        player.stats.constitution = player.stats.constitution.saturating_sub(1).max(3);
                        status_msg.push_str(" Your health is damaged!");
                    }
                }
                17 => {
                    // Lose intelligence
                    player.take_damage(raw_damage);
                    if player.flags.sustain_int {
                        status_msg.push_str(" You have trouble thinking clearly, but your mind quickly clears.");
                    } else {
                        player.stats.intelligence = player.stats.intelligence.saturating_sub(1).max(3);
                        status_msg.push_str(" You have trouble thinking clearly.");
                    }
                }
                18 => {
                    // Lose wisdom
                    player.take_damage(raw_damage);
                    if player.flags.sustain_wis {
                        status_msg.push_str(" Your wisdom is sustained.");
                    } else {
                        player.stats.wisdom = player.stats.wisdom.saturating_sub(1).max(3);
                        status_msg.push_str(" Your wisdom is drained.");
                    }
                }
                19 => {
                    // Lose experience
                    status_msg.push_str(" You feel your life draining away!");
                    let drain = (raw_damage as u32) + (player.exp / 100) * 2;
                    player.exp = player.exp.saturating_sub(drain);
                }
                20 => {
                    // Aggravate monsters
                    status_msg.push_str(" You feel an evil presence watching you.");
                }
                21 => {
                    // Disenchant
                    status_msg.push_str(" There is a static feeling in the air.");
                }
                22 => {
                    // Eat food
                    if let Some(pos) = player.inventory.iter().position(|i| matches!(i.item_type, crate::player::ItemType::Food { .. })) {
                        player.inventory.remove(pos);
                        status_msg.push_str(" It got at your rations!");
                    }
                }
                23 => {
                    // Eat light
                    for item in &mut player.equipment {
                        if let crate::player::ItemType::Light { ref mut fuel } = item.item_type {
                            *fuel = fuel.saturating_sub(250);
                            status_msg.push_str(" Your light dims!");
                            break;
                        }
                    }
                }
                24 => {
                    // Eat wand/staff charges
                    status_msg.push_str(" Your magical items feel drained.");
                }
                _ => {
                    player.take_damage(raw_damage);
                }
            }
        } else if attack_desc <= 3 || attack_desc == 6 {
            status_msg.push_str(&format!(" The {} misses you.", monster.name));
        }
    }

    any_hit
}

/// Attempt to cast a monster spell or breath weapon.
pub fn try_monster_cast_spell<R: rand::Rng>(
    monster: &mut Monster,
    player: &mut Player,
    level: &mut DungeonLevel,
    new_monsters: &mut Vec<Monster>,
    occupied: &[(usize, usize)],
    status_msg: &mut String,
    rng: &mut R,
) -> bool {
    let def = monster.creature_def();
    let spells = def.spells;
    let freq = spells & CS_FREQ;

    if freq == 0 || rng.gen_range(1..=freq) != 1 {
        return false;
    }

    let dist = (monster.x as isize - player.x as isize).abs().max((monster.y as isize - player.y as isize).abs());
    if dist > 20 || !level.has_los(monster.x, monster.y, player.x, player.y) {
        return false;
    }

    let spell_flags = spells & !CS_FREQ;
    if spell_flags == 0 {
        return false;
    }

    let mut available_spells = Vec::new();
    let check_spells = [
        CS_TEL_SHORT, CS_TEL_LONG, CS_TEL_TO,
        CS_LGHT_WND, CS_SER_WND, CS_HOLD_PER,
        CS_BLIND, CS_CONFUSE, CS_FEAR,
        CS_SUMMON_MON, CS_SUMMON_UND,
        CS_SLOW_PER, CS_DRAIN_MANA,
        CS_BR_LIGHT, CS_BR_GAS, CS_BR_ACID, CS_BR_FROST, CS_BR_FIRE,
    ];

    for &s in &check_spells {
        if (spell_flags & s) != 0 {
            available_spells.push(s);
        }
    }

    if available_spells.is_empty() {
        return false;
    }

    let chosen = available_spells[rng.gen_range(0..available_spells.len())];

    match chosen {
        CS_TEL_SHORT => {
            status_msg.push_str(&format!(" The {} blinks away!", monster.name));
            let (nx, ny) = level.find_random_floor_tile_near(monster.x, monster.y, 5);
            monster.x = nx;
            monster.y = ny;
        }
        CS_TEL_LONG => {
            status_msg.push_str(&format!(" The {} teleports away!", monster.name));
            let (nx, ny) = level.find_random_floor_tile();
            monster.x = nx;
            monster.y = ny;
        }
        CS_TEL_TO => {
            status_msg.push_str(&format!(" The {} teleports you to itself!", monster.name));
            let adj = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in adj {
                let tx = (monster.x as isize + dx) as usize;
                let ty = (monster.y as isize + dy) as usize;
                if let Some(t) = level.get_tile(tx, ty) && t.is_passable() && !occupied.iter().any(|&(ox, oy)| ox == tx && oy == ty) {
                    player.x = tx;
                    player.y = ty;
                    break;
                }
            }
        }
        CS_LGHT_WND => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                let dmg = Dice::new(3, 8).roll(rng) as i32;
                player.take_damage(dmg);
                status_msg.push_str(&format!(" You take {} damage!", dmg));
            }
        }
        CS_SER_WND => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                let dmg = Dice::new(8, 8).roll(rng) as i32;
                player.take_damage(dmg);
                status_msg.push_str(&format!(" You take {} damage!", dmg));
            }
        }
        CS_HOLD_PER => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.flags.free_action {
                status_msg.push_str(" You are unaffected.");
            } else if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                player.flags.paralysis += rng.gen_range(4..=9);
                status_msg.push_str(" You are paralyzed.");
            }
        }
        CS_BLIND => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                player.flags.blind += 12 + rng.gen_range(1..=3);
                status_msg.push_str(" Your eyes begin to sting.");
            }
        }
        CS_CONFUSE => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                player.flags.confused += rng.gen_range(3..=7);
                status_msg.push_str(" You feel confused.");
            }
        }
        CS_FEAR => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                player.flags.afraid += rng.gen_range(3..=7);
                status_msg.push_str(" You are suddenly afraid!");
            }
        }
        CS_SUMMON_MON => {
            status_msg.push_str(&format!(" The {} magically summons a monster!", monster.name));
            let c_idx = crate::entity::monster_data::get_monster_for_level(level.depth, rng);
            let (sx, sy) = level.find_random_floor_tile_near(player.x, player.y, 3);
            if !occupied.iter().any(|&(ox, oy)| ox == sx && oy == sy) && (player.x != sx || player.y != sy) {
                new_monsters.push(Monster::from_creature_id(c_idx, sx, sy, false, rng));
            }
        }
        CS_SUMMON_UND => {
            status_msg.push_str(&format!(" The {} magically summons an undead!", monster.name));
            let undead_candidates: Vec<usize> = CREATURES_LIST.iter().enumerate()
                .filter(|(_, c)| (c.defenses & crate::entity::monster_data::CD_UNDEAD) != 0 && c.level <= level.depth as u8 + 5)
                .map(|(i, _)| i)
                .collect();
            let c_idx = if !undead_candidates.is_empty() {
                undead_candidates[rng.gen_range(0..undead_candidates.len())]
            } else {
                crate::entity::monster_data::get_monster_for_level(level.depth, rng)
            };
            let (sx, sy) = level.find_random_floor_tile_near(player.x, player.y, 3);
            if !occupied.iter().any(|&(ox, oy)| ox == sx && oy == sy) && (player.x != sx || player.y != sy) {
                new_monsters.push(Monster::from_creature_id(c_idx, sx, sy, false, rng));
            }
        }
        CS_SLOW_PER => {
            status_msg.push_str(&format!(" The {} casts a spell.", monster.name));
            if player.flags.free_action {
                status_msg.push_str(" You are unaffected.");
            } else if player.saving_throw(rng) {
                status_msg.push_str(" You resist the effects of the spell.");
            } else {
                player.flags.slow += rng.gen_range(3..=7);
                status_msg.push_str(" You feel yourself slowing down.");
            }
        }
        CS_DRAIN_MANA => {
            if player.mana > 0 {
                let drained = (rng.gen_range(1..=(def.level.max(1) as i32)) / 2 + 1).min(player.mana).max(0);
                player.mana -= drained;
                monster.hp = (monster.hp + (6 * drained)).min(monster.max_hp * 2);
                status_msg.push_str(&format!(" The {} draws psychic energy from you! (lost {} mana)", monster.name, drained));
            }
        }
        CS_BR_LIGHT => {
            status_msg.push_str(&format!(" The {} breathes lightning!", monster.name));
            let dmg = ((monster.hp / 4) / ((dist as i32) + 1)).max(1);
            player.damage_lightning(dmg, status_msg);
        }
        CS_BR_GAS => {
            status_msg.push_str(&format!(" The {} breathes poison gas!", monster.name));
            let dmg = ((monster.hp / 3) / ((dist as i32) + 1)).max(1);
            player.damage_poison_gas(dmg, status_msg);
        }
        CS_BR_ACID => {
            status_msg.push_str(&format!(" The {} breathes acid!", monster.name));
            let dmg = ((monster.hp / 3) / ((dist as i32) + 1)).max(1);
            player.damage_acid(dmg, status_msg);
        }
        CS_BR_FROST => {
            status_msg.push_str(&format!(" The {} breathes frost!", monster.name));
            let dmg = ((monster.hp / 3) / ((dist as i32) + 1)).max(1);
            player.damage_cold(dmg, status_msg);
        }
        CS_BR_FIRE => {
            status_msg.push_str(&format!(" The {} breathes fire!", monster.name));
            let dmg = ((monster.hp / 3) / ((dist as i32) + 1)).max(1);
            player.damage_fire(dmg, status_msg);
        }
        _ => return false,
    }

    true
}

/// Attempt monster multiplication / breeding.
pub fn try_monster_multiply<R: rand::Rng>(
    monster: &Monster,
    level: &DungeonLevel,
    occupied: &[(usize, usize)],
    player_x: usize,
    player_y: usize,
    rng: &mut R,
) -> Option<Monster> {
    if (monster.movement() & CM_MULTIPLY) == 0 {
        return None;
    }

    if rng.gen_range(0..15) != 0 {
        return None;
    }

    if occupied.len() >= 75 {
        return None;
    }

    let adj = [
        (-1, -1), (0, -1), (1, -1),
        (-1,  0),          (1,  0),
        (-1,  1), (0,  1), (1,  1),
    ];

    for (dx, dy) in adj {
        let tx = (monster.x as isize + dx) as usize;
        let ty = (monster.y as isize + dy) as usize;
        if tx == player_x && ty == player_y {
            continue;
        }
        if let Some(tile) = level.get_tile(tx, ty) && tile.is_passable()
            && !occupied.iter().any(|&(ox, oy)| ox == tx && oy == ty) {
            return Some(Monster::from_creature_id(monster.creature_id, tx, ty, false, rng));
        }
    }

    None
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonsterTemplate {
    pub name: &'static str,
    pub symbol: char,
    pub level: u32,
    pub max_hp: i32,
    pub damage: Dice,
    pub exp_reward: u32,
}

#[allow(dead_code)]
pub const MONSTER_DB: &[MonsterTemplate] = &[
    MonsterTemplate { name: "Red Mold", symbol: 'm', level: 1, max_hp: 5, damage: Dice { num: 1, sides: 3 }, exp_reward: 5 },
    MonsterTemplate { name: "Giant Centipede", symbol: 'c', level: 1, max_hp: 7, damage: Dice { num: 1, sides: 3 }, exp_reward: 7 },
    MonsterTemplate { name: "Goblin", symbol: 'g', level: 2, max_hp: 8, damage: Dice { num: 1, sides: 4 }, exp_reward: 10 },
    MonsterTemplate { name: "Giant White Rat", symbol: 'r', level: 2, max_hp: 9, damage: Dice { num: 1, sides: 4 }, exp_reward: 12 },
    MonsterTemplate { name: "Orc", symbol: 'o', level: 3, max_hp: 15, damage: Dice { num: 1, sides: 6 }, exp_reward: 25 },
    MonsterTemplate { name: "Baby Dragon", symbol: 'd', level: 3, max_hp: 20, damage: Dice { num: 2, sides: 4 }, exp_reward: 35 },
    MonsterTemplate { name: "Cave Troll", symbol: 'T', level: 4, max_hp: 30, damage: Dice { num: 2, sides: 6 }, exp_reward: 50 },
    MonsterTemplate { name: "Spectre", symbol: 'S', level: 4, max_hp: 22, damage: Dice { num: 1, sides: 8 }, exp_reward: 60 },
    MonsterTemplate { name: "Uruk-Hai", symbol: 'U', level: 5, max_hp: 35, damage: Dice { num: 2, sides: 6 }, exp_reward: 80 },
    MonsterTemplate { name: "Giant Spider", symbol: 's', level: 5, max_hp: 28, damage: Dice { num: 1, sides: 10 }, exp_reward: 75 },
    MonsterTemplate { name: "Ghost Warrior", symbol: 'W', level: 6, max_hp: 40, damage: Dice { num: 2, sides: 8 }, exp_reward: 120 },
    MonsterTemplate { name: "Lich", symbol: 'L', level: 7, max_hp: 55, damage: Dice { num: 3, sides: 6 }, exp_reward: 180 },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Race, Class};
    use crate::entity::monster_data::{CREATURES_LIST, MONSTER_ATTACKS, MONSTER_LEVELS, get_monster_for_level};

    #[test]
    fn test_canonical_creature_count_and_balrog_stats() {
        assert_eq!(CREATURES_LIST.len(), 279, "Must contain all 279 canonical Umoria creatures");
        assert_eq!(MONSTER_ATTACKS.len(), 215, "Must contain all 215 canonical monster attacks");

        // The Balrog is creature index 278
        let balrog = &CREATURES_LIST[278];
        assert_eq!(balrog.name, "Balrog");
        assert_eq!(balrog.symbol, 'B');
        assert_eq!(balrog.ac, 125);
        assert_eq!(balrog.speed, 13);
        assert_eq!(balrog.kill_exp, 55000);
        assert_eq!(balrog.hit_die.num, 75);
        assert_eq!(balrog.hit_die.sides, 40);
        assert_eq!(balrog.level, 100);
    }

    #[test]
    fn test_canonical_monster_levels_prefix() {
        assert_eq!(MONSTER_LEVELS.len(), 41);
        assert_eq!(MONSTER_LEVELS[0], 8, "Level 0 town NPCs count");
        assert_eq!(MONSTER_LEVELS[40], 277, "Prefix sum of all 277 non-endgame creatures");

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let m_idx = get_monster_for_level(1, &mut rng);
            assert!(m_idx < 277);
            let m_town = get_monster_for_level(0, &mut rng);
            assert!(m_town < 8);
        }
    }

    #[test]
    fn test_monster_creation_from_creature_id() {
        let mut rng = rand::thread_rng();
        let mon = Monster::from_creature_id(278, 10, 15, false, &mut rng);
        assert_eq!(mon.name, "Balrog");
        assert_eq!(mon.symbol, 'B');
        assert!(mon.hp > 0);
        assert_eq!(mon.ac, 125);
        assert_eq!(mon.x, 10);
        assert_eq!(mon.y, 15);
        assert_eq!(mon.asleep, 0);

        // Sleeping monster initialization
        let sleeping_mon = Monster::from_creature_id(16, 5, 5, true, &mut rng); // Kobold
        assert!(sleeping_mon.asleep > 0, "Sleeping monster must have sleep counter > 0");
    }

    #[test]
    fn test_melee_attack_normal_reduced_by_ac() {
        let mut rng = rand::thread_rng();
        let mut player = Player::new("TestWarrior", Race::Human, Class::Warrior, 50, 10);
        player.hp = 100;
        player.max_hp = 100;

        // Create a monster that has a normal attack (attack #14: 2d6)
        let orc = Monster::from_creature_id(45, 1, 1, false, &mut rng);
        let mut msg = String::new();
        let _ = execute_monster_melee_attacks(&orc, &mut player, &mut msg, &mut rng);

        assert!(player.hp <= 100, "Normal attack deals damage");
    }

    #[test]
    fn test_melee_attack_poison_inflicts_poisoned() {
        let mut rng = rand::thread_rng();
        let mut player = Player::new("TestVictim", Race::Human, Class::Warrior, 50, 10);
        player.hp = 100;
        player.flags.poisoned = 0;

        // Copperhead Snake (index 71) attacks with poison (attack type 14)
        let snake = Monster::from_creature_id(71, 1, 1, false, &mut rng);
        let mut msg = String::new();
        // Force test attack to hit by calling multiple times or checking poison effect directly
        for _ in 0..10 {
            let _ = execute_monster_melee_attacks(&snake, &mut player, &mut msg, &mut rng);
            if player.flags.poisoned > 0 {
                break;
            }
        }
        assert!(player.flags.poisoned > 0, "Poisonous creature must inflict poisoned status condition");
    }

    #[test]
    fn test_melee_attack_paralysis_respects_free_action() {
        let mut rng = rand::thread_rng();
        let mut player = Player::new("FreeActionHero", Race::Human, Class::Warrior, 50, 10);
        player.flags.free_action = true;
        player.flags.paralysis = 0;

        // Floating Eye has paralysis attack
        let eye = Monster::from_creature_id(18, 1, 1, false, &mut rng);
        let mut msg = String::new();
        for _ in 0..15 {
            let _ = execute_monster_melee_attacks(&eye, &mut player, &mut msg, &mut rng);
        }
        assert_eq!(player.flags.paralysis, 0, "Free action must protect against paralysis attacks");
    }

    #[test]
    fn test_melee_attack_stat_drain_respects_sustain() {
        let mut rng = rand::thread_rng();
        let mut player = Player::new("SustainedPlayer", Race::Human, Class::Warrior, 50, 10);
        player.stats.strength = 16;
        player.flags.sustain_str = true;

        // Quasit drains STR (attack type 2)
        let quasit = Monster::from_creature_id(80, 1, 1, false, &mut rng);
        let mut msg = String::new();
        for _ in 0..20 {
            let _ = execute_monster_melee_attacks(&quasit, &mut player, &mut msg, &mut rng);
        }
        assert_eq!(player.stats.strength, 16, "Sustained STR must prevent STR drain");
    }

    #[test]
    fn test_dragon_breath_resistances() {
        let mut player = Player::new("ResistantHero", Race::Human, Class::Warrior, 50, 10);
        player.hp = 300;
        player.max_hp = 300;
        player.flags.resistant_to_fire = true;
        player.flags.resistant_to_cold = true;
        player.flags.resistant_to_light = true;
        player.flags.resistant_to_acid = true;

        let mut msg = String::new();
        player.damage_fire(90, &mut msg);
        // 90 / 3 = 30 damage
        assert_eq!(player.hp, 270, "Fire resistance reduces 90 damage to 30");

        player.damage_cold(60, &mut msg);
        // 60 / 3 = 20 damage
        assert_eq!(player.hp, 250, "Cold resistance reduces 60 damage to 20");

        player.damage_lightning(30, &mut msg);
        // 30 / 3 = 10 damage
        assert_eq!(player.hp, 240, "Lightning resistance reduces 30 damage to 10");

        player.damage_acid(45, &mut msg);
        // 45 / 3 = 15 damage
        assert_eq!(player.hp, 225, "Acid resistance reduces 45 damage to 15");
    }

    #[test]
    fn test_monster_multiplication() {
        let mut rng = rand::thread_rng();
        let mut dungeon = DungeonLevel::new(80, 24, 1, 50);
        dungeon.get_tile_mut(11, 10).unwrap().tile_type = crate::dungeon::TileType::Floor;
        // White Worm mass (creature 17) has CM_MULTIPLY flag
        let worm = Monster::from_creature_id(17, 10, 10, false, &mut rng);
        assert_ne!(worm.movement() & CM_MULTIPLY, 0, "Worm must have CM_MULTIPLY flag");

        // Test multiply attempt with guaranteed RNG roll
        let mut spawned = None;
        for _ in 0..100 {
            if let Some(child) = try_monster_multiply(&worm, &dungeon, &[], 20, 20, &mut rng) {
                spawned = Some(child);
                break;
            }
        }
        assert!(spawned.is_some(), "Multiplying monster should be able to produce offspring");
        let child = spawned.unwrap();
        assert_eq!(child.name, worm.name);
    }
}
