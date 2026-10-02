use serde::{Serialize, Deserialize};
use crossterm::event::{KeyEvent, KeyCode};
use rand::Rng;

use crate::player::{Player, Item};
use crate::ScreenMode;

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
}

#[allow(clippy::collapsible_if)]
pub fn handle_haggle_input(
    key_event: KeyEvent,
    screen_mode: &mut ScreenMode,
    active_haggle: &mut Option<HaggleState>,
    player: &mut Player,
    status_msg: &mut String,
) {
    let mut rng = rand::thread_rng();
    if *screen_mode == ScreenMode::BarterBuyMenu {
        if let Some(haggle) = active_haggle {
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
                        *status_msg = format!("You buy the {} for {} gp.", haggle.item.name, haggle.current_asking);
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

                        // Bid meets or exceeds asking price: deal!
                        if bid >= haggle.current_asking {
                            let final_price = bid.min(haggle.current_asking);
                            player.gold -= final_price;
                            let mut bought_item = haggle.item.clone();
                            bought_item.identified = true;
                            player.identify_item_kind(&bought_item.name);
                            player.add_item_to_inventory(bought_item);
                            *status_msg = format!("You buy the {} for {} gp.", haggle.item.name, final_price);
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
                                if haggle.insults >= 4 {
                                    *status_msg = "The shopkeeper gets angry and kicks you out!".to_string();
                                    *screen_mode = ScreenMode::Dungeon;
                                    *active_haggle = None;
                                } else {
                                    haggle.comment = "You're offering less than before?! That is insulting!".to_string();
                                }
                                return;
                            }
                        }

                        // Lowball insult check
                        let insult_threshold = (haggle.initial_price / 4).max(1);
                        if bid < insult_threshold {
                            haggle.insults += 1;
                            if haggle.insults >= 4 {
                                *status_msg = "The shopkeeper gets angry and kicks you out!".to_string();
                                *screen_mode = ScreenMode::Dungeon;
                                *active_haggle = None;
                            } else {
                                haggle.comment = match haggle.insults {
                                    1 => "That offer is insulting!".to_string(),
                                    2 => "Are you trying to rob me?".to_string(),
                                    _ => "Get out of my shop with such low offers!".to_string(),
                                };
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
                                *status_msg = format!("Accepted! You buy the {} for {} gp.", haggle.item.name, bid);
                                *screen_mode = ScreenMode::Shop;
                                *active_haggle = None;
                                return;
                            }
                        }

                        // Counter-offer: lower asking price toward player's bid
                        let gap = haggle.current_asking - bid;
                        let drop = ((gap + 1) / 3).max(1);
                        let next_ask = (haggle.current_asking - drop).max(haggle.min_price);
                        haggle.current_asking = next_ask;
                        haggle.offers_count += 1;
                        if haggle.current_asking == haggle.min_price {
                            haggle.comment = format!("{} gp is my absolute lowest price!", haggle.current_asking);
                        } else {
                            haggle.comment = format!("I can go down to {} gp?", haggle.current_asking);
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
                        *status_msg = format!("You sell the {} for {} gp.", haggle.item.name, haggle.current_asking);
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

                        // Player asks less than or equal to current offer: deal!
                        if ask <= haggle.current_asking {
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
                            *status_msg = format!("You sell the {} for {} gp.", haggle.item.name, haggle.current_asking);
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
                                if haggle.insults >= 4 {
                                    *status_msg = "The shopkeeper gets angry and kicks you out!".to_string();
                                    *screen_mode = ScreenMode::Dungeon;
                                    *active_haggle = None;
                                } else {
                                    haggle.comment = "You're asking for MORE than before?! Stop playing games!".to_string();
                                }
                                return;
                            }
                        }

                        // Insult check: asking too high (> 3x initial price)
                        let insult_threshold = haggle.initial_price * 3;
                        if ask > insult_threshold {
                            haggle.insults += 1;
                            if haggle.insults >= 4 {
                                *status_msg = "The shopkeeper gets angry and kicks you out!".to_string();
                                *screen_mode = ScreenMode::Dungeon;
                                *active_haggle = None;
                            } else {
                                haggle.comment = match haggle.insults {
                                    1 => "You are asking way too much!".to_string(),
                                    2 => "No way I am paying that much.".to_string(),
                                    _ => "Stop trying to swindle me!".to_string(),
                                };
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
                                *status_msg = format!("Accepted! You sell the {} for {} gp.", haggle.item.name, ask);
                                *screen_mode = ScreenMode::Shop;
                                *active_haggle = None;
                                return;
                            }
                        }

                        // Counter-offer: raise offer toward player's ask
                        let gap = ask - haggle.current_asking;
                        let rise = ((gap + 1) / 3).max(1);
                        let next_offer = (haggle.current_asking + rise).min(haggle.max_price);
                        haggle.current_asking = next_offer;
                        haggle.offers_count += 1;
                        if haggle.current_asking == haggle.max_price {
                            haggle.comment = format!("{} gp is my absolute final offer!", haggle.current_asking);
                        } else {
                            haggle.comment = format!("I can go up to {} gp?", haggle.current_asking);
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
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use crate::player::{Race, Class, ItemType};

    fn make_test_player() -> Player {
        let mut p = Player::new("Tester", Race::Human, Class::Warrior, 0, 0);
        p.inventory.clear();
        p.gold = 500;
        p
    }

    fn make_test_item() -> Item {
        Item::new("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 })
    }

    fn key_char(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn key_enter() -> KeyEvent {
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
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
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg);

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
        });

        // Type '2' and '0'
        handle_haggle_input(key_char('2'), &mut screen_mode, &mut haggle, &mut player, &mut status_msg);
        handle_haggle_input(key_char('0'), &mut screen_mode, &mut haggle, &mut player, &mut status_msg);
        assert_eq!(haggle.as_ref().unwrap().player_input, "20");

        // Submit offer 20
        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg);

        // Offer is registered: last_bid is updated to Some(20)
        let state = haggle.as_ref().unwrap();
        assert_eq!(state.last_bid, Some(20));
        assert!(state.player_input.is_empty());
        // Asking price dropped from 30 down towards 20
        assert!(state.current_asking < 30);
        assert!(state.current_asking >= 18);
        assert!(state.comment.contains("I can go down to") || state.comment.contains("lowest price"));
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
        });

        handle_haggle_input(key_enter(), &mut screen_mode, &mut haggle, &mut player, &mut status_msg);

        assert_eq!(screen_mode, ScreenMode::Shop);
        assert!(haggle.is_none());
        assert_eq!(player.gold, 500 + 15);
        assert!(player.inventory.is_empty());
    }
}
