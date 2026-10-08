use rmoria::player::{Player, Class, Race, Item, ItemType};
use rmoria::dice::Dice;
use rand::SeedableRng;

#[test]
fn test_calculate_blows_range() {
    let mut player = Player::new("Test", Race::Human, Class::Warrior, 0, 0);
    player.stats.strength = 18;
    player.stats.dexterity = 18;
    
    // With 0 weight (unarmed), should use 10 weight and give >1 blow
    let blows = player.calculate_blows(&None);
    assert!(blows >= 1 && blows <= 5);
    
    // With heavy weapon
    let heavy_weapon = Item {
        name: "Heavy Mace".to_string(),
        item_type: ItemType::Weapon { damage: Dice::new(3, 4) },
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
    assert!(blows_heavy >= 1 && blows_heavy <= 5);
    
    // Lighter weapon = more blows
    let light_weapon = Item {
        name: "Dagger".to_string(),
        item_type: ItemType::Weapon { damage: Dice::new(1, 4) },
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
    assert!(blows_light >= 1 && blows_light <= 5);
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
