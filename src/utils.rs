use chrono::{TimeZone, Utc};
use rand::seq::SliceRandom;
use std::collections::HashSet;

// Returns the current timestamp.
pub fn timestamp_now() -> i64 {
    Utc::now().timestamp()
}

// Returns the number of days since a given timestamp.
pub fn get_days_since(timestamp: i64) -> i64 {
    // Convert the given timestamp to a DateTime.
    let given_datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    // Calculate the difference in days.
    Utc::now().signed_duration_since(given_datetime).num_days()
}

// Returns the date in yyyy-mm-dd format.
pub fn date_iso8601() -> String {
    Utc::now().to_rfc3339()[..10].to_string()
}

// Truncates a string to a maximum length.
pub fn truncate_to_chars(s: &str, max_len: usize) -> &str {
    if s.len() > max_len {
        &s[..max_len]
    } else {
        s
    }
}

// Shuffles a vector of strings.
pub fn randomize_vec(strings: &mut Vec<String>) {
    let mut rng = rand::thread_rng();
    strings.shuffle(&mut rng);
}

// Takes a string of words and returns a vector of unique words.
pub fn sanitize_as_words(text: String) -> Vec<String> {
    // Ensure we have unique words.
    let unique_words: HashSet<String> = text
        // Replace whitespace and punctuation with a single space.
        .replace(|c: char| c.is_whitespace() || c.is_ascii_punctuation(), " ")
        // Remove non-ASCII alphabet characters.
        .replace(|c: char| !c.is_whitespace() && !c.is_ascii_alphabetic(), "")
        .to_uppercase()
        .trim()
        // Split into words...
        .split_whitespace()
        // ...that are at least 3 and at most 10 letters long.
        .filter(|&s| s.len() >= 3 && s.len() <= 10)
        .map(String::from)
        // Maximum 365 words.
        .take(365)
        .collect();

    unique_words.into_iter().collect()
}
