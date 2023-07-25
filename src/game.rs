use chrono::{TimeZone, Utc};
use el_slugify::slugify;
use fastly::KVStore;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashSet;

const KV_STORE_NAME: &str = "yourdle";

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GameData {
    pub game: String,
    pub slug: String,
    pub description: String,
    words: Vec<String>,
    #[serde(default = "timestamp_for_today")]
    starts: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GameDataForm {
    pub game: String,
    pub description: String,
    pub words: String,
}

impl GameData {
    // Any validation of form data submitted for the creation of a new game.
    pub fn from_form(form: GameDataForm) -> Self {
        let slug: String = slugify(&form.game);
        let mut words = sanitize_as_words(form.words);
        randomize_vec(&mut words);
        GameData {
            game: form.game,
            slug,
            description: form.description,
            words,
            starts: timestamp_for_today(),
        }
    }

    // Save the game data to KV store – randomize words, start the game today and return the first word.
    pub fn save(&mut self) -> Result<String, &str> {
        match KVStore::open(KV_STORE_NAME) {
            Ok(Some(mut game_store)) => {
                self.starts = timestamp_for_today();
                randomize_vec(&mut self.words);
                println!("Saving game data: {:?}", self);
                return match serde_json::to_string(&self) {
                    Ok(game_data_string) => match game_store.insert(&self.slug, game_data_string) {
                        Ok(_) => Ok(self.words[0].to_owned()),
                        _ => Err("Could not save game data"),
                    },
                    _ => Err("Could not serialize game data"),
                };
            }
            _ => Err("Could not open KV store"),
        }
    }

    // Load the game data from KV store.
    pub fn load(slug: &str) -> Result<GameData, &str> {
        match KVStore::open(KV_STORE_NAME) {
            Ok(Some(game_store)) => match game_store.lookup_str(slug) {
                Ok(Some(value)) => {
                    let game_data: GameData = serde_json::from_str(&value).unwrap();
                    println!("Loaded game data: {:?}", game_data);
                    Ok(game_data)
                }
                _ => Err("Could not load game data"),
            },
            _ => Err("Could not open KV store"),
        }
    }

    // Get today's word, the number of the current game, and the total words in the game.
    pub fn get_word(&mut self) -> Result<(String, i64, usize), &str> {
        // The index of the current word is the number of days since the game started.
        let word_idx = get_days_since(self.starts);
        let total_words = self.words.len();
        // If all words have been used, reset the game.
        if word_idx >= total_words as i64 {
            return Ok((self.save()?, 0, total_words));
        }
        Ok((
            self.words[word_idx as usize].to_owned(),
            word_idx,
            total_words,
        ))
    }

    // Validate if a word is in the game's list of words.
    pub fn validate_word(&self, word: &str) -> bool {
        self.words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .contains(&word.to_lowercase().to_owned())
    }

    // Check the game doesn't already exist.
    pub fn check_not_exists(game: &str) -> Result<String, &str> {
        let slug = slugify(&game);
        match KVStore::open(KV_STORE_NAME) {
            Ok(Some(game_store)) => match game_store.lookup(&slug) {
                Ok(None) => Ok(slug),
                Ok(Some(_)) => Err("Game already exists"),
                _ => Err("Could not load game data"),
            },
            _ => Err("Could not open KV store"),
        }
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            include_str!("browser/start.html")
                .to_string()
                .replace("{GAME}", &self.game)
                .replace("{DESCRIPTION}", &self.description)
                .replace("{SLUG}", &self.slug)
        )
    }
}

pub fn get_days_since(timestamp: i64) -> i64 {
    // Convert the given timestamp to a DateTime.
    let given_datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    // Calculate the difference in days.
    Utc::now().signed_duration_since(given_datetime).num_days()
}

pub fn timestamp_for_today() -> i64 {
    Utc::now().timestamp()
}

fn randomize_vec(strings: &mut Vec<String>) {
    let mut rng = rand::thread_rng();
    strings.shuffle(&mut rng);
}

fn sanitize_as_words(text: String) -> Vec<String> {
    // Ensure we have a maximum of 365 unique words.
    let unique_words: HashSet<String> = text
        // Replace whitespace and punctuation with a single space.
        .replace(|c: char| c.is_whitespace() || c.is_ascii_punctuation(), " ")
        // Remove non-English alphabet characters.
        .replace(|c: char| !c.is_whitespace() && !c.is_ascii_alphabetic(), "")
        .to_uppercase()
        .trim()
        // Split into words.
        .split_whitespace()
        // Ensure each word is at least 3 and at most 10 letters long.
        .filter(|&s| s.len() >= 3 && s.len() <= 10)
        .map(String::from)
        .take(365)
        .collect();

    unique_words.into_iter().collect()
}
