use crate::utils;
use fastly::KVStore;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

const KV_STORE_NAME: &str = "yourdle-stats";
pub const TRIES: usize = 6;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Match {
    Correct,
    Near,
    Wrong,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Guess(pub String, pub Vec<Match>);

impl Guess {
    pub fn new(guess: &str, word: &str) -> Guess {
        let mut diff = Guess(guess.to_owned(), vec![Match::Wrong; word.len()]);
        let guess = guess.as_bytes();
        let mut word = word.to_uppercase().as_bytes().to_owned();

        // Check correct guesses first.
        for (i, &b) in guess.iter().enumerate() {
            if word[i] == b {
                word[i] = 0; // Match once
                diff.1[i] = Match::Correct;
            }
        }

        // Check near guesses.
        for (i, &b) in guess.iter().enumerate() {
            if diff.1[i] != Match::Wrong {
                continue;
            }
            if let Some(j) = word.iter().position(|&x| x == b) {
                word[j] = 0; // Match once
                diff.1[i] = Match::Near;
            }
        }
        diff
    }

    // Check if the entire guess is correct.
    pub fn is_win(&self) -> bool {
        self.1.iter().all(|&x| x == Match::Correct)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Guesses {
    pub today: String,
    pub today_length: usize,
    pub outcome: Vec<Guess>,
    pub distribution: Vec<u16>,
    pub last_win: String,
    pub last_loss: String,
    pub max_streak: u16,
    pub streak: u16,
    pub games: u16,
}

impl Guesses {
    // Initialize a new Guesses struct from state.
    pub fn load(game: &str, user_id: &str, today_word_length: usize) -> Guesses {
        // Retrieve saved stats from KV Store.
        let key = format!("{}-{}", game, user_id);
        let saved_stats = match KVStore::open(KV_STORE_NAME) {
            Ok(Some(stats_store)) => match stats_store.lookup_str(&key) {
                Ok(Some(stats)) => stats,
                _ => "".to_owned(),
            },
            _ => "".to_owned(),
        };
        // Initialize Guesses struct.
        let mut guesses: Guesses = serde_json::from_str(&saved_stats).unwrap_or_default();
        guesses.today_length = today_word_length;
        if guesses.distribution.len() != TRIES {
            guesses.distribution = vec![0; TRIES];
        }
        // Verify if the loaded game state is current.
        let today = utils::date_iso8601();
        if guesses.today != today {
            // Record an abandoned session as a loss.
            if guesses.outcome.len() > 0
                && guesses.outcome.len() < TRIES
                && !guesses.outcome.last().unwrap().is_win()
            {
                guesses.lose();
            }
            guesses.today = today.to_owned();
            guesses.outcome.clear();
        }
        guesses
    }

    // Record a guess.
    pub fn update(&mut self, game: &str, user_id: &str, guess: Guess) -> Result<(), fastly::Error> {
        // Record guess.
        self.outcome.push(guess);
        // Update outcome.
        if self.outcome.last().unwrap().is_win() {
            self.win();
        } else if self.outcome.len() == TRIES {
            self.lose();
        }
        // Save new game stats.
        let mut stats_store = KVStore::open(KV_STORE_NAME)?.unwrap();
        stats_store.insert(
            &format!("{}-{}", game, user_id),
            serde_json::to_string(&self)?,
        )?;
        Ok(())
    }

    // Update win statistics.
    fn win(&mut self) {
        self.distribution[self.outcome.len() - 1] += 1;
        self.last_win = self.today.to_owned();
        self.streak += 1;
        self.games += 1;
        if self.streak > self.max_streak {
            self.max_streak = self.streak;
        }
    }

    // Update loss statistics.
    fn lose(&mut self) {
        self.last_loss = self.today.to_owned();
        self.streak = 0;
        self.games += 1;
    }
}

impl Display for Guess {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "<section class=\"row\">")?;
        for i in 0..self.1.len() {
            write!(
                f,
                "<div class=\"tile\" data-state=\"{}\">{}</div>",
                match self.1[i] {
                    Match::Correct => "correct",
                    Match::Near => "near",
                    Match::Wrong => "wrong",
                },
                self.0.as_bytes()[i] as char
            )?;
        }
        write!(f, "</section>")
    }
}

impl Display for Guesses {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let win_rate = match &self.games {
            0 => 0,
            _ => &self.distribution.iter().sum() / &self.games * 100,
        };
        // TODO: Use a templating engine.
        write!(
            f,
            "{}",
            include_str!("browser/stats.html")
                .replace("{GAMES}", &self.games.to_string())
                .replace("{STREAK}", &self.streak.to_string())
                .replace("{MAX_STREAK}", &self.max_streak.to_string())
                .replace("{WON_TODAY}", &(&self.last_win == &self.today).to_string())
                .replace("{PERC_WON}", &win_rate.to_string())
        )?;
        for i in 0..TRIES {
            write!(
                f,
                "<div class=\"dist\">\
                <div class=\"bar\"><span>{}</span></div>\
                <h5>{}</h5>\
                </div>",
                &self.distribution[i].to_string(),
                i + 1
            )?;
        }
        write!(f, "</div></div>")?;
        for i in 0..TRIES {
            if i < self.outcome.len() {
                write!(f, "{}", &self.outcome[i])?;
            } else {
                if i == self.outcome.len()
                    && self.today != self.last_loss
                    && self.today != self.last_win
                {
                    write!(f, "<section class=\"row active\">")?;
                } else {
                    write!(f, "<section class=\"row\">")?;
                }
                for _ in 0..self.today_length {
                    write!(f, "<div class=\"tile\"></div>")?;
                }
                write!(f, "</section>")?;
            }
        }
        write!(f, "<!-- guesses end -->")
    }
}
