use redis::AsyncTypedCommands;
use serenity::all::{MessageId, UserId};

use crate::{bot::types::RedisClient, error::{BotError, DbError}};

pub enum LeaveResponse {
  CreatorLeave,
  NotJoined,
  Left,
  Expired,
}

pub async fn leave(redis_client: &mut RedisClient, leave_user: UserId, message: MessageId) -> Result<LeaveResponse, BotError> {
  let webhook_data = match redis_client.get_webhook_data(message).await {
        Ok(data) => data,
        Err(_) => return Ok(LeaveResponse::Expired),
  };
  if webhook_data.creator == leave_user {
    return Ok(LeaveResponse::CreatorLeave);
  }
  if !webhook_data.joined.contains(&leave_user) {
    Ok(LeaveResponse::NotJoined)
  } else {
    let mut joined = webhook_data.joined.clone();
    joined.retain(|&u| u != leave_user);
    let joined_string = joined
      .iter()
      .map(|&u| format!("{}", u.get()))
      .collect::<Vec<String>>()
      .join(",");
    let mut conn = redis_client.connection.get().await.map_err(DbError::from)?;
    conn.hset(message.get(), "joined", joined_string).await.map_err(DbError::from)?;
    drop(conn);
    Ok(LeaveResponse::Left)
  }
}