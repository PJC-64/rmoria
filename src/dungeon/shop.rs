use serde::{Serialize, Deserialize};
use crossterm::event::{KeyEvent, KeyCode};
use rand::Rng;

use crate::player::{Player, Item, ItemType, Race};
use crate::ScreenMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreOwner {
    pub name: &'static str,
    pub max_cost: u32,
    pub max_inflate: u32,
    pub min_inflate: u32,
    pub haggles_per: u32,
    pub race: usize, // 0..7
    pub max_insults: u32,
}

pub const MAX_OWNERS: usize = 18;
pub const STORE_OWNERS: [StoreOwner; MAX_OWNERS] = [
    // General Store (Store 0)
    StoreOwner { name: "Erick the Honest       (Human)      General Store", max_cost: 250,   max_inflate: 175, min_inflate: 108, haggles_per: 4, race: 0, max_insults: 12 },
    // Armory (Store 1)
    StoreOwner { name: "Mauglin the Grumpy     (Dwarf)      Armory",        max_cost: 32000, max_inflate: 200, min_inflate: 112, haggles_per: 4, race: 5, max_insults: 5 },
    // Weaponsmith (Store 2)
    StoreOwner { name: "Arndal Beast-Slayer    (Half-Elf)   Weaponsmith",   max_cost: 10000, max_inflate: 185, min_inflate: 110, haggles_per: 5, race: 1, max_insults: 8 },
    // Temple (Store 3)
    StoreOwner { name: "Hardblow the Humble    (Human)      Temple",        max_cost: 3500,  max_inflate: 175, min_inflate: 109, haggles_per: 6, race: 0, max_insults: 15 },
    // Alchemist (Store 4)
    StoreOwner { name: "Ga-nat the Greedy      (Gnome)      Alchemist",     max_cost: 12000, max_inflate: 220, min_inflate: 115, haggles_per: 4, race: 4, max_insults: 9 },
    // Magic Shop (Store 5)
    StoreOwner { name: "Valeria Starshine      (Elf)        Magic Shop",    max_cost: 32000, max_inflate: 175, min_inflate: 110, haggles_per: 5, race: 2, max_insults: 11 },

    // General Store (Store 0, Variant 1)
    StoreOwner { name: "Andy the Friendly      (Halfling)   General Store", max_cost: 200,   max_inflate: 170, min_inflate: 108, haggles_per: 5, race: 3, max_insults: 15 },
    // Armory (Store 1, Variant 1)
    StoreOwner { name: "Darg-Low the Grim      (Human)      Armory",        max_cost: 10000, max_inflate: 190, min_inflate: 111, haggles_per: 4, race: 0, max_insults: 9 },
    // Weaponsmith (Store 2, Variant 1)
    StoreOwner { name: "Oglign Dragon-Slayer   (Dwarf)      Weaponsmith",   max_cost: 32000, max_inflate: 195, min_inflate: 112, haggles_per: 4, race: 5, max_insults: 8 },
    // Temple (Store 3, Variant 1)
    StoreOwner { name: "Gunnar the Paladin     (Human)      Temple",        max_cost: 5000,  max_inflate: 185, min_inflate: 110, haggles_per: 5, race: 0, max_insults: 23 },
    // Alchemist (Store 4, Variant 1)
    StoreOwner { name: "Mauser the Chemist     (Half-Elf)   Alchemist",     max_cost: 10000, max_inflate: 190, min_inflate: 111, haggles_per: 5, race: 1, max_insults: 8 },
    // Magic Shop (Store 5, Variant 1)
    StoreOwner { name: "Gopher the Great!      (Gnome)      Magic Shop",    max_cost: 20000, max_inflate: 215, min_inflate: 113, haggles_per: 6, race: 4, max_insults: 10 },

    // General Store (Store 0, Variant 2)
    StoreOwner { name: "Lyar-el the Comely     (Elf)        General Store", max_cost: 300,   max_inflate: 165, min_inflate: 107, haggles_per: 6, race: 2, max_insults: 18 },
    // Armory (Store 1, Variant 2)
    StoreOwner { name: "Mauglim the Horrible   (Half-Orc)   Armory",        max_cost: 3000,  max_inflate: 200, min_inflate: 113, haggles_per: 5, race: 6, max_insults: 9 },
    // Weaponsmith (Store 2, Variant 2)
    StoreOwner { name: "Ithyl-Mak the Beastly  (Half-Troll) Weaponsmith",   max_cost: 3000,  max_inflate: 210, min_inflate: 115, haggles_per: 6, race: 7, max_insults: 8 },
    // Temple (Store 3, Variant 2)
    StoreOwner { name: "Delilah the Pure       (Half-Elf)   Temple",        max_cost: 25000, max_inflate: 180, min_inflate: 107, haggles_per: 6, race: 1, max_insults: 20 },
    // Alchemist (Store 4, Variant 2)
    StoreOwner { name: "Wizzle the Chaotic     (Halfling)   Alchemist",     max_cost: 10000, max_inflate: 190, min_inflate: 110, haggles_per: 6, race: 3, max_insults: 8 },
    // Magic Shop (Store 5, Variant 2)
    StoreOwner { name: "Inglorian the Mage     (Human?)     Magic Shop",    max_cost: 32000, max_inflate: 200, min_inflate: 110, haggles_per: 7, race: 0, max_insults: 10 },
];

pub fn race_index(race: Race) -> usize {
    match race {
        Race::Human => 0,
        Race::HalfElf => 1,
        Race::Elf => 2,
        Race::Halfling => 3,
        Race::Gnome => 4,
        Race::Dwarf => 5,
        Race::HalfOrc => 6,
        Race::HalfTroll => 7,
    }
}

pub const RACE_GOLD_ADJUSTMENTS: [[u32; 8]; 8] = [
    // Hum, HfE, Elf, Hal, Gno, Dwa, HfO, HfT
    [100, 105, 105, 110, 113, 115, 120, 125], // Human
    [110, 100, 100, 105, 110, 120, 125, 130], // Half-Elf
    [110, 105, 100, 105, 110, 120, 125, 130], // Elf
    [115, 110, 105,  95, 105, 110, 115, 130], // Halfling
    [115, 115, 110, 105,  95, 110, 115, 130], // Gnome
    [115, 120, 120, 110, 110,  95, 125, 135], // Dwarf
    [115, 120, 125, 115, 115, 130, 110, 115], // Half-Orc
    [110, 115, 115, 110, 110, 130, 110, 110], // Half-Troll
];

pub fn player_stat_adjustment_charisma(chr: i16) -> u32 {
    if chr > 117 {
        90
    } else if chr > 107 {
        92
    } else if chr > 87 {
        94
    } else if chr > 67 {
        96
    } else if chr > 18 {
        98
    } else {
        match chr {
            18 => 100,
            17 => 101,
            16 => 102,
            15 => 103,
            14 => 104,
            13 => 106,
            12 => 108,
            11 => 110,
            10 => 112,
            9 => 114,
            8 => 116,
            7 => 118,
            6 => 120,
            5 => 122,
            4 => 125,
            _ => 130,
        }
    }
}

pub const SPEECH_SALE_ACCEPTED: &[&str] = &[
    "Done!",
    "Accepted!",
    "Fine.",
    "Agreed!",
    "Ok.",
    "Taken!",
    "You drive a hard bargain, but taken.",
    "You'll force me bankrupt, but it's a deal.",
    "Sigh. I'll take it.",
    "My poor sick children may starve, but done!",
    "Finally! I accept.",
    "Robbed again.",
    "A pleasure to do business with you!",
    "My spouse will skin me, but accepted.",
];

pub const SPEECH_SELLING_HAGGLE_FINAL: &[&str] = &[
    "%A2 is my final offer; take it or leave it.",
    "I'll give you no more than %A2.",
    "My patience grows thin. %A2 is final.",
];

pub const SPEECH_SELLING_HAGGLE: &[&str] = &[
    "%A1 for such a fine item? HA! No less than %A2.",
    "%A1 is an insult! Try %A2 gold pieces.",
    "%A1?!? You would rob my poor starving children?",
    "Why, I'll take no less than %A2 gold pieces.",
    "Ha! No less than %A2 gold pieces.",
    "Thou knave! No less than %A2 gold pieces.",
    "%A1 is far too little, how about %A2?",
    "I paid more than %A1 for it myself, try %A2.",
    "%A1? Are you mad?!? How about %A2 gold pieces?",
    "As scrap this would bring %A1. Try %A2 in gold.",
    "May the fleas of 1000 Orcs molest you. I want %A2.",
    "My mother you can get for %A1, this costs %A2.",
    "May your chickens grow lips. I want %A2 in gold!",
    "Sell this for such a pittance? Give me %A2 gold.",
    "May the Balrog find you tasty! %A2 gold pieces?",
    "Your mother was a Troll! %A2 or I'll tell.",
];

pub const SPEECH_BUYING_HAGGLE_FINAL: &[&str] = &[
    "I'll pay no more than %A1; take it or leave it.",
    "You'll get no more than %A1 from me.",
    "%A1 and that's final.",
];

pub const SPEECH_BUYING_HAGGLE: &[&str] = &[
    "%A2 for that piece of junk? No more than %A1.",
    "For %A2 I could own ten of those. Try %A1.",
    "%A2? NEVER! %A1 is more like it.",
    "Let's be reasonable. How about %A1 gold pieces?",
    "%A1 gold for that junk, no more.",
    "%A1 gold pieces and be thankful for it!",
    "%A1 gold pieces and not a copper more.",
    "%A2 gold? HA! %A1 is more like it.",
    "Try about %A1 gold.",
    "I wouldn't pay %A2 for your children, try %A1.",
    "*CHOKE* For that!? Let's say %A1.",
    "How about %A1?",
    "That looks war surplus! Say %A1 gold.",
    "I'll buy it as scrap for %A1.",
    "%A2 is too much, let us say %A1 gold.",
];

pub const SPEECH_INSULTED_HAGGLED_DONE: &[&str] = &[
    "ENOUGH! You have abused me once too often!",
    "THAT DOES IT! You shall waste my time no more!",
    "This is getting nowhere. I'm going home!",
    "BAH! No more shall you insult me!",
    "Begone! I have had enough abuse for one day.",
];

pub const SPEECH_GET_OUT_OF_MY_STORE: &[&str] = &[
    "Out of my place!",
    "out... Out... OUT!!!",
    "Come back tomorrow.",
    "Leave my place. Begone!",
    "Come back when thou art richer.",
];

pub const SPEECH_HAGGLING_TRY_AGAIN: &[&str] = &[
    "You will have to do better than that!",
    "That's an insult!",
    "Do you wish to do business or not?",
    "Hah! Try again.",
    "Ridiculous!",
    "You've got to be kidding!",
    "You'd better be kidding!",
    "You try my patience.",
    "I don't hear you.",
    "Hmmm, nice weather we're having.",
];

pub const SPEECH_SORRY: &[&str] = &[
    "I must have heard you wrong.",
    "What was that?",
    "I'm sorry, say that again.",
    "What did you say?",
    "Sorry, what was that again?",
];

pub fn format_speech(template: &str, offer: u32, asking: u32) -> String {
    template.replace("%A1", &offer.to_string()).replace("%A2", &asking.to_string())
}

/// Calculate asking price range when the player is buying an item from a store.
pub fn calculate_buy_price(
    base_cost: u32,
    owner: &StoreOwner,
    player_race: Race,
    player_chr: i16,
) -> (u32, u32) {
    let p_race_idx = race_index(player_race);
    let race_adj = RACE_GOLD_ADJUSTMENTS[owner.race][p_race_idx];
    let price = (base_cost * race_adj / 100).max(1);

    let max_price = (price * owner.max_inflate / 100).max(1);
    let min_price = (price * owner.min_inflate / 100).max(1).min(max_price);

    let chr_adj = player_stat_adjustment_charisma(player_chr);
    let final_max = (max_price * chr_adj / 100).max(1);
    let final_min = (min_price * chr_adj / 100).max(1).min(final_max);

    (final_min, final_max)
}

/// Calculate offer price range when the player is selling an item to the store.
pub fn calculate_sell_price(
    base_cost: u32,
    owner: &StoreOwner,
    player_race: Race,
    player_chr: i16,
) -> (u32, u32) {
    let p_race_idx = race_index(player_race);
    let chr_adj = player_stat_adjustment_charisma(player_chr);
    let race_adj = RACE_GOLD_ADJUSTMENTS[owner.race][p_race_idx];

    let mut cost = (base_cost * (200 - chr_adj) / 100).max(1);
    cost = (cost * (200 - race_adj) / 100).max(1);

    let max_buy = (cost * (200 - owner.min_inflate.min(199)) / 100).max(1).min(owner.max_cost);
    let min_buy = (cost * (200 - owner.max_inflate.min(199)) / 100).max(1).min(max_buy);

    (min_buy, max_buy)
}

pub fn get_item_base_value(item: &Item) -> u32 {
    match &item.item_type {
        ItemType::Weapon { damage } => 25 + damage.num * damage.sides * 3,
        ItemType::Bow { multiplier } => 25 * multiplier,
        ItemType::Missile { damage } => (damage.num * damage.sides).max(2),
        ItemType::Armor { ac } => 20 + (*ac).max(0) as u32 * 15,
        ItemType::Potion { heal_amount } => 15 + (*heal_amount).max(0) as u32 * 2,
        ItemType::Scroll { teleport } => if *teleport { 35 } else { 25 },
        ItemType::Wand { charges, .. } => 30 + charges * 5,
        ItemType::Staff { charges, .. } => 40 + charges * 6,
        ItemType::Food { nutrition } => ((*nutrition).max(0) as u32 / 500).max(2),
        ItemType::Light { fuel } => 10 + (*fuel).max(0) as u32 / 500,
        ItemType::Ring { bonus } => 80 + bonus.unsigned_abs() * 50,
        ItemType::Amulet { bonus } => 80 + bonus.unsigned_abs() * 50,
        ItemType::MagicBook { spell_flags } => 30 + spell_flags.count_ones() * 15,
        ItemType::PrayerBook { spell_flags } => 30 + spell_flags.count_ones() * 15,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaggleState {
    pub item: Item,
    pub item_index: usize,
    pub initial_price: u32,
    pub min_price: u32,
    pub max_price: u32,
    pub current_asking: u32,
    pub last_bid: Option<u32>,
    pub insults: u32,
    pub offers_count: u32,
    pub comment: String,
    pub player_input: String,
    #[serde(default)]
    pub owner_index: usize,
}

#[allow(clippy::collapsible_if)]
pub fn handle_haggle_input(
    key_event: KeyEvent,
    screen_mode: &mut ScreenMode,
    active_haggle: &mut Option<HaggleState>,
    player: &mut Player,
    status_msg: &mut String,
    current_turn: u64,
    mut shop_closed_until: Option<&mut u64>,
) {
    let mut rng = rand::thread_rng();
    if *screen_mode == ScreenMode::BarterBuyMenu {
        if let Some(haggle) = active_haggle {
            let owner = &STORE_OWNERS[haggle.owner_index.min(MAX_OWNERS - 1)];
            match key_event.code {
                KeyCode::Esc => {
                    *screen_mode = ScreenMode::Shop;
                    *status_msg = "Transaction cancelled.".to_string();
                    *active_haggle = None;
                }
                KeyCode::Backspace => {
                    haggle.player_input.pop();
                }
                KeyCode::Enter => {
                    let input = haggle.player_input.trim();
                    // Blank input: accept shopkeeper's asking price
                    if input.is_empty() {
                        if haggle.current_asking > player.gold {
                            haggle.comment = "You do not have enough gold for that price!".to_string();
                            return;
                        }
                        player.gold -= haggle.current_asking;
                        let mut bought_item = haggle.item.clone();
                        bought_item.identified = true;
                        player.identify_item_kind(&bought_item.name);
                        player.add_item_to_inventory(bought_item);
                        let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                        *status_msg = format!("{} You buy the {} for {} gp.", speech, haggle.item.name, haggle.current_asking);
                        *screen_mode = ScreenMode::Shop;
                        *active_haggle = None;
                        return;
                    }

                    if let Ok(bid) = input.parse::<u32>() {
                        if bid == 0 {
                            haggle.comment = "You must offer at least 1 gold piece!".to_string();
                            haggle.player_input.clear();
                            return;
                        }
                        if bid > player.gold {
                            haggle.comment = "You do not have enough gold for that offer!".to_string();
                            haggle.player_input.clear();
                            return;
                        }

                        if bid > haggle.current_asking {
                            let sorry = SPEECH_SORRY[rng.gen_range(0..SPEECH_SORRY.len())];
                            haggle.comment = sorry.to_string();
                            haggle.player_input.clear();
                            return;
                        }

                        // Bid meets asking price: deal!
                        if bid == haggle.current_asking {
                            let final_price = haggle.current_asking;
                            player.gold -= final_price;
                            let mut bought_item = haggle.item.clone();
                            bought_item.identified = true;
                            player.identify_item_kind(&bought_item.name);
                            player.add_item_to_inventory(bought_item);
                            let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                            *status_msg = format!("{} You buy the {} for {} gp.", speech, haggle.item.name, final_price);
                            *screen_mode = ScreenMode::Shop;
                            *active_haggle = None;
                            return;
                        }

                        let prev_bid = haggle.last_bid;
                        haggle.last_bid = Some(bid);
                        haggle.player_input.clear();

                        // Bad faith: offering less than prior bid
                        if let Some(prev) = prev_bid {
                            if bid < prev {
                                haggle.insults += 1;
                                if haggle.insults >= owner.max_insults {
                                    let line1 = SPEECH_INSULTED_HAGGLED_DONE[rng.gen_range(0..SPEECH_INSULTED_HAGGLED_DONE.len())];
                                    let line2 = SPEECH_GET_OUT_OF_MY_STORE[rng.gen_range(0..SPEECH_GET_OUT_OF_MY_STORE.len())];
                                    *status_msg = format!("{} {}", line1, line2);
                                    if let Some(ref mut closed) = shop_closed_until {
                                        **closed = current_turn + rng.gen_range(2500..5000);
                                    }
                                    *screen_mode = ScreenMode::Dungeon;
                                    *active_haggle = None;
                                } else {
                                    let try_again = SPEECH_HAGGLING_TRY_AGAIN[rng.gen_range(0..SPEECH_HAGGLING_TRY_AGAIN.len())];
                                    haggle.comment = format!("{} (Offering less than before?!)", try_again);
                                }
                                return;
                            }
                        }

                        // Lowball insult check
                        let insult_threshold = (haggle.min_price / 3).max(1);
                        if bid < insult_threshold {
                            haggle.insults += 1;
                            if haggle.insults >= owner.max_insults {
                                let line1 = SPEECH_INSULTED_HAGGLED_DONE[rng.gen_range(0..SPEECH_INSULTED_HAGGLED_DONE.len())];
                                let line2 = SPEECH_GET_OUT_OF_MY_STORE[rng.gen_range(0..SPEECH_GET_OUT_OF_MY_STORE.len())];
                                *status_msg = format!("{} {}", line1, line2);
                                if let Some(ref mut closed) = shop_closed_until {
                                    **closed = current_turn + rng.gen_range(2500..5000);
                                }
                                *screen_mode = ScreenMode::Dungeon;
                                *active_haggle = None;
                            } else {
                                haggle.comment = SPEECH_HAGGLING_TRY_AGAIN[rng.gen_range(0..SPEECH_HAGGLING_TRY_AGAIN.len())].to_string();
                            }
                            return;
                        }

                        // Chance of acceptance if bid is at/above min_price
                        if bid >= haggle.min_price && haggle.offers_count >= 1 {
                            let spread = haggle.current_asking.saturating_sub(haggle.min_price);
                            let margin = bid.saturating_sub(haggle.min_price);
                            let accept_chance = if spread == 0 { 1.0 } else { 0.40 + 0.50 * (margin as f64 / spread as f64) };
                            if rng.gen_bool(accept_chance.clamp(0.20, 0.95)) {
                                player.gold -= bid;
                                let mut bought_item = haggle.item.clone();
                                bought_item.identified = true;
                                player.identify_item_kind(&bought_item.name);
                                player.add_item_to_inventory(bought_item);
                                let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                                *status_msg = format!("{} You buy the {} for {} gp.", speech, haggle.item.name, bid);
                                *screen_mode = ScreenMode::Shop;
                                *active_haggle = None;
                                return;
                            }
                        }

                        // Counter-offer: lower asking price toward player's bid
                        let gap = haggle.current_asking - bid;
                        let drop = ((gap * owner.haggles_per.max(1) / 10) + rng.gen_range(1..=3)).max(1);
                        let next_ask = (haggle.current_asking.saturating_sub(drop)).max(haggle.min_price);
                        haggle.current_asking = next_ask;
                        haggle.offers_count += 1;
                        if haggle.current_asking == haggle.min_price {
                            let template = SPEECH_SELLING_HAGGLE_FINAL[rng.gen_range(0..SPEECH_SELLING_HAGGLE_FINAL.len())];
                            haggle.comment = format_speech(template, bid, haggle.current_asking);
                        } else {
                            let template = SPEECH_SELLING_HAGGLE[rng.gen_range(0..SPEECH_SELLING_HAGGLE.len())];
                            haggle.comment = format_speech(template, bid, haggle.current_asking);
                        }
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() && haggle.player_input.len() < 7 => {
                    haggle.player_input.push(c);
                }
                _ => {}
            }
        }
    } else if *screen_mode == ScreenMode::BarterSellMenu {
        if let Some(haggle) = active_haggle {
            let owner = &STORE_OWNERS[haggle.owner_index.min(MAX_OWNERS - 1)];
            match key_event.code {
                KeyCode::Esc => {
                    *screen_mode = ScreenMode::Shop;
                    *status_msg = "Transaction cancelled.".to_string();
                    *active_haggle = None;
                }
                KeyCode::Backspace => {
                    haggle.player_input.pop();
                }
                KeyCode::Enter => {
                    let input = haggle.player_input.trim();
                    // Blank input: accept shopkeeper's offer
                    if input.is_empty() {
                        player.gold += haggle.current_asking;
                        player.identify_item_kind(&haggle.item.name);
                        let idx = haggle.item_index;
                        if idx < player.inventory.len() {
                            if player.inventory[idx].count > 1 {
                                player.inventory[idx].count -= 1;
                            } else {
                                player.inventory.remove(idx);
                            }
                        }
                        let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                        *status_msg = format!("{} You sell the {} for {} gp.", speech, haggle.item.name, haggle.current_asking);
                        *screen_mode = ScreenMode::Shop;
                        *active_haggle = None;
                        return;
                    }

                    if let Ok(ask) = input.parse::<u32>() {
                        if ask == 0 {
                            haggle.comment = "You cannot offer it for 0 gold!".to_string();
                            haggle.player_input.clear();
                            return;
                        }

                        if ask < haggle.current_asking {
                            let sorry = SPEECH_SORRY[rng.gen_range(0..SPEECH_SORRY.len())];
                            haggle.comment = sorry.to_string();
                            haggle.player_input.clear();
                            return;
                        }

                        // Player asks equal to current offer: deal!
                        if ask == haggle.current_asking {
                            let final_val = haggle.current_asking;
                            player.gold += final_val;
                            player.identify_item_kind(&haggle.item.name);
                            let idx = haggle.item_index;
                            if idx < player.inventory.len() {
                                if player.inventory[idx].count > 1 {
                                    player.inventory[idx].count -= 1;
                                } else {
                                    player.inventory.remove(idx);
                                }
                            }
                            let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                            *status_msg = format!("{} You sell the {} for {} gp.", speech, haggle.item.name, final_val);
                            *screen_mode = ScreenMode::Shop;
                            *active_haggle = None;
                            return;
                        }

                        let prev_ask = haggle.last_bid;
                        haggle.last_bid = Some(ask);
                        haggle.player_input.clear();

                        // Bad faith: asking more than prior ask
                        if let Some(prev) = prev_ask {
                            if ask > prev {
                                haggle.insults += 1;
                                if haggle.insults >= owner.max_insults {
                                    let line1 = SPEECH_INSULTED_HAGGLED_DONE[rng.gen_range(0..SPEECH_INSULTED_HAGGLED_DONE.len())];
                                    let line2 = SPEECH_GET_OUT_OF_MY_STORE[rng.gen_range(0..SPEECH_GET_OUT_OF_MY_STORE.len())];
                                    *status_msg = format!("{} {}", line1, line2);
                                    if let Some(ref mut closed) = shop_closed_until {
                                        **closed = current_turn + rng.gen_range(2500..5000);
                                    }
                                    *screen_mode = ScreenMode::Dungeon;
                                    *active_haggle = None;
                                } else {
                                    let try_again = SPEECH_HAGGLING_TRY_AGAIN[rng.gen_range(0..SPEECH_HAGGLING_TRY_AGAIN.len())];
                                    haggle.comment = format!("{} (Asking more than before?!)", try_again);
                                }
                                return;
                            }
                        }

                        // Insult check: asking too high (> 3x initial price)
                        let insult_threshold = haggle.initial_price * 3;
                        if ask > insult_threshold {
                            haggle.insults += 1;
                            if haggle.insults >= owner.max_insults {
                                let line1 = SPEECH_INSULTED_HAGGLED_DONE[rng.gen_range(0..SPEECH_INSULTED_HAGGLED_DONE.len())];
                                let line2 = SPEECH_GET_OUT_OF_MY_STORE[rng.gen_range(0..SPEECH_GET_OUT_OF_MY_STORE.len())];
                                *status_msg = format!("{} {}", line1, line2);
                                if let Some(ref mut closed) = shop_closed_until {
                                    **closed = current_turn + rng.gen_range(2500..5000);
                                }
                                *screen_mode = ScreenMode::Dungeon;
                                *active_haggle = None;
                            } else {
                                haggle.comment = SPEECH_HAGGLING_TRY_AGAIN[rng.gen_range(0..SPEECH_HAGGLING_TRY_AGAIN.len())].to_string();
                            }
                            return;
                        }

                        // Chance of acceptance if ask is within max_price
                        if ask <= haggle.max_price && haggle.offers_count >= 1 {
                            let spread = haggle.max_price.saturating_sub(haggle.current_asking);
                            let margin = haggle.max_price.saturating_sub(ask);
                            let accept_chance = if spread == 0 { 1.0 } else { 0.40 + 0.50 * (margin as f64 / spread as f64) };
                            if rng.gen_bool(accept_chance.clamp(0.20, 0.95)) {
                                player.gold += ask;
                                player.identify_item_kind(&haggle.item.name);
                                let idx = haggle.item_index;
                                if idx < player.inventory.len() {
                                    if player.inventory[idx].count > 1 {
                                        player.inventory[idx].count -= 1;
                                    } else {
                                        player.inventory.remove(idx);
                                    }
                                }
                                let speech = SPEECH_SALE_ACCEPTED[rng.gen_range(0..SPEECH_SALE_ACCEPTED.len())];
                                *status_msg = format!("{} You sell the {} for {} gp.", speech, haggle.item.name, ask);
                                *screen_mode = ScreenMode::Shop;
                                *active_haggle = None;
                                return;
                            }
                        }

                        // Counter-offer: raise offer toward player's ask
                        let gap = ask - haggle.current_asking;
                        let bump = ((gap * owner.haggles_per.max(1) / 10) + rng.gen_range(1..=3)).max(1);
                        let next_offer = (haggle.current_asking + bump).min(haggle.max_price).min(owner.max_cost);
                        haggle.current_asking = next_offer;
                        haggle.offers_count += 1;
                        if haggle.current_asking == haggle.max_price || haggle.current_asking == owner.max_cost {
                            let template = SPEECH_BUYING_HAGGLE_FINAL[rng.gen_range(0..SPEECH_BUYING_HAGGLE_FINAL.len())];
                            haggle.comment = format_speech(template, haggle.current_asking, ask);
                        } else {
                            let template = SPEECH_BUYING_HAGGLE[rng.gen_range(0..SPEECH_BUYING_HAGGLE.len())];
                            haggle.comment = format_speech(template, haggle.current_asking, ask);
                        }
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() && haggle.player_input.len() < 7 => {
                    haggle.player_input.push(c);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    use crate::player::{Class, Race};

    fn make_test_player() -> Player {
        let mut player = Player::new("Testy", Race::Human, Class::Warrior, 50, 10);
        player.inventory.clear();
        player.gold = 500;
        player.stats.strength = 14;
        player.stats.charisma = 14;
        player
    }

    fn make_test_item() -> Item {
        Item::new("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 })
    }

    fn key_enter() -> KeyEvent {
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn key_char(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_canonical_store_owners_count_and_stats() {
        assert_eq!(STORE_OWNERS.len(), 18);
        for owner in &STORE_OWNERS {
            assert!(owner.max_cost > 0);
            assert!(owner.max_inflate >= owner.min_inflate);
            assert!(owner.haggles_per >= 4);
            assert!(owner.max_insults >= 5);
            assert!(owner.race <= 7);
        }
        // First owner: Erick the Honest
        assert_eq!(STORE_OWNERS[0].name, "Erick the Honest       (Human)      General Store");
        assert_eq!(STORE_OWNERS[0].max_cost, 250);
        assert_eq!(STORE_OWNERS[0].max_inflate, 175);
        assert_eq!(STORE_OWNERS[0].min_inflate, 108);
        assert_eq!(STORE_OWNERS[0].max_insults, 12);
    }

    #[test]
    fn test_charisma_gold_adjustments() {
        assert_eq!(player_stat_adjustment_charisma(118), 90);
        assert_eq!(player_stat_adjustment_charisma(108), 92);
        assert_eq!(player_stat_adjustment_charisma(88), 94);
        assert_eq!(player_stat_adjustment_charisma(68), 96);
        assert_eq!(player_stat_adjustment_charisma(25), 98);
        assert_eq!(player_stat_adjustment_charisma(18), 100);
        assert_eq!(player_stat_adjustment_charisma(14), 104);
        assert_eq!(player_stat_adjustment_charisma(10), 112);
        assert_eq!(player_stat_adjustment_charisma(3), 130);
    }

    #[test]
    fn test_race_gold_adjustments_matrix() {
        // Human owner to human player: 100
        assert_eq!(RACE_GOLD_ADJUSTMENTS[0][0], 100);
        // Human owner to half-troll: 125
        assert_eq!(RACE_GOLD_ADJUSTMENTS[0][7], 125);
        // Dwarf owner to elf player: 120
        assert_eq!(RACE_GOLD_ADJUSTMENTS[5][2], 120);
        // Dwarf owner to dwarf player: 95
        assert_eq!(RACE_GOLD_ADJUSTMENTS[5][5], 95);
    }

    #[test]
    fn test_buy_and_sell_price_calculations() {
        let owner = &STORE_OWNERS[0]; // Erick (Human General Store, max_inflate 175, min_inflate 108)
        let (min_buy, max_buy) = calculate_buy_price(100, owner, Race::Human, 18);
        // Base 100 * 100/100 = 100. Inflates: min 108, max 175. Charisma 18 = 100/100.
        assert_eq!(min_buy, 108);
        assert_eq!(max_buy, 175);

        let (min_sell, max_sell) = calculate_sell_price(100, owner, Race::Human, 18);
        assert!(min_sell <= max_sell);
        assert!(max_sell <= owner.max_cost);
    }

    #[test]
    fn test_speech_formatting() {
        let formatted = format_speech("%A1 for such a fine item? HA! No less than %A2.", 50, 80);
        assert_eq!(formatted, "50 for such a fine item? HA! No less than 80.");
    }

    #[test]
    fn test_haggle_blank_enter_accepts_asking_price() {
        let mut player = make_test_player();
        let mut screen_mode = ScreenMode::BarterBuyMenu;
        let mut status_msg = String::new();
        let mut haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 30,
            min_price: 18,
            max_price: 30,
            current_asking: 27,
            last_bid: None,
            insults: 0,
            offers_count: 0,
            comment: String::new(),
            player_input: String::new(),
            owner_index: 0,
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);

        assert_eq!(screen_mode, ScreenMode::Shop);
        assert!(haggle.is_none());
        assert_eq!(player.gold, 500 - 27);
        assert_eq!(player.inventory.len(), 1);
        assert!(status_msg.contains("You buy the Potion of Cure Light Wounds for 27 gp"));
    }

    #[test]
    fn test_haggle_bid_updates_last_bid_and_lowers_asking() {
        let mut player = make_test_player();
        let mut screen_mode = ScreenMode::BarterBuyMenu;
        let mut status_msg = String::new();
        let mut haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 30,
            min_price: 18,
            max_price: 30,
            current_asking: 30,
            last_bid: None,
            insults: 0,
            offers_count: 0,
            comment: String::new(),
            player_input: String::new(),
            owner_index: 0,
        });

        // Type '2' and '0'
        handle_haggle_input(key_char('2'), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);
        handle_haggle_input(key_char('0'), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);
        assert_eq!(haggle.as_ref().unwrap().player_input, "20");

        // Submit offer 20
        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);

        // Offer is registered: last_bid is updated to Some(20)
        let state = haggle.as_ref().unwrap();
        assert_eq!(state.last_bid, Some(20));
        assert!(state.player_input.is_empty());
        // Asking price dropped from 30 down towards 20
        assert!(state.current_asking < 30);
        assert!(state.current_asking >= 18);
        assert!(!state.comment.is_empty());
    }

    #[test]
    fn test_haggle_selling_blank_enter_accepts_offer() {
        let mut player = make_test_player();
        player.add_item_to_inventory(make_test_item());
        let mut screen_mode = ScreenMode::BarterSellMenu;
        let mut status_msg = String::new();
        let mut haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 15,
            min_price: 10,
            max_price: 25,
            current_asking: 15,
            last_bid: None,
            insults: 0,
            offers_count: 0,
            comment: String::new(),
            player_input: String::new(),
            owner_index: 0,
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);

        assert_eq!(screen_mode, ScreenMode::Shop);
        assert!(haggle.is_none());
        assert_eq!(player.gold, 500 + 15);
        assert!(player.inventory.is_empty());
    }

    #[test]
    fn test_haggle_insults_closes_store() {
        let mut player = make_test_player();
        let mut screen_mode = ScreenMode::BarterBuyMenu;
        let mut status_msg = String::new();
        let mut closed_turn = 0u64;

        // Owner 1: Mauglin the Grumpy has max_insults = 5
        let mut haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 30,
            min_price: 18,
            max_price: 30,
            current_asking: 30,
            last_bid: Some(25),
            insults: 4, // 1 insult away from being kicked out
            offers_count: 1,
            comment: String::new(),
            player_input: "10".to_string(), // Bidding 10 < 25 (bad faith)
            owner_index: 1,
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 500, Some(&mut closed_turn));

        assert_eq!(screen_mode, ScreenMode::Dungeon);
        assert!(haggle.is_none());
        assert!(closed_turn >= 500 + 2500);
        assert!(!status_msg.is_empty());
    }

    #[test]
    fn test_haggle_overbid_and_underask_speech_sorry() {
        let mut player = make_test_player();
        let mut screen_mode = ScreenMode::BarterBuyMenu;
        let mut status_msg = String::new();

        // 1. Buying: offering 35 when current asking is 30
        let mut haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 30,
            min_price: 18,
            max_price: 30,
            current_asking: 30,
            last_bid: None,
            insults: 0,
            offers_count: 0,
            comment: String::new(),
            player_input: "35".to_string(),
            owner_index: 0,
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);
        let state = haggle.as_ref().unwrap();
        assert!(SPEECH_SORRY.contains(&state.comment.as_str()));
        assert!(state.player_input.is_empty());

        // 2. Selling: asking 10 when current offer is 15
        screen_mode = ScreenMode::BarterSellMenu;
        haggle = Some(HaggleState {
            item: make_test_item(),
            item_index: 0,
            initial_price: 15,
            min_price: 10,
            max_price: 25,
            current_asking: 15,
            last_bid: None,
            insults: 0,
            offers_count: 0,
            comment: String::new(),
            player_input: "10".to_string(),
            owner_index: 0,
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg, 100, None);
        let state = haggle.as_ref().unwrap();
        assert!(SPEECH_SORRY.contains(&state.comment.as_str()));
        assert!(state.player_input.is_empty());
    }

    #[test]
    fn test_shop_info_maintain() {
        let mut rng = rand::thread_rng();
        let mut shop = crate::dungeon::ShopInfo::new(10, 10, crate::dungeon::ShopType::General, &mut rng);
        shop.insults = 3;
        shop.closed_until_turn = 1000;

        // Turn 500: closed_until_turn not reached, insults decrement by 1
        shop.maintain(500, &mut rng);
        assert_eq!(shop.insults, 2);
        assert_eq!(shop.closed_until_turn, 1000);
        assert!(shop.is_closed(500));

        // Turn 1000: closed_until_turn reached -> resets closed_until_turn to 0
        shop.maintain(1000, &mut rng);
        assert_eq!(shop.insults, 1);
        assert_eq!(shop.closed_until_turn, 0);
        assert!(!shop.is_closed(1000));
    }
}
