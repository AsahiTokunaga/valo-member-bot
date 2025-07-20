use std::str::FromStr;

use redis::AsyncTypedCommands;
use serenity::all::{ButtonStyle, CacheHttp, ChannelId, CreateActionRow, CreateButton, CreateEmbed, CreateMessage, Http, MessageId};

use crate::{bot::{colors::PIN_MESSAGE_COLOR, types::RedisClient}, config, error::{BotError, DbError}};

pub async fn entry<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &mut RedisClient) -> Result<(), BotError> {
  delete_latest(http, redis_client).await?;
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
  let latest_entry = channel.send_message(http, entry_panel).await?;
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  conn.set("latest_entry", latest_entry.id.get()).await.map_err(DbError::from)?;
  drop(conn);
  Ok(())
}

async fn delete_latest<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &mut RedisClient) -> Result<(), BotError> {
  let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
  let channel = ChannelId::from_str(&config::get("CHANNEL_ID")?)?;
  match conn.get("latest_entry").await {
    Ok(Some(message_id)) => {
      drop(conn);
      let message = MessageId::from_str(&message_id)?;
      channel.delete_message(http, message).await?;
      return Ok(());
    }
    _ => {
      drop(conn);
      return Ok(());
    }
  }
}
