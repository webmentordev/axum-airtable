use nanoid::nanoid;
use rand::{Rng, distributions::Alphanumeric};

pub fn generate_id(prefix: &str) -> String {
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(17)
        .map(char::from)
        .collect();

    format!("{}{}", prefix, random)
}

pub fn generate_token(prefix: &str) -> String {
    let token = format!("{}_{}", prefix, nanoid!(64));
    token
}
