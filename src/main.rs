use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, KVStore, Request, Response};
mod game;
mod guess;
mod state;
mod utils;

use game::{GameData, GameDataForm};
use guess::{Guess, Guesses};

// const LONG_CACHE: &str = "public, max-age=21600, immutable";
const LONG_CACHE: &str = "public, max-age=3600, must-revalidate";
// const LONG_CACHE: &str = "private, no-cache, max-age=0, no-store";

#[fastly::main]
fn main(mut req: Request) -> Result<Response, Error> {
    match req.get_method() {
        &Method::GET | &Method::HEAD | &Method::POST => (),
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD, POST"))
        }
    };
    // Route requests.
    match req.get_path() {
        // Static assets.
        "/favicon.png" => Ok(png(include_bytes!("browser/images/favicon.png"))),
        "/card.png" => Ok(png(include_bytes!("browser/images/card.png"))),
        "/yourdle.svg" => Ok(svg(include_str!("browser/images/yourdle.svg"))),
        "/info.svg" => Ok(svg(include_str!("browser/images/info.svg"))),
        "/contrast.svg" => Ok(svg(include_str!("browser/images/contrast.svg"))),
        "/stats.svg" => Ok(svg(include_str!("browser/images/stats.svg"))),
        "/style.css" => Ok(text(mime::TEXT_CSS, include_str!("browser/style.css"))),
        "/script.js" => Ok(js(include_bytes!("browser/script.js"))),
        "/new.js" => Ok(js(include_bytes!("browser/new.js"))),
        "/" => Ok(html(include_str!("browser/index.html"))),
        // All other routes.
        "/feedback" => {
            if req.get_method() == &Method::POST {
                if let Ok(Some(mut feedback_store)) = KVStore::open("yourdle-feedback") {
                    let feedback = req.take_body_str();
                    let cookie = req.get_header_str("cookie").unwrap_or_default();
                    let user_id = state::get_user_id(cookie);
                    if let Ok(_) = feedback_store.insert(
                        &format!("{}-{}", utils::timestamp_now(), user_id),
                        utils::truncate_to_chars(&feedback, 300),
                    ) {
                        return Ok(with_cookie(StatusCode::OK, &state::set_user_id(&user_id)));
                    }
                }
            }
            Ok(Response::from_status(StatusCode::BAD_REQUEST))
        }
        "/validate" => {
            if let Ok(form) = req.take_body_json::<GameDataForm>() {
                if GameData::check_not_exists(&form.game).is_ok() {
                    return Ok(Response::from_status(StatusCode::OK));
                }
            }
            Ok(Response::from_status(StatusCode::BAD_REQUEST))
        }
        "/new" => match req.get_method() {
            &Method::POST => {
                if let Ok(form) = req.take_body_json::<GameDataForm>() {
                    if let Ok(mut game_data) = GameData::from_form(form) {
                        if game_data.save().is_ok() {
                            return Ok(Response::from_status(StatusCode::OK)
                                .with_body_text_plain(&game_data.slug));
                        }
                    }
                }
                Ok(Response::from_status(StatusCode::BAD_REQUEST))
            }
            _ => Ok(html(include_str!("browser/new.html"))),
        },
        // Game routes (yourdle.edgecomptech.com/game-slug).
        req_path => {
            let game = &req_path[1..];
            if !game.contains("/") {
                // Load game data.
                if let Ok(mut game_data) = game::GameData::load(game) {
                    // Load today's word.
                    let (word, _, _) = game_data.get_word().unwrap();
                    // Get the user ID from the cookie (or create a new one).
                    let cookie = req.get_header_str("cookie").unwrap_or_default();
                    let user_id = state::get_user_id(cookie);
                    // Load game stats.
                    let mut guesses = Guesses::load(&game, &user_id, word.len());
                    // Record a guess, if the guess query parmeter is set.
                    if let Some(guess) = req.get_query_parameter("guess") {
                        // Check if the guessed word is in the game's list of words.
                        if game_data.validate_word(guess) {
                            // Update guesses (save stats) and respond with outcome.
                            guesses.update(&game, &user_id, Guess::new(&guess, &word))?;
                            return Ok(with_cookie(
                                StatusCode::OK,
                                &state::set_user_id(&user_id),
                            )
                            .with_body_json(&guesses.outcome.last())?);
                        }
                        // Respond with 404 if the word isn't in the list.
                        return Ok(with_cookie(
                            StatusCode::NOT_FOUND,
                            &state::set_user_id(&user_id),
                        ));
                    }
                    // Render the game index.
                    return Ok(with_cookie(StatusCode::OK, &state::set_user_id(&user_id))
                        .with_body_text_html(&format!(
                            "{}{}{}",
                            game_data,
                            guesses,
                            include_str!("browser/end.html")
                        )));
                }
            }
            // Respond with 404 for anything else.
            Ok(Response::from_status(StatusCode::NOT_FOUND)
                .with_body_text_html(&include_str!("browser/404.html")))
        }
    }
}

// Response helpers (useful for serving the frontend).
fn long_cache_resp(mime_type: mime::Mime) -> Response {
    Response::from_status(StatusCode::OK)
        .with_content_type(mime_type)
        .with_header(header::CACHE_CONTROL, LONG_CACHE)
}

fn bytes(mime_type: mime::Mime, body: &[u8]) -> Response {
    long_cache_resp(mime_type).with_body_octet_stream(body)
}

fn text(mime_type: mime::Mime, body: &str) -> Response {
    long_cache_resp(mime_type).with_body(body)
}

fn png(body: &[u8]) -> Response {
    bytes(mime::IMAGE_PNG, body)
}

fn svg(body: &str) -> Response {
    text(mime::IMAGE_SVG, body)
}

fn js(body: &[u8]) -> Response {
    bytes(mime::APPLICATION_JAVASCRIPT_UTF_8, body)
}

fn html(body: &str) -> Response {
    Response::from_status(StatusCode::OK).with_body_text_html(body)
}

fn with_cookie(status: StatusCode, cookie: &str) -> Response {
    Response::from_status(status).with_header(header::SET_COOKIE, cookie)
}
