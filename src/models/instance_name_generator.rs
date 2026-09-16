use crate::models::InstanceName;
use std::str::FromStr;

/// How often a fresh pair is drawn before the last one counts up
const MAX_ROLLS: usize = 3;

/// An endless source of readable instance names like 'grumpy-dragon'. The
/// first names are fresh pairs and after that the last pair counts up.
#[derive(Default)]
pub struct InstanceNameGenerator {
    rolls: usize,
    name: String,
}

impl InstanceNameGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    const ADJECTIVES: &'static [&'static str] = &[
        "nameless", "cursed", "feral", "buggy", "salty", "early", "offduty", "parttime", "retired",
        "lawful", "grumpy", "sleepy", "wobbly", "squeaky", "harmless", "clueless", "chaotic",
        "grave", "crabby", "musty", "stuffy", "stiff", "sneaky", "smug", "cuddly", "squishy",
        "dapper", "glitchy", "shifty", "fishy", "ruthless", "restless", "timeless", "ageless",
        "flawless", "fearless", "formal", "posh", "fancy", "vintage", "janky", "chatty", "hungry",
        "snappy", "dry", "fried", "burnt", "haunted", "hollow", "brittle", "jaded", "washed",
        "lost", "frozen", "broke", "wired", "cross", "cranky", "cheeky", "tireless", "sharp",
        "shady", "flaky", "feisty", "silly", "moody", "jolly", "sulky", "giddy", "soulless",
        "lifeless", "reckless", "fuzzy", "fluffy", "bouncy", "wiggly", "jiggly", "gooey", "sticky",
        "crispy", "curly", "droopy", "puffy", "spiky", "shaggy", "stubby", "dusty", "rusty",
        "muddy", "soggy", "frosty", "sparkly", "bumpy", "lanky", "chunky", "lumpy", "slimy",
        "prickly", "scruffy", "wrinkly", "jumpy", "perky", "rowdy", "spunky", "tiny",
    ];

    const CREATURES: &'static [&'static str] = &[
        "dragon",
        "wyvern",
        "basilisk",
        "hydra",
        "serpent",
        "unicorn",
        "hippogriff",
        "griffin",
        "phoenix",
        "firebird",
        "sphinx",
        "chimera",
        "minotaur",
        "centaur",
        "cyclops",
        "gorgon",
        "siren",
        "kraken",
        "mermaid",
        "roc",
        "behemoth",
        "leviathan",
        "sandworm",
        "medusa",
        "cerberus",
        "pegasus",
        "imp",
        "demon",
        "genie",
        "valkyrie",
        "reaper",
        "hellhound",
        "darkelf",
        "woodelf",
        "seaserpent",
        "frostgiant",
        "firegiant",
        "stonegolem",
        "irongolem",
        "snowbeast",
        "direwolf",
        "elf",
        "dwarf",
        "gnome",
        "halfling",
        "pixie",
        "goblin",
        "hobgoblin",
        "orc",
        "gremlin",
        "changeling",
        "owlbear",
        "bugbear",
        "kobold",
        "elemental",
        "mimic",
        "troll",
        "ogre",
        "giant",
        "colossus",
        "beast",
        "monster",
        "creature",
        "titan",
        "brute",
        "critter",
        "fiend",
        "witch",
        "wizard",
        "warlock",
        "berserker",
        "yeti",
        "bigfoot",
        "dropbear",
        "jackalope",
        "squonk",
        "hidebehind",
        "snipe",
        "teakettler",
        "rubberduck",
        "ghost",
        "ghoul",
        "wraith",
        "phantom",
        "spirit",
        "banshee",
        "zombie",
        "vampire",
        "werewolf",
        "wolfman",
        "mummy",
        "skeleton",
        "boogeyman",
        "sandman",
        "scarecrow",
        "golem",
        "mandrake",
        "wisp",
        "clone",
        "mutant",
        "blob",
        "slime",
        "toothfairy",
        "dustbunny",
    ];

    fn draw_name() -> String {
        Self::build_name(
            Self::get_random_number(Self::ADJECTIVES.len()),
            Self::get_random_number(Self::CREATURES.len()),
        )
    }

    fn get_random_number(limit: usize) -> usize {
        let mut bytes = [0_u8; 8];
        getrandom::fill(&mut bytes).ok();
        (u64::from_ne_bytes(bytes) % limit.max(1) as u64) as usize
    }

    fn build_name(adjective_index: usize, creature_index: usize) -> String {
        format!(
            "{}-{}",
            Self::ADJECTIVES[adjective_index % Self::ADJECTIVES.len()],
            Self::CREATURES[creature_index % Self::CREATURES.len()]
        )
    }
}

impl Iterator for InstanceNameGenerator {
    type Item = InstanceName;

    fn next(&mut self) -> Option<Self::Item> {
        self.rolls += 1;

        let name = if self.rolls <= MAX_ROLLS {
            self.name = Self::draw_name();
            self.name.clone()
        } else {
            format!("{}{}", self.name, self.rolls - MAX_ROLLS + 1)
        };

        // Every word in both lists is a valid instance name, which a test guards.
        Some(InstanceName::from_str(&name).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_every_word_is_a_valid_and_unique_instance_name() {
        for list in [
            InstanceNameGenerator::ADJECTIVES,
            InstanceNameGenerator::CREATURES,
        ] {
            assert!(!list.is_empty());
            assert_eq!(list.len(), list.iter().collect::<HashSet<_>>().len());

            for word in list {
                InstanceName::from_str(word).unwrap();
            }
        }
    }

    #[test]
    fn test_draw_a_pair_from_both_lists() {
        for name in InstanceNameGenerator::new().take(MAX_ROLLS) {
            let (adjective, creature) = name.as_str().split_once('-').unwrap();

            assert!(InstanceNameGenerator::ADJECTIVES.contains(&adjective));
            assert!(InstanceNameGenerator::CREATURES.contains(&creature));
        }
    }

    #[test]
    fn test_count_up_after_the_last_roll() {
        let names = InstanceNameGenerator::new()
            .take(MAX_ROLLS + 2)
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let last_roll = &names[MAX_ROLLS - 1];

        assert_eq!(names[MAX_ROLLS], format!("{last_roll}2"));
        assert_eq!(names[MAX_ROLLS + 1], format!("{last_roll}3"));
    }
}
