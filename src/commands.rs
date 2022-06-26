use ron::ser::PrettyConfig;

use crate::{data::Phrase, Context, Error, EMBED_COLOR};

/// Adds a catchphrase
#[poise::command(slash_command, owners_only)]
pub async fn add_catchphrase(
    ctx: Context<'_>,
    #[description = "Added catchphrase"] catchphrase: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let mut phrases = data.phrases.write().await;
    let config = data.config.read().await;
    let cooldown = config.cooldown;
    phrases.push(Phrase::new_without_cooldown(catchphrase.clone(), cooldown));

    ctx.send(|r| {
        r.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Added catchphrase");
            e.field("catchphrase: ", catchphrase, true)
        })
    })
    .await?;
    Ok(())
}

/// Lists all catchphrases
#[poise::command(slash_command)]
pub async fn list_catchphrases(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let phrases = data.phrases.read().await;
    let phrases = phrases
        .iter()
        .map(|phrase| &phrase.content)
        .cloned()
        .enumerate()
        .map(|(index, string)| (index, string, true));

    ctx.send(|r| {
        r.embed(|e| {
            e.color(EMBED_COLOR);
            e.title("Phrases");
            e.fields(phrases)
        })
    })
    .await?;
    Ok(())
}

/// Removes a catchphrase by index
#[poise::command(slash_command, owners_only)]
pub async fn remove_catchphrase(
    ctx: Context<'_>,
    #[description = "Removed catchphrase index"] index: usize,
) -> Result<(), Error> {
    let data = ctx.data();

    let mut phrases = data.phrases.write().await;
    let phrase = phrases.get(index);
    if let Some(phrase) = phrase {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Removed phrase");
                e.field("phrase: ", &phrase.content, true)
            })
        })
        .await?;
        phrases.swap_remove(index);
    } else {
        ctx.send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Removed phrase");
                e.field("error: ", format!("index: {index} is out of bounds"), true)
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
    let config = &data.config.read().await;
    let config = ron::ser::to_string_pretty(&**config, PrettyConfig::default())?;

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
pub async fn load_config(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let handle = ctx
        .send(|r| {
            r.embed(|e| {
                e.color(EMBED_COLOR);
                e.title("Edit config");
                e.field("config values:", "awaiting message", true)
            })
        })
        .await?;

    let message = ctx
        .channel_id()
        .await_reply(&ctx.discord().shard)
        .await
        .ok_or("failed awaiting message")?;
    let content = &message.content;

    if content.starts_with("```") && content.ends_with("```") {
        let config_str = &content[3..content.len() - 3];
        let config = ron::from_str(config_str);

        if let Ok(config) = config {
            let mut current_config = data.config.write().await;
            *current_config = config;
            handle
                .edit(ctx, |r| {
                    r.embed(|e| {
                        e.color(EMBED_COLOR);
                        e.title("Edit config");
                        e.field("config values:", config_str, false)
                    })
                })
                .await?;
            message.delete(&ctx.discord().http).await?;
        } else {
            handle
                .edit(ctx, |r| {
                    r.embed(|e| {
                        e.color(EMBED_COLOR);
                        e.title("Edit config");
                        e.field("config values:", "invalid format", true)
                    })
                })
                .await?;
        }
    } else {
        handle
            .edit(ctx, |r| {
                r.embed(|e| {
                    e.color(EMBED_COLOR);
                    e.title("Edit config");
                    e.field("config values:", "expected codeblock", true)
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
