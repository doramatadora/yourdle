use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};

const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

const MAX_AGE: u64 = 365 * 24 * 60 * 60 * 1000; // 1 year

pub fn get_from(slug: &str, cookie_string: &str) -> String {
    let cookie_name = format!("yourdle-{}", slug);
    match cookie_string
        .split("; ")
        .find(|c| c.starts_with(&format!("{}=", cookie_name)))
    {
        Some(c) => match BASE64_ENGINE.decode(&c[(&cookie_name.len() + 1)..]) {
            Ok(s) => std::str::from_utf8(&s).unwrap().to_owned(),
            Err(e) => "".to_owned(),
        },
        None => "".to_owned(),
    }
}

pub fn as_cookie(slug: &str, value: &str) -> String {
    format!(
        "yourdle-{}={}; Max-Age={}; Path=/{}; SameSite=Lax; Secure",
        slug,
        BASE64_ENGINE.encode(value),
        MAX_AGE,
        slug,
    )
}
