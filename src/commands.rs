use std::collections::HashSet;

use log::{debug, warn};
use ron::ser::PrettyConfig;

use crate::{Context, Error, DATA_PATH, EMBED_COLOR};

/// Adds a catchphrase
#[poise::command(slash_command, owners_only)]
pub async fn add_catchphrase(
    ctx: Context<'_>,
    #[description = "Added catchphrase"] catchphrase: String,
    #[description = "Optional keywords to associate with that catchphrase"] keywords: Vec<String>,
) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let mut guild_meta = guild_meta_lock.write().await;

    ctx.send(|r| {
        r.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Added catchphrase");
            e.field("catchphrase: ", &catchphrase, true);
            if !keywords.is_empty() {
                e.field(
                    "keywords: ",
                    keywords
                        .iter()
                        .map(|keyword| &**keyword)
                        .collect::<String>(),
                    false,
                );
            }
            e
        })
    })
    .await?;

    guild_meta.phrases.insert(
        catchphrase.clone(),
        keywords.into_iter().collect::<HashSet<_>>(),
    );

    Ok(())
}

/// Lists all catchphrases
#[poise::command(slash_command)]
pub async fn list_catchphrases(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let guild_meta = guild_meta_lock.read().await;

    let phrases = &guild_meta.phrases;

    if !phrases.is_empty() {
        let phrases = phrases.iter().map(|(phrase, keywords)| {
            (
                phrase,
                format!(
                    "keywords: {}",
                    keywords
                        .iter()
                        .map(|keyword| &**keyword)
                        .intersperse(", ")
                        .collect::<String>()
                ),
                false,
            )
        });

        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Phrases");
                e.fields(phrases)
            })
        })
        .await?;
    } else {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Removed phrase");
                e.field("error: ", "this guild has no catchphrases", true)
            })
        })
        .await?;
    }

    Ok(())
}

/// Removes a catchphrase
#[poise::command(slash_command, owners_only)]
pub async fn remove_catchphrase(
    ctx: Context<'_>,
    #[description = "Removed catchphrase"] phrase: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let mut guild_meta = guild_meta_lock.write().await;

    let phrase_existed = guild_meta.phrases.remove(&phrase).is_some();

    if phrase_existed {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Removed phrase");
                e.field("phrase: ", &phrase, true)
            })
        })
        .await?;
    } else {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Removed phrase");
                e.field("error: ", "phrase not found", true)
            })
        })
        .await?;
    }

    Ok(())
}
/// shows the bot config
#[poise::command(slash_command, owners_only)]
pub async fn show_config(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let guild_meta = guild_meta_lock.read().await;

    let config = &guild_meta.config;
    let config = ron::ser::to_string_pretty(config, PrettyConfig::default())?;

    ctx.send(|r| {
        r.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Config");
            e.field("config values:", format!("```\n{config}\n```"), false)
        })
    })
    .await?;
    Ok(())
}
/// loads the bot config
#[poise::command(slash_command, owners_only)]
pub async fn load_config(
    ctx: Context<'_>,
    #[description = "the config to load"] config: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let new_config = ron::from_str(&config);

    if let Ok(new_config) = new_config {
        let guild_meta_lock = data
            .get_guild(ctx.guild_id().unwrap())
            .await
            .expect("guild not found");
        let mut guild_meta = guild_meta_lock.write().await;

        guild_meta.config = new_config;
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Edit config");
                e.field("config values:", config, false)
            })
        })
        .await?;
    } else {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Edit config");
                e.field("config values:", "invalid format", true)
            })
        })
        .await?;
    }
    Ok(())
}

/// registers slash commands
#[poise::command(prefix_command, owners_only)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// add this channel
#[poise::command(slash_command, owners_only)]
pub async fn add_channel(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let mut guild_meta = guild_meta_lock.write().await;

    let channels = &mut guild_meta.channels;
    let channel = ctx.channel_id();
    channels.insert(channel);
    ctx.say("added this channel").await?;
    Ok(())
}

/// removes this channel
#[poise::command(slash_command, owners_only)]
pub async fn remove_channel(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_lock = data
        .get_guild(ctx.guild_id().unwrap())
        .await
        .expect("guild not found");
    let mut guild_meta = guild_meta_lock.write().await;

    let channels = &mut guild_meta.channels;
    let channel = ctx.channel_id();
    channels.remove(&channel);
    ctx.say("removed this channel").await?;
    Ok(())
}

/// dumps config data to a file
#[poise::command(slash_command, owners_only)]
pub async fn dump_configs(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild_meta_map = data.guild_meta_map.read().await;
    ctx.say("attempting to dump configs to file").await?;

    let mut errors: Option<u32> = None;

    for (guild_id, guild_meta_lock) in guild_meta_map.iter() {
        let guild_id = guild_id.0;
        let guild_meta = guild_meta_lock.read().await;
        let text = ron::ser::to_string_pretty(&*guild_meta, Default::default())?;
        drop(guild_meta);

        match tokio::fs::write(format!("{DATA_PATH}/{guild_id}.ron"), text).await {
            Ok(_) => {
                debug!("dumped guild: {}", guild_id);
            }
            Err(err) => {
                warn!("error while dumping guild: {}", guild_id);
                warn!("error: `{}`", err);
                errors = Some(errors.unwrap_or_default() + 1);
            }
        }
    }

    if let Some(errors) = errors {
        ctx.say(format!("failed to dump {} guilds", errors)).await?;
    } else {
        ctx.say("dumped configs succesfully").await?;
    }
    Ok(())
}

/// loads phrases from serializable format
#[poise::command(slash_command, owners_only)]
pub async fn load_phrases(
    ctx: Context<'_>,
    #[description = "the phrases to load"] phrases: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let phrases_loaded: Option<HashSet<String>> = ron::de::from_str(&*phrases).ok();

    if let Some(mut phrases_loaded) = phrases_loaded {
        let guild_meta_lock = data
            .get_guild(ctx.guild_id().unwrap())
            .await
            .expect("guild not found");
        let mut guild_meta = guild_meta_lock.write().await;

        for phrase in phrases_loaded.drain() {
            guild_meta.phrases.insert(phrase, Default::default());
        }
        ctx.say("loaded phrases").await?;
    } else {
        ctx.say("error loading phrases").await?;
    }
    Ok(())
}
