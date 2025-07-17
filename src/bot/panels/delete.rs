use serenity::all::{CacheHttp, Http, MessageId};

use crate::{bot::types::RedisClient, error::BotError};

pub async fn delete<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &mut RedisClient, message: MessageId) -> Result<(), BotError> {
  let webhook = redis_client.get_webhook(http);
  webhook.await?.delete_message(http, None, message).await?;
  Ok(())
}
