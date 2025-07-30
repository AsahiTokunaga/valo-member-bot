use std::sync::Arc;

use futures::TryFutureExt;
use serenity::all::{CreateEmbed, EditWebhookMessage, Http, MessageId};

use crate::{bot::{panels::get_button, types::RedisClient}, error::BotError};

pub async fn edit(http: Arc<Http>, redis_client: Arc<RedisClient>, message: MessageId, is_fill: bool) -> Result<(), BotError> {
  let webhook = redis_client.get_webhook(http.clone()).await?;
  let (webhook_data, old_message) = tokio::try_join!(
    redis_client.get_webhook_data(message),
    webhook.get_message(http.clone(), None, message).map_err(BotError::from)
  )?;
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
  let new_buttons = get_button(is_fill);
  new_message = new_message.components(vec![new_buttons]);

  tokio::spawn(async move {
    if let Err(e) = webhook.edit_message(http, message, new_message).await {
      tracing::warn!(error = %e, "Failed to edit message");
    }
  });
  Ok(())
}
