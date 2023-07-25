use crate::game::{get_days_since, timestamp_for_today};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

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

    // Output as JSON string.
    pub fn json(&self) -> String {
        serde_json::to_string(&self.1).unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Guesses {
    #[serde(default = "timestamp_for_today")]
    pub today: i64,
    pub today_length: usize,
    pub outcome: Vec<Guess>,
    pub distribution: Vec<u16>,
    pub last_win: i64,
    pub last_loss: i64,
    pub max_streak: u16,
    pub streak: u16,
    pub games: u16,
}

impl Guesses {
    // Initialize a new Guesses struct from state.
    pub fn load(state: &str, today_length: usize) -> Guesses {
        let mut guesses: Guesses = serde_json::from_str(state).unwrap_or_default();
        guesses.today_length = today_length;
        if guesses.distribution.len() != TRIES {
            guesses.distribution = vec![0; TRIES];
        }
        // Verify if the loaded game state is current.
        if get_days_since(guesses.today) != 0 {
            // Record an abandoned session as a loss.
            if guesses.outcome.len() > 0
                && guesses.outcome.len() < TRIES
                && !guesses.outcome.last().unwrap().is_win()
            {
                guesses.lose();
            }
            guesses.today = timestamp_for_today();
            guesses.outcome.clear();
        }

        guesses
    }

    // Record a guess.
    pub fn update(&mut self, guess: Guess) {
        // Record guess.
        self.outcome.push(guess);

        // Update statistics.
        if self.outcome.last().unwrap().is_win() {
            self.win();
        } else if self.outcome.len() == TRIES {
            self.lose();
        }
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
        println!("Win: {:?}", self);
    }

    // Update loss statistics.
    fn lose(&mut self) {
        self.last_loss = self.today.to_owned();
        self.streak = 0;
        self.games += 1;
    }

    // Output as JSON string.
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    // Get last outcome.
    pub fn last_outcome_json(&self) -> String {
        self.outcome.last().unwrap().json()
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
        write!(f, "<!-- guesses start -->")?;
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
