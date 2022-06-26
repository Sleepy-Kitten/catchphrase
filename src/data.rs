use std::time::Duration;

use gpt3_rs::Client;
use poise::serenity_prelude::RwLock;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

pub struct Data {
    pub client: Client,
    pub last_message: RwLock<Instant>,
    pub phrases: RwLock<Vec<Phrase>>,
    pub config: RwLock<Config>,
}
pub struct Phrase {
    pub last_phrase: Instant,
    pub content: String,
}
impl Phrase {
    pub fn new_without_cooldown(content: String, cooldown: u16) -> Self {
        Self {
            last_phrase: Instant::now() - Duration::from_secs(cooldown as u64),
            content,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub minimum_score: u16,
    pub max_context_len: usize,
    pub chance: u8,
    pub cooldown: u16,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            cooldown: 20,
            chance: 25,
            max_context_len: 512,
            minimum_score: 20,
        }
    }
}
impl Data {
    pub fn new(token: String) -> Self {
        let config = Config::default();
        Self {
            client: Client::new(token),
            last_message: RwLock::new(Instant::now() - Duration::from_secs(config.cooldown as u64)),
            phrases: Default::default(),
            config: RwLock::new(config),
        }
    }
}
