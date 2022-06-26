use crate::{data::Data, Error, BOT_ID};
use gpt3_rs::Request;
use log::{debug, info};
use poise::{BoxFuture, Event, FrameworkContext};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

pub fn listener<'a>(
    context: &'a poise::serenity_prelude::Context,
    event: &'a Event<'a>,
    _framework: FrameworkContext<'a, Arc<Data>, Error>,
    data: &'a Arc<Data>,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        if let Event::Message { new_message } = event {
            let mut last_message = data.last_message.write().await;
            let config = data.config.read().await;
            let chance = config.chance;
            let context_len = config.max_context_len;
            let minimum_score = config.minimum_score;
            let cooldown = config.cooldown;

            if new_message.author.id == BOT_ID {
                return Ok(());
            }
            if last_message.elapsed().as_secs() < cooldown as u64 {
                return Ok(());
            }

            if fastrand::u8(0..=100) <= chance {
                debug!("message chance occured");
                let mut phrases = data.phrases.write().await;
                let phrases_content = phrases
                    .iter()
                    .map(|phrase| phrase.content.as_str())
                    .collect::<Vec<_>>();

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

                // build search query
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

                if let Some(catchphrase) = index.and_then(|i| phrases.get_mut(i)) {
                    if catchphrase.last_phrase.elapsed() > Duration::from_secs(cooldown as u64) {
                        *last_message = Instant::now();
                        catchphrase.last_phrase = Instant::now();
                        let content = &catchphrase.content;
                        info!("found catchphrase: {}", content);
                        new_message.channel_id.say(&context.http, content).await?;
                    }
                }
            }
        }
        Ok(())
    })
}
