use crate::utils::{
    get_days_since, randomize_vec, sanitize_as_words, timestamp_for_today, truncate_to_max_length,
};
use el_slugify::slugify;
use fastly::KVStore;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

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
    pub description: Option<String>,
    pub words: Option<String>,
}

impl GameData {
    // Any validation of form data submitted for the creation of a new game.
    pub fn from_form(form: GameDataForm) -> Result<Self, &'static str> {
        if form.game.len() < 3 {
            return Err("Name too short");
        }
        let description = form.description.unwrap_or_default();
        if description.len() < 10 {
            return Err("Description too short");
        }
        let words = sanitize_as_words(form.words.unwrap_or_default());
        if words.len() < 7 {
            return Err("Must have at least 7 unique words");
        }
        let game = truncate_to_max_length(&form.game, 12);
        if GameData::check_not_exists(&game).is_err() {
            return Err("Game already exists");
        }
        Ok(GameData {
            game: game.to_string(),
            slug: slugify(&game),
            description: truncate_to_max_length(&description, 140).to_string(),
            words,
            starts: timestamp_for_today(),
        })
    }

    // Save the game data to KV store – randomize words, start the game today and return the first word.
    pub fn save(&mut self) -> Result<String, &str> {
        match KVStore::open(KV_STORE_NAME) {
            Ok(Some(mut game_store)) => {
                self.starts = timestamp_for_today();
                randomize_vec(&mut self.words);
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
                Ok(Some(value)) => Ok(serde_json::from_str::<GameData>(&value).unwrap()),
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
            return Ok((self.save().ok().unwrap(), 0, total_words));
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
        // Handle reserved routes first.
        if vec![
            "new".to_string(),
            "validate".to_string(),
            "feedback".to_string(),
        ].contains(&slug) {
            return Err("Reserved route");
        }
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
        let word_idx = get_days_since(self.starts);
        write!(
            f,
            "{}",
            include_str!("browser/start.html")
                .to_string()
                .replace("{GAME}", &self.game)
                .replace("{DESCRIPTION}", &self.description)
                .replace("{SLUG}", &self.slug)
                .replace("{CURRENT}", &word_idx.to_string())
                .replace("{TOTAL}", &self.words.len().to_string())
        )
    }
}
