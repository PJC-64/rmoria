use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::player::{Item, ItemType};

pub const COLORS: &[&str] = &[
    "Icky Green", "Light Brown", "Clear",
    "Azure", "Blue", "Blue Speckled", "Black", "Brown", "Brown Speckled", "Bubbling",
    "Chartreuse", "Cloudy", "Copper Speckled", "Crimson", "Cyan", "Dark Blue",
    "Dark Green", "Dark Red", "Gold Speckled", "Green", "Green Speckled", "Grey",
    "Grey Speckled", "Hazy", "Indigo", "Light Blue", "Light Green", "Magenta",
    "Metallic Blue", "Metallic Red", "Metallic Green", "Metallic Purple", "Misty",
    "Orange", "Orange Speckled", "Pink", "Pink Speckled", "Puce", "Purple",
    "Purple Speckled", "Red", "Red Speckled", "Silver Speckled", "Smoky",
    "Tangerine", "Violet", "Vermilion", "White", "Yellow",
];

pub const MUSHROOMS: &[&str] = &[
    "Blue", "Black", "Black Spotted", "Brown", "Dark Blue", "Dark Green", "Dark Red",
    "Ecru", "Furry", "Green", "Grey", "Light Blue", "Light Green", "Plaid", "Red",
    "Slimy", "Tan", "White", "White Spotted", "Wooden", "Wrinkled", "Yellow",
];

pub const WOODS: &[&str] = &[
    "Aspen", "Balsa", "Banyan", "Birch", "Cedar", "Cottonwood", "Cypress", "Dogwood",
    "Elm", "Eucalyptus", "Hemlock", "Hickory", "Ironwood", "Locust", "Mahogany",
    "Maple", "Mulberry", "Oak", "Pine", "Redwood", "Rosewood", "Spruce", "Sycamore",
    "Teak", "Walnut",
];

pub const METALS: &[&str] = &[
    "Aluminum", "Cast Iron", "Chromium", "Copper", "Gold", "Iron", "Magnesium",
    "Molybdenum", "Nickel", "Rusty", "Silver", "Steel", "Tin", "Titanium", "Tungsten",
    "Zirconium", "Zinc", "Aluminum-Plated", "Copper-Plated", "Gold-Plated",
    "Nickel-Plated", "Silver-Plated", "Steel-Plated", "Tin-Plated", "Zinc-Plated",
];

pub const ROCKS: &[&str] = &[
    "Alexandrite", "Amethyst", "Aquamarine", "Azurite", "Beryl", "Bloodstone",
    "Calcite", "Carnelian", "Corundum", "Diamond", "Emerald", "Fluorite", "Garnet",
    "Granite", "Jade", "Jasper", "Lapis Lazuli", "Malachite", "Marble", "Moonstone",
    "Onyx", "Opal", "Pearl", "Quartz", "Quartzite", "Rhodonite", "Ruby", "Sapphire",
    "Tiger Eye", "Topaz", "Turquoise", "Zircon",
];

pub const AMULETS: &[&str] = &[
    "Amber", "Driftwood", "Coral", "Agate", "Ivory", "Obsidian",
    "Bone", "Brass", "Bronze", "Pewter", "Tortoise Shell",
];

pub const SYLLABLES: &[&str] = &[
    "a", "ab", "ag", "aks", "ala", "an", "ankh", "app", "arg",
    "arze", "ash", "aus", "ban", "bar", "bat", "bek", "bie", "bin",
    "bit", "bjor", "blu", "bot", "bu", "byt", "comp", "con", "cos",
    "cre", "dalf", "dan", "den", "doe", "dok", "eep", "el", "eng",
    "er", "ere", "erk", "esh", "evs", "fa", "fid", "for", "fri",
    "fu", "gan", "gar", "glen", "gop", "gre", "ha", "he", "hyd",
    "i", "ing", "ion", "ip", "ish", "it", "ite", "iv", "jo",
    "kho", "kli", "klis", "la", "lech", "man", "mar", "me", "mi",
    "mic", "mik", "mon", "mung", "mur", "nej", "nelg", "nep", "ner",
    "nes", "nis", "nih", "nin", "o", "od", "ood", "org", "orn",
    "ox", "oxy", "pay", "pet", "ple", "plu", "po", "pot", "prok",
    "re", "rea", "rhov", "ri", "ro", "rog", "rok", "rol", "sa",
    "san", "sat", "see", "sef", "seh", "shu", "ski", "sna", "sne",
    "snik", "sno", "so", "sol", "sri", "sta", "sun", "ta", "tab",
    "tem", "ther", "ti", "tox", "trol", "tue", "turs", "u", "ulk",
    "um", "un", "uni", "ur", "val", "viv", "vly", "vom", "wah",
    "wed", "werg", "wex", "whon", "wun", "x", "yerg", "yp", "zun",
];

pub fn generate_scroll_title<R: Rng>(rng: &mut R) -> String {
    let word_count = rng.gen_range(1..=3);
    let mut words = Vec::new();
    for _ in 0..word_count {
        let syl_count = rng.gen_range(1..=2);
        let mut word = String::new();
        for _ in 0..syl_count {
            let s = SYLLABLES[rng.gen_range(0..SYLLABLES.len())];
            word.push_str(s);
        }
        words.push(word);
    }
    words.join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlavorRegistry {
    pub potion_flavors: HashMap<String, String>,
    pub scroll_flavors: HashMap<String, String>,
    pub available_colors: Vec<String>,
    pub identified_items: HashSet<String>,
}

impl Default for FlavorRegistry {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self::new(&mut rng)
    }
}

impl FlavorRegistry {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut colors: Vec<String> = COLORS.iter().map(|s| s.to_string()).collect();
        // Umoria preserves first 3 colors fixed, shuffles the rest
        if colors.len() > 3 {
            colors[3..].shuffle(rng);
        }

        Self {
            potion_flavors: HashMap::new(),
            scroll_flavors: HashMap::new(),
            available_colors: colors,
            identified_items: HashSet::new(),
        }
    }

    pub fn is_identified(&self, item_name: &str) -> bool {
        self.identified_items.contains(item_name)
    }

    pub fn identify(&mut self, item_name: &str) -> bool {
        self.identified_items.insert(item_name.to_string())
    }

    pub fn get_or_create_potion_flavor<R: Rng>(&mut self, item_name: &str, rng: &mut R) -> String {
        if let Some(flavor) = self.potion_flavors.get(item_name) {
            return flavor.clone();
        }

        let flavor = if !self.available_colors.is_empty() {
            self.available_colors.remove(0)
        } else {
            let idx = rng.gen_range(0..COLORS.len());
            COLORS[idx].to_string()
        };

        self.potion_flavors.insert(item_name.to_string(), flavor.clone());
        flavor
    }

    pub fn get_or_create_scroll_flavor<R: Rng>(&mut self, item_name: &str, rng: &mut R) -> String {
        if let Some(title) = self.scroll_flavors.get(item_name) {
            return title.clone();
        }

        let title = loop {
            let cand = generate_scroll_title(rng);
            if !self.scroll_flavors.values().any(|v| v == &cand) {
                break cand;
            }
        };

        self.scroll_flavors.insert(item_name.to_string(), title.clone());
        title
    }

    pub fn assign_flavor<R: Rng>(&mut self, item: &mut Item, rng: &mut R) {
        // Books and mundane equipment don't have flavors
        if item.name.contains("Spellbook") || item.name.contains("Prayerbook") || item.name.contains("Torch") || item.name.contains("Flask") {
            item.identified = true;
            return;
        }

        if self.is_identified(&item.name) {
            item.identified = true;
        }

        if item.flavor.is_none() {
            match &item.item_type {
                ItemType::Potion { .. } => {
                    let flv = self.get_or_create_potion_flavor(&item.name, rng);
                    item.flavor = Some(flv);
                }
                ItemType::Scroll { .. } => {
                    let flv = self.get_or_create_scroll_flavor(&item.name, rng);
                    item.flavor = Some(flv);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Item, ItemType};

    #[test]
    fn test_flavor_assignment_and_display() {
        let mut rng = rand::thread_rng();
        let mut reg = FlavorRegistry::new(&mut rng);

        let mut pot = Item::new("Potion of Cure Light Wounds", 1, 5, ItemType::Potion { heal_amount: 10 });
        pot.identified = false;
        reg.assign_flavor(&mut pot, &mut rng);

        assert!(!pot.identified);
        assert!(pot.flavor.is_some());
        assert!(pot.display_name().ends_with("Potion"));
        assert!(!pot.display_name().contains("Cure Light Wounds"));

        // Now identify the kind
        reg.identify(&pot.name);
        pot.identified = true;
        assert_eq!(pot.display_name(), "Potion of Cure Light Wounds");
    }

    #[test]
    fn test_scroll_title_generation() {
        let mut rng = rand::thread_rng();
        let mut reg = FlavorRegistry::new(&mut rng);

        let mut sc = Item::new("Scroll of Phase Door", 1, 2, ItemType::Scroll { teleport: true });
        sc.identified = false;
        reg.assign_flavor(&mut sc, &mut rng);

        assert!(!sc.identified);
        assert!(sc.flavor.is_some());
        assert!(sc.display_name().starts_with("Scroll titled \""));
        assert!(!sc.display_name().contains("Phase Door"));

        reg.identify(&sc.name);
        sc.identified = true;
        assert_eq!(sc.display_name(), "Scroll of Phase Door");
    }
}
