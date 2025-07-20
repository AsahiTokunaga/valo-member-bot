use serenity::all::{CacheHttp, CreateEmbed, EditWebhookMessage, Http, MessageId};

use crate::{bot::{panels::get_button, types::RedisClient}, error::BotError};

pub async fn edit<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &RedisClient, message: MessageId)  -> Result<bool, BotError>{
  let webhook_data = redis_client.get_webhook_data(message).await?;
  let webhook = redis_client.get_webhook(http).await?;
  let old_message = webhook.get_message(http, None, message).await?;
  let embed = old_message.embeds.first().cloned().ok_or(BotError::EmbedBroken("embed"))?;
  let joined_users = webhook_data
    .joined
    .iter()
    .map(|&u| format!("<@{}>", u.get()))
    .collect::<Vec<String>>()
    .join("\n");
  let new_embed = CreateEmbed::new()
    .title(format!("({}/{})",webhook_data.joined.len(), u8::from(webhook_data.member)))
    .color(embed.colour.ok_or(BotError::EmbedBroken("color"))?)
    .description(embed.description.ok_or(BotError::EmbedBroken("description"))?)
    .thumbnail(embed.thumbnail.map_or(String::new(), |t| t.url))
    .field("参加者", joined_users, false);
  let mut new_message = EditWebhookMessage::new()
    .embed(new_embed);
  let is_fill = webhook_data.joined.len() == u8::from(webhook_data.member) as usize;
  let new_buttons = get_button(is_fill);
  new_message = new_message.components(vec![new_buttons]);
  webhook.edit_message(http, message, new_message).await?;
  Ok(is_fill)
}
