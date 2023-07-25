use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};
mod game;
mod guess;
mod page;
mod state;

use guess::{Guess, Guesses};

// const LONG_CACHE: &str = "public, max-age=21600, immutable";
const LONG_CACHE: &str = "private, no-cache, max-age=0, no-store";

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    match req.get_method() {
        &Method::GET | &Method::HEAD | &Method::POST => (),
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD, POST"))
        }
    };
    // Route requests.
    match req.get_path() {
        "/favicon.png" => Ok(png_resp(include_bytes!("browser/images/favicon.png"))),
        "/card.png" => Ok(png_resp(include_bytes!("browser/images/card.png"))),
        "/yourdle.svg" => Ok(text_resp(
            mime::IMAGE_SVG,
            include_str!("browser/images/yourdle.svg"),
        )),
        "/style.css" => Ok(text_resp(
            mime::TEXT_CSS_UTF_8,
            include_str!("browser/style.css"),
        )),
        "/script.js" => Ok(byte_resp(
            mime::APPLICATION_JAVASCRIPT_UTF_8,
            include_bytes!("browser/script.js"),
        )),
        "/" => Ok(html_resp(include_str!("browser/index.html"))),
        "/new" => match req.get_method() {
            &Method::POST => Ok(html_resp(include_str!("browser/new.html"))),
            _ => Ok(html_resp(include_str!("browser/new.html"))),
        },
        "/report" => Ok(text_resp(mime::TEXT_PLAIN, "Report game page here")),
        req_path => {
            let game_slug = match req_path[1..].find('/') {
                Some(i) => &req_path[1..i],
                None => &req_path[1..],
            };
            println!("The game is {}", game_slug);
            // Load game data.
            if let Ok(mut game_data) = game::GameData::load(game_slug) {
                // Load today's word.
                let (word, word_idx, total_words) = game_data.get_word().unwrap();
                println!("Loaded word: {}, {}/{}", word, word_idx, total_words);

                println!(
                    "Loaded state: {:?}",
                    &state::get_from(game_slug, req.get_header_str("cookie").unwrap_or_default())
                );
                // Load game state.
                let mut guesses = Guesses::load(
                    &state::get_from(game_slug, req.get_header_str("cookie").unwrap_or_default()),
                    word.len(),
                );

                // Record a guess, if the guess query parmeter is set.
                if let Some(guess) = req.get_query_parameter("guess") {
                    if game_data.validate_word(guess) {
                        // Check if the guess is correct.
                        guesses.update(Guess::new(&guess, &word));
                        // Save new state and respond with outcome.
                        return Ok(Response::from_status(StatusCode::OK)
                            .with_header(
                                header::SET_COOKIE,
                                state::as_cookie(game_slug, &guesses.json()),
                            )
                            .with_body_text_plain(&guesses.last_outcome_json()));
                    }
                    println!("Invalid guess: {}", guess);
                    // Respond with 404 for invalid PoP.
                    return Ok(Response::from_status(StatusCode::NOT_FOUND));
                }
                // Render the game index.
                return Ok(Response::from_status(StatusCode::OK)
                    .with_header(
                        header::SET_COOKIE,
                        state::as_cookie(game_slug, &guesses.json()),
                    )
                    .with_body_text_html(&format!(
                        "{}{}{}",
                        game_data,
                        guesses,
                        include_str!("browser/end.html")
                    )));
            }
            // Respond with 404 for anything else.
            Ok(Response::from_status(StatusCode::NOT_FOUND).with_body_text_html(&include_str!("browser/404.html")))
        }
    }
}

fn byte_resp(mime_type: mime::Mime, body: &[u8]) -> Response {
    Response::from_status(StatusCode::OK)
        .with_content_type(mime_type)
        .with_header(header::CACHE_CONTROL, LONG_CACHE)
        .with_body_octet_stream(body)
}

fn text_resp(mime_type: mime::Mime, body: &str) -> Response {
    Response::from_status(StatusCode::OK)
        .with_content_type(mime_type)
        .with_header(header::CACHE_CONTROL, LONG_CACHE)
        .with_body(body)
}

fn html_resp(body: &str) -> Response {
    Response::from_status(StatusCode::OK).with_body_text_html(body)
}

fn png_resp(body: &[u8]) -> Response {
    byte_resp(mime::IMAGE_PNG, body)
}
