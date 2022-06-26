#![feature(iter_intersperse)]
mod commands;
mod data;
mod listener;

use std::sync::Arc;

use data::Data;
use log::info;
use poise::{
    serenity_prelude::{self as serenity, Color, Colour, UserId},
    PrefixFrameworkOptions,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

const EMBED_COLOR: Color = Colour(0x2F3136);
const BOT_ID: UserId = UserId(973661305502330881);

#[tokio::main]
async fn main() {
    let bot_token = std::env::var("GPT_BOT_TOKEN").unwrap();
    let gpt_token = std::env::var("GPT_API_TOKEN").unwrap();

    env_logger::builder()
        .filter(Some("catchphrase::listener"), log::LevelFilter::Debug)
        .init();
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
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            owners: { [UserId(375636762094993419), UserId(821483662297923664)].into() },
            listener: listener::listener,
            ..Default::default()
        })
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| {
            info!("starting bot");
            let data = Arc::new(Data::new(gpt_token));
            Box::pin(async move { Ok(Arc::clone(&data)) })
        })
        .run()
        .await
        .unwrap();
}
