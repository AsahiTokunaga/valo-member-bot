use redis::AsyncTypedCommands;
use serenity::all::{MessageId, UserId};
use crate::{bot::types::RedisClient, error::BotError};

pub enum JoinResponse {
    AlreadyJoined,
    Joined,
    Expired,
}

pub async fn join(redis_client: &mut RedisClient, join_user: UserId, message: MessageId) -> Result<JoinResponse, BotError> {
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
    let mut conn = redis_client.connection.lock().await;
    conn.hset(message.get(), "joined", joined_string).await?;
    drop(conn);
    Ok(JoinResponse::Joined)
  }
}
