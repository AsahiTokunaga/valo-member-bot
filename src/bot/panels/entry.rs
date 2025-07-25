use std::{str::FromStr, sync::Arc};

use redis::AsyncTypedCommands;
use serenity::all::{
  ButtonStyle, CacheHttp, ChannelId, CreateActionRow, CreateButton,
  CreateEmbed, CreateMessage, GetMessages, Http, MessageId
};

use crate::{
  bot::{colors::PIN_MESSAGE_COLOR, types::RedisClient},
  config, error::{BotError, DbError}
};

pub async fn entry(http: &Arc<Http>, redis_client: &RedisClient) -> Result<(), BotError> {
  if !is_updatable(http, redis_client).await? { return Ok(()); }
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
  if let Err(e) = delete_latest(http, redis_client).await {
    tracing::warn!(error = %e, "Failed to delete latest entry message");
    return Err(e);
  }

  let conn = tokio::spawn({
    let redis_client = redis_client.clone();
    async move {
      redis_client.connection.get().await
    }
  });
  let latest_entry = match channel.send_message(http, entry_panel).await {
    Ok(message) => message.id,
    Err(e) => {
      tracing::warn!(error = %e, "Failed to send entry message");
      return Err(BotError::from(e));
    }
  };

  tokio::spawn({
    let mut conn = conn.await?.map_err(DbError::from)?;
    async move {
      if let Err(e) = conn.set("latest_entry", latest_entry.get()).await {
        tracing::warn!(error = %e, "Failed to set latest entry message ID in Redis");
      }
    }
  });
  Ok(())
}

async fn delete_latest(http: &Arc<Http>, redis_client: &RedisClient) -> Result<(), BotError> {
  let channel = ChannelId::from_str(&config::get("CHANNEL_ID")?)?;
  tokio::spawn({
    let redis_client = redis_client.clone();
    let http = http.clone();
    async move {
      let conn = redis_client.connection.get();
      let result: Result<(), BotError> = async {
        let mut conn = conn.await.map_err(DbError::from)?;
        match conn.get("latest_entry").await {
          Ok(Some(message_id)) => {
            let message = MessageId::from_str(&message_id).map_err(BotError::from)?;
            channel.delete_message(&http, message).await.map_err(BotError::from)?;
            Ok(())
          }
          _ => Ok(())
        }
      }.await;

      if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to delete latest entry message");
      }
    }
  });
  Ok(())
}

async fn is_updatable<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &RedisClient) -> Result<bool, BotError> {
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  let latest_entry: Option<String> = conn.get("latest_entry").await.map_err(DbError::from)?;
  if let Some(message_id) = latest_entry {
    let messages = ChannelId::from_str(&config::get("CHANNEL_ID")?)?
      .messages(http, GetMessages::new().after(MessageId::from_str(&message_id)?).limit(5))
      .await
      .map_err(BotError::from)?;
    return Ok(4 <= messages.len()); 
  }
  Ok(true)
}
