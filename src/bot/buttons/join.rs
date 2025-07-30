use std::sync::Arc;

use redis::AsyncTypedCommands;
use serenity::all::{MessageId, UserId};
use crate::{bot::types::RedisClient, error::{BotError, DbError}};

pub enum JoinResponse {
    AlreadyJoined,
    Joined,
    Expired,
}

pub async fn join(redis_client: Arc<RedisClient>, join_user: UserId, message: MessageId) -> Result<JoinResponse, BotError> {
  let webhook_data = match redis_client.get_webhook_data(message).await {
        Ok(data) => data,
        Err(_) => return Ok(JoinResponse::Expired),
  };
  if webhook_data.joined.contains(&join_user) {
    Ok(JoinResponse::AlreadyJoined)
  } else {
    let mut joined = webhook_data.joined.clone();
    joined.push(join_user);
    let joined_string = joined.iter()
      .map(|u| format!("{}", u.get()))
      .collect::<Vec<String>>()
      .join(",");
    let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
    tokio::spawn(async move {
      if let Err(e) = conn.hset(message.get(), "joined", joined_string).await {
        tracing::warn!(error = %e, "Failed to update joined users in Redis");
      }
      drop(conn);
    });
    Ok(JoinResponse::Joined)
  }
}
