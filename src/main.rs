#![feature(iter_intersperse)]
#![feature(let_else)]
#![feature(is_some_with)]
#![feature(async_closure)]

mod commands;
mod data;
mod listener;

use data::Data;
use log::info;
use poise::{
    serenity_prelude::{self as serenity, Color, Colour, GuildId, RwLock, UserId},
    PrefixFrameworkOptions,
};
use std::sync::Arc;

use crate::data::GuildMeta;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

const EMBED_COLOR: Color = Colour(0x2F3136);
const BOT_ID: UserId = UserId(973661305502330881);
const DATA_PATH: &str = "servers";

#[tokio::main]
async fn main() {
    let bot_token = std::env::var("GPT_BOT_TOKEN").unwrap();
    let gpt_token = std::env::var("GPT_API_TOKEN").unwrap();

    // init logger
    env_logger::builder()
        .filter(Some("catchphrase::listener"), log::LevelFilter::Debug)
        .init();
        
    // start framework
    poise::Framework::build()
        .token(bot_token)
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::register(),
                commands::add_catchphrase(),
                commands::remove_catchphrase(),
                commands::list_catchphrases(),
                commands::show_config(),
                commands::load_config(),
                commands::load_phrases(),
                commands::dump_configs(),
                commands::add_channel(),
                commands::remove_channel(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            owners: {
                [
                    UserId(375636762094993419),
                    UserId(821483662297923664),
                    UserId(842700734583930881),
                ]
                .into()
            },
            listener: listener::listener,
            ..Default::default()
        })
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| {
            info!("starting bot");

            Box::pin(async move {
                let data = Arc::new(Data::new(gpt_token));

                let mut guild_meta_map = data.guild_meta_map.write().await;

                // walk dir and add config files to data
                if let Ok(mut server_files) = tokio::fs::read_dir(DATA_PATH).await {
                    while let Ok(Some(server_file)) = server_files.next_entry().await {
                        // cursed "try" block ( || {} )()
                        if let Some((id, guild_meta)) = (async || {
                            // file name is guild id
                            let id = server_file
                                .file_name()
                                .to_string_lossy()
                                .parse::<u64>()
                                .ok()?;
                            // file content
                            let bytes = tokio::fs::read(server_file.path()).await.ok()?;
                            // deserialzed metadata
                            let guild_meta = ron::de::from_bytes::<GuildMeta>(&bytes).ok()?;

                            Some((id, guild_meta))
                        })()
                        .await
                        {
                            guild_meta_map.insert(GuildId(id), Arc::new(RwLock::new(guild_meta)));
                        }
                    }
                }

                // init guilds not saved as files
                for guild in _ready.guilds.iter() {
                    guild_meta_map.entry(guild.id).or_default();
                }

                drop(guild_meta_map);

                Ok(data)
            })
        })
        .run()
        .await
        .unwrap();
}
