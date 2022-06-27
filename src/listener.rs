use crate::{data::Data, Error, BOT_ID};
use gpt3_rs::Request;
use log::{debug, info};
use poise::{BoxFuture, Event, FrameworkContext};
use std::sync::Arc;
use tokio::time::Instant;

pub fn listener<'a>(
    context: &'a poise::serenity_prelude::Context,
    event: &'a Event<'a>,
    _framework: FrameworkContext<'a, Arc<Data>, Error>,
    data: &'a Arc<Data>,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        match event {
            Event::Message { new_message } => {
                // return if not guild message
                let Some(guild_id) = new_message.guild_id else {
                    return Ok(())
                };
                // return if not a registered guild
                let Some(guild_meta_lock) = data.get_guild(guild_id).await else {
                    return Ok(())
                };

                // guild metadata guard
                let mut guild_meta = guild_meta_lock.write().await;

                let config = &guild_meta.config;
                let chance = config.chance;
                let context_len = config.max_context_len;
                let minimum_score = config.minimum_score;
                let cooldown = config.cooldown;
                let last_response = guild_meta.last_response;
                let phrases = &guild_meta.phrases;

                if new_message.author.id == *BOT_ID.get().unwrap() {
                    return Ok(());
                }

                // return if on cooldown
                match last_response {
                    Some(last_response) if last_response.elapsed().as_secs() < cooldown as u64 => {
                        return Ok(())
                    }
                    _ => {}
                }

                if fastrand::u8(0..=100) <= chance {
                    debug!("message chance occured");

                    // collect phrases for request
                    let phrases_content = phrases.iter().cloned().collect::<Vec<_>>();

                    let client = &data.client;

                    // fetches recent messages
                    let messages = new_message
                        .channel_id
                        .messages(&context.http, |m| m.limit(10))
                        .await?;

                    // merges recent messages into one
                    let merged = messages
                        .iter()
                        .rev()
                        .map(|messages| &*messages.content)
                        .intersperse(" ")
                        .collect::<String>();

                    // finds start index of message within bounds of max context length
                    let mut acc = 0;
                    let start = merged
                        .split(' ')
                        .rev()
                        .take_while(|word| {
                            acc += word.len() + 1;
                            acc < context_len
                        })
                        .fold(0, |start, word| start + word.len() + 1);
                    let (_, message_text) = merged.split_at(merged.len() - (start - 1));

                    debug!("query:\n{}", message_text);

                    // build search request
                    let response = gpt3_rs::api::searches::Builder::default()
                        .model(gpt3_rs::Model::Babbage)
                        .documents(phrases_content)
                        .query(message_text)
                        .build()?
                        .request(client)
                        .await?;

                    debug!("response:\n{:#?}", response);

                    // gets highest score and check if it's above the threshold
                    let index = response
                        .data
                        .into_iter()
                        .reduce(|acc, data| if acc.score < data.score { data } else { acc })
                        .filter(|data| data.score >= minimum_score as f64)
                        .map(|data| data.document);

                    if let Some(catchphrase) = index.and_then(|i| phrases.iter().nth(i)).cloned() {
                        let phrase_cooldown = guild_meta.cooldown.get(&catchphrase);

                        // if phrase has cooldown
                        if phrase_cooldown.is_some_and(|last_phrase| {
                            last_phrase.elapsed().as_secs() > cooldown as u64
                        }) || phrase_cooldown.is_none()
                        {
                            // reset cooldowns
                            guild_meta.last_response = Some(Instant::now());
                            guild_meta
                                .cooldown
                                .insert(catchphrase.clone(), Instant::now());

                            info!("found catchphrase: {}", catchphrase);

                            // send phrase
                            new_message
                                .channel_id
                                .say(&context.http, catchphrase)
                                .await?;
                        }
                    }
                }
            }
            // adds newly joined guilds to the guild map
            Event::GuildCreate { guild, ..}=> {
                let guild_id = guild.id;
                let mut guild_meta_map = data.guild_meta_map.write().await;
                guild_meta_map.entry(guild_id).or_default();
            },
            _ => {}
        }
        Ok(())
    })
}
