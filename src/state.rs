use uuid::Uuid;

const COOKIE_NAME: &str = "yourdle";
const MAX_AGE: u64 = 365 * 24 * 60 * 60 * 1000; // 1 year

pub fn get_user_id(cookie_string: &str) -> String {
    match cookie_string
        .split("; ")
        .find(|c| c.starts_with(&format!("{}=", COOKIE_NAME)))
    {
        Some(c) => c[(&COOKIE_NAME.len() + 1)..].to_string(),
        None => Uuid::new_v4().to_string(),
    }
}

pub fn set_user_id(id: &str) -> String {
    format!(
        "{}={}; Max-Age={}; Path=/; SameSite=Lax; Secure",
        COOKIE_NAME, id, MAX_AGE
    )
}
