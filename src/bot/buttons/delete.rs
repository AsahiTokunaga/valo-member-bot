use redis::AsyncTypedCommands;
use serenity::all::{MessageId, UserId};

use crate::{bot::types::RedisClient, error::{BotError, DbError}};

pub enum DeleteResponse {
  NotCreator,
  NotJoined,
  Deleted,
  Expired,
}

pub async fn delete(redis_client: &mut RedisClient, delete_user: UserId, message: MessageId) -> Result<DeleteResponse, BotError> {
  let webhook_data = match redis_client.get_webhook_data(message).await {
        Ok(data) => data,
        Err(_) => return Ok(DeleteResponse::Expired),
  };
  if webhook_data.creator != delete_user {
    return Ok(DeleteResponse::NotCreator);
  }
  if !webhook_data.joined.contains(&delete_user) {
    return Ok(DeleteResponse::NotJoined);
  } else {
    let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
    conn.del(message.get()).await.map_err(DbError::from)?;
    drop(conn);
    Ok(DeleteResponse::Deleted)
  }
}