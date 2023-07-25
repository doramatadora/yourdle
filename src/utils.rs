use chrono::{TimeZone, Utc};
use rand::seq::SliceRandom;
use std::collections::HashSet;

pub fn get_days_since(timestamp: i64) -> i64 {
    // Convert the given timestamp to a DateTime.
    let given_datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    // Calculate the difference in days.
    Utc::now().signed_duration_since(given_datetime).num_days()
}

pub fn timestamp_for_today() -> i64 {
    Utc::now().timestamp()
}

pub fn randomize_vec(strings: &mut Vec<String>) {
    let mut rng = rand::thread_rng();
    strings.shuffle(&mut rng);
}

// Takes a string of words and returns a vector of unique words.
pub fn sanitize_as_words(text: String) -> Vec<String> {
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

pub fn truncate_to_max_length(s: &str, max_len: usize) -> &str {
    if s.len() > max_len {
        &s[..max_len]
    } else {
        s
    }
}
