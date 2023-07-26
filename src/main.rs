use chrono::Utc;
use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};
mod game;
mod guess;
mod state;
mod utils;

use game::{GameData, GameDataForm};
use guess::{Guess, Guesses};

// const LONG_CACHE: &str = "public, max-age=21600, immutable";
const LONG_CACHE: &str = "private, no-cache, max-age=0, no-store";

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
        "/favicon.png" => Ok(png_resp(include_bytes!("browser/images/favicon.png"))),
        "/card.png" => Ok(png_resp(include_bytes!("browser/images/card.png"))),
        "/yourdle.svg" => Ok(svg_resp(include_str!("browser/images/yourdle.svg"))),
        "/info.svg" => Ok(svg_resp(include_str!("browser/images/info.svg"))),
        "/contrast.svg" => Ok(svg_resp(include_str!("browser/images/contrast.svg"))),
        "/stats.svg" => Ok(svg_resp(include_str!("browser/images/stats.svg"))),
        "/style.css" => Ok(text_resp(
            mime::TEXT_CSS_UTF_8,
            include_str!("browser/style.css"),
        )),
        "/script.js" => Ok(js_resp(include_bytes!("browser/script.js"))),
        "/new.js" => Ok(js_resp(include_bytes!("browser/new.js"))),
        "/" => Ok(html_resp(include_str!("browser/index.html"))),
        "/feedback" => Ok(text_resp(mime::TEXT_PLAIN, "Report game page here")),
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
            _ => Ok(html_resp(include_str!("browser/new.html"))),
        },
        req_path => {
            let game_slug = match req_path[1..].find('/') {
                Some(i) => &req_path[1..i],
                None => &req_path[1..],
            };
            // Load game data.
            if let Ok(mut game_data) = game::GameData::load(game_slug) {
                // Load today's word.
                let (word, _, _) = game_data.get_word().unwrap();
                // Get the date in yyyy-mm-dd format.
                let today: &str = &Utc::now().to_rfc3339()[..10];
                // Load game state.
                let mut guesses = Guesses::load(
                    today,
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
                            .with_body_json(&guesses.outcome.last())?);
                    }
                    // Respond with 404 if the word isn't in the list.
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
            Ok(Response::from_status(StatusCode::NOT_FOUND)
                .with_body_text_html(&include_str!("browser/404.html")))
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
fn png_resp(body: &[u8]) -> Response {
    byte_resp(mime::IMAGE_PNG, body)
}
fn svg_resp(body: &str) -> Response {
    text_resp(mime::IMAGE_SVG, body)
}

fn js_resp(body: &[u8]) -> Response {
    byte_resp(mime::APPLICATION_JAVASCRIPT_UTF_8, body)
}

fn html_resp(body: &str) -> Response {
    Response::from_status(StatusCode::OK).with_body_text_html(body)
}
