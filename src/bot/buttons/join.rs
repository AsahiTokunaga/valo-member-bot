use redis::AsyncTypedCommands;
use serenity::all::{MessageId, UserId};
use crate::{bot::types::RedisClient, error::{BotError, DbError}};

pub enum JoinResponse {
    AlreadyJoined,
    Joined,
    Expired,
}

pub async fn join(redis_client: &RedisClient, join_user: UserId, message: MessageId) -> Result<JoinResponse, BotError> {
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
    conn.hset(message.get(), "joined", joined_string).await.map_err(DbError::from)?;
    drop(conn);
    Ok(JoinResponse::Joined)
  }
}
