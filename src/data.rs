use std::{collections::{HashMap, HashSet}, sync::Arc};

use gpt3_rs::Client;
use poise::serenity_prelude::{ChannelId, GuildId, RwLock};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

pub struct Data {
    pub client: Client,
    pub guild_meta_map: RwLock<HashMap<GuildId, Arc<RwLock<GuildMeta>>>>,
}
impl Data {
    /// regrieves a guild from the guild map
    pub async fn get_guild(&self, guild_id: GuildId) -> Option<Arc<RwLock<GuildMeta>>> {
        let guild_meta_map = self.guild_meta_map.read().await;
        guild_meta_map.get(&guild_id).cloned()
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct GuildMeta {
    #[serde(skip)]
    pub last_response: Option<Instant>,
    #[serde(skip)]
    pub cooldown: HashMap<String, Instant>,
    pub phrases: HashSet<String>,
    pub channels: HashSet<ChannelId>,
    pub config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub minimum_score: u16,
    pub max_context_len: usize,
    pub chance: u8,
    pub cooldown: u16,
    pub model: gpt3_rs::Model,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            cooldown: 60,
            chance: 25,
            max_context_len: 512,
            minimum_score: 20,
            model: gpt3_rs::Model::Babbage,
        }
    }
}
impl Data {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(token),
            guild_meta_map: Default::default(),
        }
    }
}
