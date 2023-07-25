use crate::guess::{Guess, Guesses, Match, TRIES};
use std::fmt::{Display, Formatter, Result as FmtResult};

const START: &str = include_str!("browser/start.html");
const END: &str = include_str!("browser/end.html");

// impl Display for Guess {
//     fn fmt(&self, f: &mut Formatter) -> FmtResult {
//         write!(f, "<section class=\"row\">")?;
//         for i in 0..self.1.len() {
//             write!(
//                 f,
//                 "<div class=\"tile\" data-state=\"{}\">{}</div>",
//                 match self.1[i] {
//                     Match::Correct => "correct",
//                     Match::Near => "near",
//                     Match::Wrong => "wrong",
//                 },
//                 self.0.as_bytes()[i] as char
//             )?;
//         }
//         write!(f, "</section>")
//     }
// }

// impl Display for Guesses {
//     fn fmt(&self, f: &mut Formatter) -> FmtResult {
//         write!(f, "{}", START)?;
//         for i in 0..TRIES {
//             if i < self.outcome.len() {
//                 write!(f, "{}", &self.outcome[i])?;
//             } else {
//                 if i == self.outcome.len()
//                     && self.today != self.last_loss
//                     && self.today != self.last_win
//                 {
//                     write!(f, "<section class=\"row active\">")?;
//                 } else {
//                     write!(f, "<section class=\"row\">")?;
//                 }
//                 for _ in 0..self.today_length {
//                     write!(f, "<div class=\"tile\"></div>")?;
//                 }
//                 write!(f, "</section>")?;
//             }
//         }
//         write!(f, "{}", END)
//     }
// }
