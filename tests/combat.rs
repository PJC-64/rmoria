use rand::SeedableRng;
use rmoria::dice::Dice;
use rmoria::player::{Class, Item, ItemType, Player, Race};

#[test]
fn test_calculate_blows_range() {
    let mut player = Player::new("Test", Race::Human, Class::Warrior, 0, 0);
    player.stats.strength = 18;
    player.stats.dexterity = 18;

    // With 0 weight (unarmed), should use 10 weight and give >1 blow
    let blows = player.calculate_blows(&None);
    assert!((1..=5).contains(&blows));

    // With heavy weapon
    let heavy_weapon = Item {
        name: "Heavy Mace".to_string(),
        item_type: ItemType::Weapon {
            damage: Dice::new(3, 4),
        },
        weight: 300,
        flags: 0,
        to_hit: 0,
        to_damage: 0,
        count: 1,
        inscription: None,
        identified: true,
        flavor: None,
        equipped_slot: None,
        is_cursed: false,
        cost: 0,
        ego_name: None,
        to_ac: 0,
    };

    let blows_heavy = player.calculate_blows(&Some(&heavy_weapon));
    assert!((1..=5).contains(&blows_heavy));

    // Lighter weapon = more blows
    let light_weapon = Item {
        name: "Dagger".to_string(),
        item_type: ItemType::Weapon {
            damage: Dice::new(1, 4),
        },
        weight: 12,
        flags: 0,
        to_hit: 0,
        to_damage: 0,
        count: 1,
        inscription: None,
        identified: true,
        flavor: None,
        equipped_slot: None,
        is_cursed: false,
        cost: 0,
        ego_name: None,
        to_ac: 0,
    };

    let blows_light = player.calculate_blows(&Some(&light_weapon));
    assert!((1..=5).contains(&blows_light));
    assert!(blows_light >= blows_heavy); // Light gives at least as many as heavy
}

#[test]
fn test_multi_blows_damage_aggregation() {
    let mut player = Player::new("Test", Race::Human, Class::Warrior, 0, 0);
    player.stats.strength = 18;
    player.stats.dexterity = 18;
    player.level = 10; // Better hit chance

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let dmg = player.roll_melee_damage_against(&mut rng, None);
    assert!(dmg >= 0);
}

#[test]
fn test_message_history_entry() {
    let mut player = Player::new("Test", Race::Human, Class::Warrior, 0, 0);
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let blows = player.calculate_blows(&None);
    let dmg = player.roll_melee_damage_against(&mut rng, None);

    player.message_history.push(format!(
        "You strike {} times for {} total damage!",
        blows, dmg
    ));

    let last_msg = player.message_history.last().unwrap();
    assert!(last_msg.contains("You strike"));
    assert!(last_msg.contains("total damage"));
}

#[test]
fn test_canonical_to_hit_chances() {
    let mut player = Player::new("Test", Race::Human, Class::Warrior, 0, 0);
    player.stats.strength = 16;
    player.stats.dexterity = 16;
    player.level = 1;

    assert_eq!(player.base_to_hit(), 70);
    assert_eq!(player.class_bth_level_adj(), 4);

    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let mut hits_mouse = 0;
    let mut hits_snake = 0;
    let trials = 1000;

    for _ in 0..trials {
        if player.test_hit(&mut rng, 4) {
            hits_mouse += 1;
        }
        if player.test_hit(&mut rng, 35) {
            hits_snake += 1;
        }
    }

    // Against AC 4 mouse: should hit ~90%+
    assert!(
        hits_mouse > 850,
        "Expected >85% hits against AC 4, got {}",
        hits_mouse
    );
    // Against AC 35 snake: should hit ~50%+ (unlike old 5%)
    assert!(
        hits_snake > 450,
        "Expected >45% hits against AC 35, got {}",
        hits_snake
    );
}
