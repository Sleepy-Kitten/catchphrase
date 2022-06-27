#![feature(iter_intersperse)]
#![feature(let_else)]
#![feature(is_some_with)]
#![feature(async_closure)]

mod commands;
mod data;
mod listener;

use data::Data;
use log::{debug, info, warn};
use poise::{
    serenity_prelude::{self as serenity, Color, Colour, GuildId, RwLock, UserId},
    PrefixFrameworkOptions,
};
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::data::GuildMeta;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

static BOT_ID: OnceCell<UserId> = OnceCell::const_new();
const EMBED_COLOR: Color = Colour(0x2F3136);
const DATA_PATH: &str = "guilds";

#[tokio::main]
async fn main() {
    let bot_token = std::env::var("GPT_BOT_TOKEN").unwrap();
    let gpt_token = std::env::var("GPT_API_TOKEN").unwrap();

    // init logger
    env_logger::builder()
        .filter(Some("catchphrase"), log::LevelFilter::Debug)
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
                BOT_ID
                    .get_or_init(async || _ctx.cache.current_user_id())
                    .await;
                let data = Arc::new(Data::new(gpt_token));

                let mut guild_meta_map = data.guild_meta_map.write().await;

                // init joined guilds with default values
                for guild in _ready.guilds.iter() {
                    guild_meta_map.entry(guild.id).or_default();
                    debug!("initialized: {}", guild.id)
                }

                // walk dir and add config files to data
                if let Ok(mut guild_files) = tokio::fs::read_dir(DATA_PATH).await {
                    while let Ok(Some(guild_file)) = guild_files.next_entry().await {
                        // cursed "try" block ( || {} )()
                        match (async || -> Result<_, Error> {
                            let file_name = guild_file.file_name();
                            let file_name = file_name.to_string_lossy();

                            // get id from file name
                            let id = file_name[..file_name.len() - 4].parse::<u64>()?;
                            // file content
                            let bytes = tokio::fs::read(guild_file.path()).await?;
                            // deserialzed metadata
                            let guild_meta = ron::de::from_bytes::<GuildMeta>(&bytes)?;

                            Ok((id, guild_meta))
                        })()
                        .await
                        {
                            Ok((id, guild_meta)) => {
                                debug!("loaded guild: {}", id);
                                guild_meta_map
                                    .insert(GuildId(id), Arc::new(RwLock::new(guild_meta)));
                            }
                            Err(err) => warn!("error loading guild: {}", err),
                        }
                    }
                }

                drop(guild_meta_map);

                Ok(data)
            })
        })
        .run()
        .await
        .unwrap();
}
