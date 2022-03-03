use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn random_string(length: u8) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}