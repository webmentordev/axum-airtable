use nanoid::nanoid;
use rand::{Rng, distributions::Alphanumeric};

pub fn generate_id(prefix: &str) -> String {
    let limit = rand::thread_rng().gen_range(17..=21);
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(limit)
        .map(char::from)
        .collect();

    format!("{}{}", prefix, random)
}

pub fn generate_token(prefix: &str) -> String {
    let token = format!("{}_{}", prefix, nanoid!(64));
    token
}
