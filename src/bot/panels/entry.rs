use std::str::FromStr;

use redis::AsyncTypedCommands;
use serenity::all::{ButtonStyle, CacheHttp, ChannelId, Context, CreateActionRow, CreateButton, CreateEmbed, CreateMessage, GetMessages, Http, MessageId};

use crate::{bot::{colors::PIN_MESSAGE_COLOR, types::RedisClient}, config, error::{BotError, DbError}};

pub async fn entry(ctx: &Context, redis_client: &RedisClient) -> Result<(), BotError> {
  if !is_updatable(ctx.http(), redis_client).await? { return Ok(()); }
  let embed = CreateEmbed::new()
    .description("# 募集を作成！\n下のボタンを押して、アンレート、コンペティティブ、カスタムの募集を作成しましょう！")
    .color(PIN_MESSAGE_COLOR);
  let entry_panel = CreateMessage::new()
    .embed(embed)
    .components(vec![CreateActionRow::Buttons(vec![
      CreateButton::new("募集を作成")
        .style(ButtonStyle::Secondary)
        .label("募集を作成")
    ])]);

  let channel = ChannelId::from_str(&config::get("CHANNEL_ID")?)?;
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  tokio::spawn({
    let redis_client = redis_client.clone();
    let ctx = ctx.clone();
    async move {
      if let Err(e) = delete_latest(ctx.http(), &redis_client).await {
          tracing::warn!(error = %e, "Failed to delete latest entry");
      }
      let latest_entry = match channel.send_message(ctx.http(), entry_panel).await {
        Ok(msg) => msg,
        Err(e) => {
          tracing::warn!(error = %e, "Failed to send entry panel message");
          return;
        },
      };
      if let Err(e) = conn.set("latest_entry", latest_entry.id.get()).await {
        tracing::warn!(error = %e, "Failed to set latest entry in Redis");
      }
    }
  });
  tracing::info!("Entry panel updated successfully");
  Ok(())
}

async fn delete_latest<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &RedisClient) -> Result<(), BotError> {
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  let channel = ChannelId::from_str(&config::get("CHANNEL_ID")?)?;
  match conn.get("latest_entry").await {
    Ok(Some(message_id)) => {
      let message = MessageId::from_str(&message_id)?;
      channel.delete_message(http, message).await?;
      return Ok(());
    }
    _ => return Ok(()),
  }
}

async fn is_updatable<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &RedisClient) -> Result<bool, BotError> {
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  let latest_entry: Option<String> = conn.get("latest_entry").await.map_err(DbError::from)?;
  if let Some(message_id) = latest_entry {
    let messages = ChannelId::from_str(&config::get("CHANNEL_ID")?)?
      .messages(http, GetMessages::new().after(MessageId::from_str(&message_id)?).limit(5))
      .await
      .map_err(BotError::from)?;
    return Ok(3 <= messages.len()); 
  }
  Ok(true)
}
